//! Cluster state struct, patch struct, `ModeBaseCluster` impl, and top-level member enum emission.

use genco::{quote, tokens::quoted};

use super::{
    Result, attribute_enum::emit_attribute_enum_tokens, patch::emit_cluster_patch_tokens, resolve_cluster_attributes,
};
use crate::{
    model::{ClusterDef, EnumDef, StructDef},
    render::{
        commands::emit_command_enum_tokens,
        context::{PushLine as _, Tokens, TypeContext, format_hex},
        events::emit_event_enum_tokens,
        record::{IdPrefix, MemberPartition, RecordConstructors, RecordStructDef, SingleMandatoryFrom, StructAttrs},
    },
};

/// Emits the cluster state struct, sparse patch struct, trait impls, and attribute/command/event enums.
pub(super) fn emit_cluster_struct_and_attribute_enum(
    out: &mut Tokens,
    cluster: &ClusterDef,
    cluster_struct_name: &str,
    effective_enums: &[EnumDef],
    effective_structs: &[StructDef],
    ctx: &TypeContext<'_>,
) -> Result<()> {
    let resolved_attrs = resolve_cluster_attributes(cluster, ctx)?;
    let partition = MemberPartition::new(&resolved_attrs);

    let can_derive_eq_hash = partition.can_derive_eq_hash(ctx);

    let attrs = if partition.is_empty() {
        StructAttrs::unit(true)
    } else {
        StructAttrs::model(can_derive_eq_hash, true)
    };

    let cluster_struct_def = emit_cluster_struct_definitions(cluster_struct_name, &cluster.name, &partition, attrs);

    let methods_tokens = emit_cluster_methods_and_impl(cluster, cluster_struct_name, &partition);

    let is_mode_base = (effective_enums.iter().any(|enm| enm.name == "ModeTag")
        || ctx.global_defs.enums.iter().any(|enm| enm.name == "ModeTag"))
        && (effective_structs.iter().any(|strct| strct.name == "ModeTagStruct")
            || ctx
                .global_defs
                .structs
                .iter()
                .any(|strct| strct.name == "ModeTagStruct"))
        && (effective_structs.iter().any(|strct| strct.name == "ModeOptionStruct")
            || ctx
                .global_defs
                .structs
                .iter()
                .any(|strct| strct.name == "ModeOptionStruct"))
        && matches!(partition.mandatory.as_slice(), [m0, m1] if m0.fname == "supported_modes" && m1.fname == "current_mode");

    let single_mand_from = quote!($(SingleMandatoryFrom::maybe(cluster_struct_name, &partition.mandatory)));
    let wire_conversions = emit_cluster_wire_conversions(cluster_struct_name);
    let is_empty_str = if partition.is_empty() { "true" } else { "false" };

    let patch_name = format!("{cluster_struct_name}Patch");
    let patch_attrs = if partition.is_empty() {
        StructAttrs::unit(true)
    } else {
        StructAttrs::model(can_derive_eq_hash, true)
    };
    let has_features = !cluster.features.is_empty();
    let features_ty = if has_features { "FeatureMap" } else { "u32" };
    let has_commands = !cluster.commands.is_empty();
    let has_events = !cluster.events.is_empty();
    let has_client_commands = cluster.commands.iter().any(|c| c.source != "server");
    let has_server_commands = cluster.commands.iter().any(|c| c.source == "server");

    let patch_tokens = emit_cluster_patch_tokens(
        cluster_struct_name,
        &patch_name,
        patch_attrs,
        &partition,
        features_ty,
        has_client_commands,
        has_server_commands,
        has_events,
    );

    out.append(quote! {
        $cluster_struct_def

        $patch_tokens

        impl $cluster_struct_name {
            $methods_tokens
        }

        $(if is_mode_base =>
            impl crate::traits::ModeBaseCluster for $cluster_struct_name {
                type ModeTag = ModeTag;
                type ModeTagStruct = ModeTagStruct;
                type ModeOptionStruct = ModeOptionStruct;

                #[inline]
                fn new(supported_modes: Vec<Self::ModeOptionStruct>, current_mode: u8) -> Self {
                    Self::new(supported_modes, current_mode)
                }
            }
        )

        $single_mand_from
        $wire_conversions

        crate::impl_matter_describe!(cluster, $cluster_struct_name, $is_empty_str);

        $(if !partition.is_empty() =>
            $(emit_attribute_enum_tokens(cluster_struct_name, partition.all, can_derive_eq_hash))
        )
        $(if has_commands =>
            $(emit_command_enum_tokens(cluster, cluster_struct_name, ctx))
        )
        $(if has_events =>
            $(emit_event_enum_tokens(cluster, cluster_struct_name, ctx))
        )
    });
    out.push_blank();

    Ok(())
}

fn emit_cluster_struct_definitions(
    cluster_struct_name: &str,
    cluster_name: &str,
    partition: &MemberPartition<'_>,
    attrs: StructAttrs,
) -> Tokens {
    quote! {
        $(RecordStructDef {
            doc: quote!($(format!("/// {cluster_name} cluster."))),
            name: cluster_struct_name,
            partition,
            attrs,
            cfg_gate: None,
        })
    }
}

fn emit_to_attributes_tokens(partition: &MemberPartition<'_>) -> Tokens {
    let mandatory = &partition.mandatory;
    let optional = &partition.optional;

    if partition.is_empty() {
        quote! {
            #[must_use]
            pub fn to_attributes(&self) -> Vec<(AttributeId, AttributeValue)> {
                Vec::new()
            }
        }
    } else if optional.is_empty() {
        quote! {
            #[must_use]
            pub fn to_attributes(&self) -> Vec<(AttributeId, AttributeValue)> {
                vec![
                    $(for attr in mandatory join ($['\r']) =>
                        $(if attr.is_copy() =>
                            (
                                Self::$(attr.attr_const_ident()),
                                attributes::$(&attr.name)::new(self.$(&attr.fname)).into_value(),
                            ),
                        )
                        $(if !attr.is_copy() =>
                            (
                                Self::$(attr.attr_const_ident()),
                                attributes::$(&attr.name)::new(self.$(&attr.fname).clone()).into_value(),
                            ),
                        )
                    )
                ]
            }
        }
    } else {
        quote! {
            #[must_use]
            pub fn to_attributes(&self) -> Vec<(AttributeId, AttributeValue)> {
                $(if mandatory.is_empty() =>
                    let mut attrs = Vec::new();
                )
                $(if !mandatory.is_empty() =>
                    let mut attrs = vec![
                        $(for attr in mandatory join ($['\r']) =>
                            $(if attr.is_copy() =>
                                (
                                    Self::$(attr.attr_const_ident()),
                                    attributes::$(&attr.name)::new(self.$(&attr.fname)).into_value(),
                                ),
                            )
                            $(if !attr.is_copy() =>
                                (
                                    Self::$(attr.attr_const_ident()),
                                    attributes::$(&attr.name)::new(self.$(&attr.fname).clone()).into_value(),
                                ),
                            )
                        )
                    ];
                )
                $(for attr in optional join ($['\r']) =>
                    if let Optional::Present(v) = &self.$(&attr.fname) {
                        $(if attr.is_copy() =>
                            attrs.push((
                                Self::$(attr.attr_const_ident()),
                                attributes::$(&attr.name)::new(*v).into_value(),
                            ));
                        )
                        $(if !attr.is_copy() =>
                            attrs.push((
                                Self::$(attr.attr_const_ident()),
                                attributes::$(&attr.name)::new(v.clone()).into_value(),
                            ));
                        )
                    }
                )
                attrs
            }
        }
    }
}

fn emit_try_from_attributes_tokens(partition: &MemberPartition<'_>) -> Tokens {
    let many_single_char_reason = "Specification attribute single-character field names match CSA naming";

    if partition.is_empty() {
        quote! {
            pub(crate) fn decode_attributes_inner(
                attrs: &mut dyn Iterator<Item = (AttributeId, AttributeValue)>,
            ) -> Result<Self, ClusterError> {
                for (id, _) in attrs {
                    if !id.is_global() {
                        return Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(id)));
                    }
                }
                Ok(Self)
            }

            pub fn try_from_attributes(
                attrs: impl IntoIterator<Item = (AttributeId, AttributeValue)>,
            ) -> Result<Self, ClusterError> {
                Self::decode_attributes_inner(&mut attrs.into_iter())
            }

            #[must_use]
            pub fn into_attributes(self) -> Vec<(AttributeId, AttributeValue)> {
                Vec::new()
            }

            pub fn apply_patch(&mut self, _patch: <Self as crate::traits::ClusterSpec>::Patch) {}
        }
    } else {
        let resolved_attrs = partition.all;
        let mandatory = &partition.mandatory;
        let optional = &partition.optional;

        quote! {
            $(if partition.has_many_single_chars() =>
                #[expect(clippy::many_single_char_names, reason = $(quoted(many_single_char_reason)))]
            )
            pub(crate) fn decode_attributes_inner(
                attrs: &mut dyn Iterator<Item = (AttributeId, AttributeValue)>,
            ) -> Result<Self, ClusterError> {
                $(for attr in resolved_attrs join ($['\r']) =>
                    let mut $(&attr.fname) = None;
                )
                for (id, val) in attrs {
                    if id.is_global() {
                        continue;
                    }
                    match Attribute::try_from(model::Attribute::new(id, val)).map_err(ClusterError::InvalidAttribute)? {
                        $(for attr in resolved_attrs join ($['\r']) =>
                            Attribute::$(&attr.name)(decoded) => {
                                $(&attr.fname).is_none().then_some(()).ok_or(ClusterError::InvalidAttribute(
                                    AttributeError::InvalidValue {
                                        id: Self::$(attr.attr_const_ident()),
                                        field_id: None,
                                        status: MatterStatus::ConstraintError,
                                    },
                                ))?;
                                $(&attr.fname) = Some(decoded.into());
                            }
                        )
                    }
                }
                $(for attr in mandatory join ($['\r']) =>
                    let $(&attr.fname) = $(&attr.fname)
                        .ok_or(ClusterError::MissingMandatoryAttribute(Self::$(attr.attr_const_ident())))?;
                )
                $(for attr in optional join ($['\r']) =>
                    let $(&attr.fname) = Optional::from($(&attr.fname));
                )
                Ok(Self {
                    $(for attr in mandatory join ($['\r']) =>
                        $(&attr.fname),
                    )
                    $(for attr in optional join ($['\r']) =>
                        $(&attr.fname),
                    )
                })
            }

            pub fn try_from_attributes(
                attrs: impl IntoIterator<Item = (AttributeId, AttributeValue)>,
            ) -> Result<Self, ClusterError> {
                Self::decode_attributes_inner(&mut attrs.into_iter())
            }

            pub(crate) fn encode_attributes_inner(self, out: &mut Vec<(AttributeId, AttributeValue)>) {
                $(for attr in mandatory join ($['\r']) =>
                    out.push((
                        Self::$(attr.attr_const_ident()),
                        attributes::$(&attr.name)::from(self.$(&attr.fname)).into_value(),
                    ));
                )
                $(for attr in optional join ($['\r']) =>
                    if let Optional::Present(v) = self.$(&attr.fname) {
                        out.push((
                            Self::$(attr.attr_const_ident()),
                            attributes::$(&attr.name)::from(v).into_value(),
                        ));
                    }
                )
            }

            #[must_use]
            pub fn into_attributes(self) -> Vec<(AttributeId, AttributeValue)> {
                let mut out = Vec::new();
                self.encode_attributes_inner(&mut out);
                out
            }

            pub fn apply_patch(&mut self, patch: <Self as crate::traits::ClusterSpec>::Patch) {
                $(for attr in mandatory join ($['\r']) =>
                    if let Optional::Present(v) = patch.$(&attr.fname) {
                        self.$(&attr.fname) = v;
                    }
                )
                $(for attr in optional join ($['\r']) =>
                    if let Optional::Present(v) = patch.$(&attr.fname) {
                        self.$(&attr.fname) = Optional::Present(v);
                    }
                )
            }
        }
    }
}

fn emit_cluster_constructor_methods(
    cluster: &ClusterDef,
    cluster_id_hex: &str,
    cluster_revision_value: u16,
    partition: &MemberPartition<'_>,
) -> Tokens {
    let cluster_name = &cluster.name;
    quote! {
        pub const CLUSTER_ID: ClusterId = ClusterId::new($cluster_id_hex);
        pub const NAME: &'static str = $(quoted(cluster_name));
        pub const CLUSTER_REVISION: u16 = $cluster_revision_value;

        $(RecordConstructors {
            partition,
            prefix: IdPrefix::Attr,
            too_many_args_reason: "Cluster has large mandatory attribute set matching CSA specification",
            single_char_reason: None,
        })
    }
}

fn emit_cluster_methods_and_impl(
    cluster: &ClusterDef,
    cluster_struct_name: &str,
    partition: &MemberPartition<'_>,
) -> Tokens {
    let cluster_id_hex = format_hex(u64::from(cluster.code), 4);
    let cluster_revision_value = cluster.cluster_revision;

    let methods_tokens = emit_cluster_constructor_methods(cluster, &cluster_id_hex, cluster_revision_value, partition);

    let to_attributes_tokens = emit_to_attributes_tokens(partition);
    let try_from_attributes_tokens = emit_try_from_attributes_tokens(partition);

    quote! {
        $methods_tokens
        $(if cluster_struct_name == "OnOffCluster" =>
            pub const OFF: Self = Self {
                on_off: OnOff::Off,
                $(for opt in &partition.optional join ($['\r']) =>
                    $(&opt.fname): Optional::Omitted,
                )
            };
            pub const ON: Self = Self {
                on_off: OnOff::On,
                $(for opt in &partition.optional join ($['\r']) =>
                    $(&opt.fname): Optional::Omitted,
                )
            };
        )
        $to_attributes_tokens
        $try_from_attributes_tokens
    }
}

fn emit_cluster_wire_conversions(cluster_struct_name: &str) -> Tokens {
    quote! {
        crate::impl_cluster_wire_conversions!($cluster_struct_name);
    }
}
