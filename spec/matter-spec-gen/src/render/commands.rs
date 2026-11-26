use std::result::Result as StdResult;

use genco::{quote, tokens::quoted};

use super::{
    context::{PushLine as _, Tokens, TypeContext, doc_comment, format_hex},
    record::{IdPrefix, MemberPartition, RecordConstructors, RecordStructDef, SingleMandatoryFrom, StructAttrs},
    structs::{emit_fields_decode_and_construct, emit_fields_into_vec, resolve_fields},
};
use crate::{
    error::Error,
    model::{ClusterDef, CommandDef},
};

type Result<T, E = Error> = StdResult<T, E>;

pub(super) fn emit_commands(
    out: &mut Tokens,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ctx: &TypeContext<'_>,
) -> Result<()> {
    if cluster.commands.is_empty() {
        return Ok(());
    }

    let mut cmd_tokens_list = Vec::new();
    for command in &cluster.commands {
        cmd_tokens_list.push(single_command_tokens(command, cluster, cluster_struct, ctx)?);
    }

    let module_tokens: Tokens = quote! {
        pub mod commands {
            #[allow(unused_imports, reason = "Generated command imports")]
            use alloc::{string::String, vec::Vec};

            #[allow(unused_imports, reason = "Generated command imports")]
            use crate::{
                types,
                model::{
                    ActionId, AttributeId, AttributeValue, ClusterId, CommandDirection, CommandId, DataVersion,
                    DeviceTypeId, EndpointId, EventId, EventNumber, EventPriority, FabricId, FabricIndex, FieldId,
                    GroupId, MatterStatus, NodeId, Nullable, Optional, TransactionId, VendorId,
                },
            };
            #[cfg(feature = $(quoted("serde")))]
            #[allow(unused_imports, reason = "Generated command imports")]
            use serde::{Deserialize, Serialize};

            $(for cmd_tok in cmd_tokens_list join ($['\n']) =>
                $cmd_tok
            )
        }
    };

    out.append(module_tokens);
    out.push_blank();
    Ok(())
}

pub(super) fn single_command_tokens(
    cmd: &CommandDef,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ctx: &TypeContext<'_>,
) -> Result<Tokens> {
    let CommandDef {
        name,
        code,
        source,
        response,
        args,
        description,
        ..
    } = cmd;
    let direction = if source.eq_ignore_ascii_case("server") {
        quote!(CommandDirection::ServerToClient)
    } else if source.eq_ignore_ascii_case("client") {
        quote!(CommandDirection::ClientToServer)
    } else {
        return Err(Error::UnknownCommandSource {
            command: name.clone(),
            source_name: source.clone(),
        });
    };
    let response_type = if let Some(resp) = response {
        if cluster
            .commands
            .iter()
            .any(|command| command.name.eq_ignore_ascii_case(resp))
        {
            resp.clone()
        } else {
            "()".to_owned()
        }
    } else {
        "()".to_owned()
    };

    let cmd_name = name;
    let cmd_id_hex = format_hex(u64::from(*code), 2);

    if args.is_empty() {
        return Ok(emit_unit_command_tokens(
            cmd_name,
            description,
            cluster_struct,
            &cmd_id_hex,
            &direction,
            &response_type,
            !cmd.source.eq_ignore_ascii_case("server"),
        ));
    }

    emit_struct_command_tokens(
        cmd,
        cluster,
        cluster_struct,
        &cmd_id_hex,
        &direction,
        &response_type,
        ctx,
    )
}

pub(super) fn emit_command_enum_tokens(
    cluster: &ClusterDef,
    cluster_struct_name: &str,
    ctx: &TypeContext<'_>,
) -> Tokens {
    let can_derive_eq_hash = cluster.commands.iter().filter(|cmd| !cmd.args.is_empty()).all(|cmd| {
        resolve_fields(&cluster.name, &cmd.args, true, ctx)
            .is_ok_and(|fields| MemberPartition::new(&fields).can_derive_eq_hash(ctx))
    });

    let client_cmds: Vec<&CommandDef> = cluster
        .commands
        .iter()
        .filter(|c| c.source.eq_ignore_ascii_case("client"))
        .collect();
    let server_cmds: Vec<&CommandDef> = cluster
        .commands
        .iter()
        .filter(|c| c.source.eq_ignore_ascii_case("server"))
        .collect();

    quote! {
        $(if !client_cmds.is_empty() =>
            $(emit_single_command_enum("Command", &client_cmds, cluster_struct_name, true, can_derive_eq_hash))
        )
        $(if !server_cmds.is_empty() =>
            $(emit_single_command_enum("CommandResponse", &server_cmds, cluster_struct_name, false, can_derive_eq_hash))
        )
    }
}

fn emit_unit_command_tokens(
    cmd_name: &str,
    description: &str,
    cluster_struct: &str,
    cmd_id_hex: &str,
    direction: &Tokens,
    response_type: &str,
    is_client: bool,
) -> Tokens {
    let resp = if is_client { response_type } else { "()" };
    quote! {
        $(doc_comment(description))
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize))]
        #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema))]
        pub struct $cmd_name;

        impl $cmd_name {
            pub const CLUSTER_ID: ClusterId = super::$cluster_struct::CLUSTER_ID;
            pub const COMMAND_ID: CommandId = CommandId::new($cmd_id_hex);
            pub const NAME: &'static str = $(quoted(cmd_name));
            pub const DIRECTION: CommandDirection = $direction;
        }

        crate::impl_matter_command!($cmd_name, super::$cluster_struct, $resp, true);
    }
}

fn emit_struct_command_tokens(
    cmd: &CommandDef,
    cluster: &ClusterDef,
    cluster_struct: &str,
    cmd_id_hex: &str,
    direction: &Tokens,
    response_type: &str,
    ctx: &TypeContext<'_>,
) -> Result<Tokens> {
    let CommandDef {
        name: cmd_name,
        args,
        description,
        source,
        ..
    } = cmd;

    let resolved_fields = resolve_fields(&cluster.name, args, true, ctx)?;
    let partition = MemberPartition::new(&resolved_fields);
    let can_derive_eq_hash = partition.can_derive_eq_hash(ctx);

    let (decode_loop, construct_fields) =
        emit_fields_decode_and_construct(&resolved_fields, true, "MatterStatus::InvalidCommand");
    let encode_vec_body = emit_fields_into_vec(&resolved_fields, "self", "Self", false);

    let attrs = if partition.is_empty() {
        StructAttrs::unit(true)
    } else {
        StructAttrs::model(can_derive_eq_hash, true)
    };
    let struct_doc = quote!($(doc_comment(description)));
    let struct_def = quote! {
        $(RecordStructDef {
            doc: struct_doc,
            name: cmd_name,
            partition: &partition,
            attrs,
            cfg_gate: None,
        })
    };
    let methods_tokens = quote! {
        $(RecordConstructors {
            partition: &partition,
            prefix: IdPrefix::Field,
            too_many_args_reason: "Command has large mandatory argument set matching CSA specification",
            single_char_reason: Some("Command field names match CSA specification"),
        })
    };
    let single_mand_from = quote!($(SingleMandatoryFrom::maybe(cmd_name, &partition.mandatory)));

    let short_field_count = partition.short_field_count();
    let many_single_char_reason = "Specification command contains multiple CSA single-character argument names";
    let resp_type = if source.eq_ignore_ascii_case("server") {
        "()"
    } else {
        response_type
    };

    Ok(quote! {
        $struct_def

        impl $cmd_name {
            pub const CLUSTER_ID: ClusterId = super::$cluster_struct::CLUSTER_ID;
            pub const COMMAND_ID: CommandId = CommandId::new($cmd_id_hex);
            pub const NAME: &'static str = $(quoted(cmd_name));
            pub const DIRECTION: CommandDirection = $direction;

            $methods_tokens

            #[must_use]
            pub fn into_fields(self) -> Vec<(FieldId, AttributeValue)> {
                $encode_vec_body
            }

            $(if short_field_count >= 5 =>
                #[expect(clippy::many_single_char_names, reason = $(quoted(many_single_char_reason)))]
            )
            pub(crate) fn decode_fields(
                fields: Vec<(FieldId, AttributeValue)>,
            ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
                $(for arg in partition.all join ($['\r']) =>
                    let mut $(&arg.fname) = None;
                )
                $decode_loop
                Ok(Self {
                    $construct_fields
                })
            }
        }

        $single_mand_from

        crate::impl_matter_command!($cmd_name, super::$cluster_struct, $resp_type, false);
    })
}

fn emit_single_command_enum(
    enum_name: &str,
    cmds: &[&CommandDef],
    cluster_struct_name: &str,
    is_client: bool,
    can_derive_eq_hash: bool,
) -> Tokens {
    let has_multiple = cmds.len() > 1;
    let empty_cmds: Vec<&CommandDef> = cmds.iter().copied().filter(|c| c.args.is_empty()).collect();
    let non_empty_cmds: Vec<&CommandDef> = cmds.iter().copied().filter(|c| !c.args.is_empty()).collect();
    let expected_dir = if is_client {
        "CommandDirection::ClientToServer"
    } else {
        "CommandDirection::ServerToClient"
    };
    let doc = if is_client {
        format!("/// Dynamic client-to-server command variant matching for [`{cluster_struct_name}`].")
    } else {
        format!("/// Dynamic server-to-client command response variant matching for [`{cluster_struct_name}`].")
    };

    quote! {
        $doc
        #[derive(Debug, Clone, PartialEq $(if can_derive_eq_hash => , Eq, Hash))]
        #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize))]
        #[cfg_attr(
            feature = $(quoted("serde")),
            serde(tag = $(quoted("command")), content = $(quoted("payload")))
        )]
        #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema))]
        #[allow(clippy::large_enum_variant, reason = "Generated command enum")]
        pub enum $enum_name {
            $(for cmd in cmds join ($['\r']) =>
                $(if cmd.args.is_empty() =>
                    $(&cmd.name),
                )
                $(if !cmd.args.is_empty() =>
                    $(&cmd.name)(commands::$(&cmd.name)),
                )
            )
        }

        impl $enum_name {
            pub const CLUSTER_ID: ClusterId = $cluster_struct_name::CLUSTER_ID;

            #[inline]
            #[must_use]
            pub const fn id(&self) -> CommandId {
                match self {
                    $(for cmd in cmds join ($['\r']) =>
                        $(if cmd.args.is_empty() =>
                            Self::$(&cmd.name) => commands::$(&cmd.name)::COMMAND_ID,
                        )
                        $(if !cmd.args.is_empty() =>
                            Self::$(&cmd.name)(_) => commands::$(&cmd.name)::COMMAND_ID,
                        )
                    )
                }
            }

            #[inline]
            #[must_use]
            pub const fn direction(&self) -> CommandDirection {
                $expected_dir
            }

            #[inline]
            #[must_use]
            pub const fn is_empty_payload(&self) -> bool {
                match self {
                    $(if !empty_cmds.is_empty() =>
                        $(for cmd in &empty_cmds join ( | ) => Self::$(&cmd.name)) => true,
                    )
                    $(if !non_empty_cmds.is_empty() =>
                        $(for cmd in &non_empty_cmds join ( | ) => Self::$(&cmd.name)(_)) => false,
                    )
                }
            }

            #[inline]
            pub fn from_model(cmd: model::Command) -> Result<Self, CommandError> {
                Self::try_from(cmd)
            }

            $(emit_command_enum_select_described(cmds, is_client))
        }

        $(emit_command_enum_wire_impls(enum_name, cmds, expected_dir, has_multiple))
        $(emit_command_enum_dispatch_impls(enum_name, cmds, cluster_struct_name, is_client))
    }
}

fn emit_command_enum_select_described(cmds: &[&CommandDef], is_client: bool) -> Tokens {
    let is_client_str = if is_client { "true" } else { "false" };
    quote! {
        #[cfg(all(feature = $(quoted("dispatch")), feature = $(quoted("serde"))))]
        const DESCRIBED_VARIANTS: &'static [DescribedCommandVariant<Self>] = &[
            $(for cmd in cmds join ($['\r']) =>
                (
                    $(quoted(&cmd.name)),
                    commands::$(&cmd.name)::COMMAND_ID.raw(),
                    $is_client_str,
                    $(if cmd.args.is_empty() =>
                        |payload| payload.deserialize_into::<commands::$(&cmd.name)>().map(|_| Self::$(&cmd.name)),
                    )
                    $(if !cmd.args.is_empty() =>
                        |payload| payload.deserialize_into::<commands::$(&cmd.name)>().map(Self::$(&cmd.name)),
                    )
                ),
            )
        ];

        crate::impl_select_described!(cmd, "command");
    }
}

fn emit_command_enum_wire_impls(
    enum_name: &str,
    cmds: &[&CommandDef],
    expected_dir: &str,
    has_multiple: bool,
) -> Tokens {
    quote! {
        impl From<$enum_name> for model::Command {
            fn from(cmd: $enum_name) -> Self {
                match cmd {
                    $(for cmd in cmds join ($['\r']) =>
                        $(if cmd.args.is_empty() =>
                            $enum_name::$(&cmd.name) => model::Command::from(commands::$(&cmd.name)),
                        )
                        $(if !cmd.args.is_empty() =>
                            $enum_name::$(&cmd.name)(inner) => model::Command::from(inner),
                        )
                    )
                }
            }
        }

        impl TryFrom<model::Command> for $enum_name {
            type Error = CommandError;

            fn try_from(cmd: model::Command) -> Result<Self, Self::Error> {
                if cmd.cluster_id != Self::CLUSTER_ID {
                    return Err(CommandError::ClusterIdMismatch {
                        expected: Self::CLUSTER_ID,
                        actual: cmd.cluster_id,
                    });
                }
                if cmd.direction != $expected_dir {
                    return Err(CommandError::DirectionMismatch {
                        expected: $expected_dir,
                        actual: cmd.direction,
                    });
                }
                match cmd.command_id {
                    $(for cmd in cmds join ($['\r']) =>
                        commands::$(&cmd.name)::COMMAND_ID => {
                            commands::$(&cmd.name)::decode_fields(cmd.payload)
                                $(if cmd.args.is_empty() => .map(|_| Self::$(&cmd.name)))
                                $(if !cmd.args.is_empty() => .map(Self::$(&cmd.name)))
                                .map_err(|(field_id, status)| CommandError::InvalidPayload {
                                    id: cmd.command_id,
                                    field_id,
                                    status,
                                })
                        }
                    )
                    _ => Err(CommandError::UnsupportedCommand(cmd.command_id)),
                }
            }
        }

        $(if has_multiple =>
            $(for cmd in cmds join ($['\r']) =>
                $(if cmd.args.is_empty() =>
                    crate::impl_cluster_member_variant!(command_unit, $enum_name, commands::$(&cmd.name), $(&cmd.name));
                )
                $(if !cmd.args.is_empty() =>
                    crate::impl_cluster_member_variant!(command, $enum_name, commands::$(&cmd.name), $(&cmd.name));
                )
            )
        )
        $(if !has_multiple =>
            $(for cmd in cmds join ($['\r']) =>
                $(if cmd.args.is_empty() =>
                    crate::impl_cluster_member_variant!(
                        command_unit_single,
                        $enum_name,
                        commands::$(&cmd.name),
                        $(&cmd.name)
                    );
                )
                $(if !cmd.args.is_empty() =>
                    crate::impl_cluster_member_variant!(
                        command_single,
                        $enum_name,
                        commands::$(&cmd.name),
                        $(&cmd.name)
                    );
                )
            )
        )
    }
}

fn emit_command_enum_dispatch_impls(
    enum_name: &str,
    cmds: &[&CommandDef],
    cluster_struct_name: &str,
    is_client: bool,
) -> Tokens {
    let empty_cmds: Vec<_> = cmds.iter().copied().filter(|c| c.args.is_empty()).collect();
    let non_empty_cmds: Vec<_> = cmds.iter().copied().filter(|c| !c.args.is_empty()).collect();
    let kind = if is_client { "command" } else { "command_response" };
    quote! {
        crate::impl_cluster_enum_dispatch!(
            $kind,
            $enum_name,
            $cluster_struct_name,
            empty = [
                $(for cmd in &empty_cmds join ($['\r']) =>
                    $(&cmd.name),
                )
            ],
            payload = [
                $(for cmd in &non_empty_cmds join ($['\r']) =>
                    $(&cmd.name),
                )
            ]
        );
    }
}
