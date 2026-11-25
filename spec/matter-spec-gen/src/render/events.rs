use std::result::Result as StdResult;

use genco::{quote, tokens::quoted};

use super::{
    context::{PushLine as _, Tokens, TypeContext, doc_comment, format_hex},
    record::{IdPrefix, MemberPartition, RecordConstructors, RecordStructDef, SingleMandatoryFrom, StructAttrs},
    structs::{emit_fields_decode_and_construct, emit_fields_into_vec, resolve_fields},
};
use crate::{
    error::Error,
    model::{ClusterDef, EventDef},
};

type Result<T, E = Error> = StdResult<T, E>;

pub(super) fn emit_events(
    out: &mut Tokens,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ctx: &TypeContext<'_>,
) -> Result<()> {
    if cluster.events.is_empty() {
        return Ok(());
    }

    let mut ev_tokens_list = Vec::new();
    for event_def in &cluster.events {
        ev_tokens_list.push(single_event_tokens(event_def, cluster, cluster_struct, ctx)?);
    }

    let module_tokens: Tokens = quote! {
        pub mod events {
            #[allow(unused_imports, reason = "Generated event imports")]
            use alloc::{string::String, vec::Vec};

            #[allow(unused_imports, reason = "Generated event imports")]
            use crate::{
                types,
                model::{
                    ActionId, AttributeId, AttributeValue, ClusterId, CommandDirection, CommandId,
                    DataVersion, DeviceTypeId, EndpointId, EventId, EventNumber, EventPriority,
                    FabricId, FabricIndex, FieldId, GroupId, MatterStatus, NodeId, Nullable, Optional,
                    TransactionId, VendorId,
                },
            };
            #[cfg(feature = $(quoted("serde")))]
            #[allow(unused_imports, reason = "Generated event imports")]
            use serde::{Deserialize, Serialize};

            $(for ev_tok in ev_tokens_list join ($['\n']) =>
                $ev_tok
            )
        }
    };

    out.append(module_tokens);
    out.push_blank();
    Ok(())
}

pub(super) fn single_event_tokens(
    event_def: &EventDef,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ctx: &TypeContext<'_>,
) -> Result<Tokens> {
    let EventDef {
        name,
        code,
        priority,
        fields,
        description,
        ..
    } = event_def;
    let priority_tok = match priority.to_ascii_lowercase().as_str() {
        "critical" => quote!(EventPriority::Critical),
        "info" => quote!(EventPriority::Info),
        "debug" => quote!(EventPriority::Debug),
        other => {
            return Err(Error::UnknownEventPriority {
                event: name.clone(),
                priority: other.to_owned(),
            });
        }
    };

    let ev_name = name;
    let ev_id_hex = format_hex(u64::from(*code), 2);

    if fields.is_empty() {
        return Ok(emit_unit_event_tokens(
            ev_name,
            description,
            cluster_struct,
            &ev_id_hex,
            &priority_tok,
        ));
    }

    emit_struct_event_tokens(event_def, cluster, cluster_struct, &ev_id_hex, &priority_tok, ctx)
}

pub(super) fn emit_event_enum_tokens(cluster: &ClusterDef, cluster_struct_name: &str, ctx: &TypeContext<'_>) -> Tokens {
    let can_derive_eq_hash = cluster.events.iter().filter(|ev| !ev.fields.is_empty()).all(|ev| {
        resolve_fields(&cluster.name, &ev.fields, true, ctx)
            .is_ok_and(|fields| MemberPartition::new(&fields).can_derive_eq_hash(ctx))
    });
    let has_multiple = cluster.events.len() > 1;
    let empty_evs: Vec<_> = cluster.events.iter().filter(|e| e.fields.is_empty()).collect();
    let non_empty_evs: Vec<_> = cluster.events.iter().filter(|e| !e.fields.is_empty()).collect();
    let critical_evs: Vec<_> = cluster
        .events
        .iter()
        .filter(|e| e.priority.eq_ignore_ascii_case("critical"))
        .collect();
    let info_evs: Vec<_> = cluster
        .events
        .iter()
        .filter(|e| e.priority.eq_ignore_ascii_case("info"))
        .collect();
    let debug_evs: Vec<_> = cluster
        .events
        .iter()
        .filter(|e| e.priority.eq_ignore_ascii_case("debug"))
        .collect();

    quote! {
        $(format!("/// Dynamic event variant matching for [`{cluster_struct_name}`]."))
        #[derive(Debug, Clone, PartialEq $(if can_derive_eq_hash => , Eq, Hash))]
        #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize))]
        #[cfg_attr(
            feature = $(quoted("serde")),
            serde(tag = $(quoted("event")), content = $(quoted("payload")))
        )]
        #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema))]
        #[allow(clippy::large_enum_variant, reason = "Generated event enum")]
        pub enum Event {
            $(for ev in &cluster.events join ($['\r']) =>
                $(if ev.fields.is_empty() =>
                    $(&ev.name),
                )
                $(if !ev.fields.is_empty() =>
                    $(&ev.name)(events::$(&ev.name)),
                )
            )
        }

        impl Event {
            pub const CLUSTER_ID: ClusterId = $cluster_struct_name::CLUSTER_ID;

            #[inline]
            #[must_use]
            pub const fn id(&self) -> EventId {
                match self {
                    $(for ev in &cluster.events join ($['\r']) =>
                        $(if ev.fields.is_empty() =>
                            Self::$(&ev.name) => events::$(&ev.name)::EVENT_ID,
                        )
                        $(if !ev.fields.is_empty() =>
                            Self::$(&ev.name)(_) => events::$(&ev.name)::EVENT_ID,
                        )
                    )
                }
            }

            #[inline]
            #[must_use]
            pub const fn priority(&self) -> EventPriority {
                match self {
                    $(if !critical_evs.is_empty() =>
                        $(for ev in &critical_evs join ( | ) =>
                            $(if ev.fields.is_empty() => Self::$(&ev.name))
                            $(if !ev.fields.is_empty() => Self::$(&ev.name)(_))
                        ) => EventPriority::Critical,
                    )
                    $(if !info_evs.is_empty() =>
                        $(for ev in &info_evs join ( | ) =>
                            $(if ev.fields.is_empty() => Self::$(&ev.name))
                            $(if !ev.fields.is_empty() => Self::$(&ev.name)(_))
                        ) => EventPriority::Info,
                    )
                    $(if !debug_evs.is_empty() =>
                        $(for ev in &debug_evs join ( | ) =>
                            $(if ev.fields.is_empty() => Self::$(&ev.name))
                            $(if !ev.fields.is_empty() => Self::$(&ev.name)(_))
                        ) => EventPriority::Debug,
                    )
                }
            }

            #[inline]
            #[must_use]
            pub const fn is_empty_payload(&self) -> bool {
                match self {
                    $(if !empty_evs.is_empty() =>
                        $(for ev in &empty_evs join ( | ) => Self::$(&ev.name)) => true,
                    )
                    $(if !non_empty_evs.is_empty() =>
                        $(for ev in &non_empty_evs join ( | ) => Self::$(&ev.name)(_)) => false,
                    )
                }
            }

            #[inline]
            pub fn from_model(ev: model::Event) -> Result<Self, EventError> {
                Self::try_from(ev)
            }

            $(emit_event_enum_select_described(cluster))
        }

        $(emit_event_enum_wire_impls(cluster, has_multiple))
        $(emit_event_enum_dispatch_impls(cluster, cluster_struct_name))
    }
}

fn emit_event_dispatch_impls(ev_name: &str, cluster_struct: &str, is_empty: bool) -> Tokens {
    let is_empty_str = if is_empty { "true" } else { "false" };
    quote! {
        crate::impl_matter_event!($ev_name, super::$cluster_struct, $is_empty_str);
    }
}

fn emit_unit_event_tokens(
    ev_name: &str,
    description: &str,
    cluster_struct: &str,
    ev_id_hex: &str,
    priority: &Tokens,
) -> Tokens {
    quote! {
        $(doc_comment(description))
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize))]
        #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema))]
        pub struct $ev_name;

        impl $ev_name {
            pub const CLUSTER_ID: ClusterId = super::$cluster_struct::CLUSTER_ID;
            pub const EVENT_ID: EventId = EventId::new($ev_id_hex);
            pub const NAME: &'static str = $(quoted(ev_name));
            pub const PRIORITY: EventPriority = $priority;

            #[inline]
            #[must_use]
            pub fn into_fields(self) -> Vec<(FieldId, AttributeValue)> {
                Vec::new()
            }

            #[inline]
            #[must_use]
            pub fn into_value(self) -> AttributeValue {
                AttributeValue::Struct(self.into_fields())
            }

            #[inline]
            pub(crate) fn decode_fields(
                fields: Vec<(FieldId, AttributeValue)>,
            ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
                if let Some((fid, _)) = fields.first() {
                    Err((Some(*fid), MatterStatus::ConstraintError))
                } else {
                    Ok(Self)
                }
            }

            #[inline]
            pub fn try_from_fields(fields: Vec<(FieldId, AttributeValue)>) -> Result<Self, MatterStatus> {
                Self::decode_fields(fields).map_err(|(_, status)| status)
            }

            #[inline]
            pub fn try_from_value(val: AttributeValue) -> Result<Self, MatterStatus> {
                match val {
                    AttributeValue::Null => Ok(Self),
                    AttributeValue::Struct(fields) => Self::try_from_fields(fields),
                    _ => Err(MatterStatus::InvalidDataType),
                }
            }
        }

        $(emit_event_dispatch_impls(ev_name, cluster_struct, true))
    }
}

fn emit_struct_event_tokens(
    event_def: &EventDef,
    cluster: &ClusterDef,
    cluster_struct: &str,
    ev_id_hex: &str,
    priority: &Tokens,
    ctx: &TypeContext<'_>,
) -> Result<Tokens> {
    let EventDef {
        name: ev_name,
        fields,
        description,
        ..
    } = event_def;

    let resolved_fields = resolve_fields(&cluster.name, fields, true, ctx)?;
    let partition = MemberPartition::new(&resolved_fields);
    let can_derive_eq_hash = partition.can_derive_eq_hash(ctx);

    let (decode_loop, construct_fields) =
        emit_fields_decode_and_construct(&resolved_fields, true, "MatterStatus::ConstraintError");
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
            name: ev_name,
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
    let single_mand_from = quote!($(SingleMandatoryFrom::maybe(ev_name, &partition.mandatory)));

    let short_field_count = partition.short_field_count();
    let many_single_char_reason = "Specification event fields contain multiple CSA single-character names";

    Ok(quote! {
        $struct_def

        impl $ev_name {
            pub const CLUSTER_ID: ClusterId = super::$cluster_struct::CLUSTER_ID;
            pub const EVENT_ID: EventId = EventId::new($ev_id_hex);
            pub const NAME: &'static str = $(quoted(ev_name));
            pub const PRIORITY: EventPriority = $priority;

            $methods_tokens

            #[must_use]
            pub fn into_fields(self) -> Vec<(FieldId, AttributeValue)> {
                $encode_vec_body
            }

            #[must_use]
            pub fn into_value(self) -> AttributeValue {
                AttributeValue::Struct(self.into_fields())
            }

            $(if short_field_count >= 5 =>
                #[expect(clippy::many_single_char_names, reason = $(quoted(many_single_char_reason)))]
            )
            pub(crate) fn decode_fields(
                fields: Vec<(FieldId, AttributeValue)>,
            ) -> Result<Self, (Option<FieldId>, MatterStatus)> {
                $(for field in partition.all join ($['\r']) =>
                    let mut $(&field.fname) = None;
                )
                $decode_loop
                Ok(Self {
                    $construct_fields
                })
            }

            #[inline]
            pub fn try_from_fields(fields: Vec<(FieldId, AttributeValue)>) -> Result<Self, MatterStatus> {
                Self::decode_fields(fields).map_err(|(_, status)| status)
            }

            #[inline]
            pub fn try_from_value(val: AttributeValue) -> Result<Self, MatterStatus> {
                let AttributeValue::Struct(fields) = val else {
                    return Err(MatterStatus::InvalidDataType);
                };
                Self::try_from_fields(fields)
            }
        }

        $single_mand_from

        $(emit_event_dispatch_impls(ev_name, cluster_struct, partition.is_empty()))
    })
}

fn emit_event_enum_select_described(cluster: &ClusterDef) -> Tokens {
    quote! {
        #[cfg(all(feature = $(quoted("dispatch")), feature = $(quoted("serde"))))]
        const DESCRIBED_VARIANTS: &'static [DescribedVariant<Self>] = &[
            $(for ev in &cluster.events join ($['\r']) =>
                (
                    $(quoted(&ev.name)),
                    events::$(&ev.name)::EVENT_ID.raw(),
                    $(if ev.fields.is_empty() =>
                        |payload| payload.deserialize_into::<events::$(&ev.name)>().map(|_| Self::$(&ev.name)),
                    )
                    $(if !ev.fields.is_empty() =>
                        |payload| payload.deserialize_into::<events::$(&ev.name)>().map(Self::$(&ev.name)),
                    )
                ),
            )
        ];

        crate::impl_select_described!(ev, "event");
    }
}

fn emit_event_enum_wire_impls(cluster: &ClusterDef, has_multiple: bool) -> Tokens {
    quote! {
        impl From<Event> for model::Event {
            fn from(ev: Event) -> Self {
                match ev {
                    $(for ev in &cluster.events join ($['\r']) =>
                        $(if ev.fields.is_empty() =>
                            Event::$(&ev.name) => model::Event::from(events::$(&ev.name)),
                        )
                        $(if !ev.fields.is_empty() =>
                            Event::$(&ev.name)(inner) => model::Event::from(inner),
                        )
                    )
                }
            }
        }

        impl TryFrom<model::Event> for Event {
            type Error = EventError;

            fn try_from(ev: model::Event) -> Result<Self, Self::Error> {
                if ev.cluster_id != Self::CLUSTER_ID {
                    return Err(EventError::ClusterIdMismatch {
                        expected: Self::CLUSTER_ID,
                        actual: ev.cluster_id,
                    });
                }
                match ev.event_id {
                    $(for ev in &cluster.events join ($['\r']) =>
                        events::$(&ev.name)::EVENT_ID => {
                            if ev.priority != events::$(&ev.name)::PRIORITY {
                                return Err(EventError::PriorityMismatch {
                                    expected: events::$(&ev.name)::PRIORITY,
                                    actual: ev.priority,
                                });
                            }
                            events::$(&ev.name)::decode_fields(ev.payload)
                                $(if ev.fields.is_empty() => .map(|_| Self::$(&ev.name)))
                                $(if !ev.fields.is_empty() => .map(Self::$(&ev.name)))
                                .map_err(|(field_id, status)| EventError::InvalidPayload {
                                    id: ev.event_id,
                                    field_id,
                                    status,
                                })
                        }
                    )
                    other => Err(EventError::UnsupportedEvent(other)),
                }
            }
        }

        $(if has_multiple =>
            $(for ev in &cluster.events join ($['\r']) =>
                $(if ev.fields.is_empty() =>
                    crate::impl_cluster_member_variant!(event_unit, Event, events::$(&ev.name), $(&ev.name));
                )
                $(if !ev.fields.is_empty() =>
                    crate::impl_cluster_member_variant!(event, Event, events::$(&ev.name), $(&ev.name));
                )
            )
        )
        $(if !has_multiple =>
            $(for ev in &cluster.events join ($['\n']) =>
                impl From<events::$(&ev.name)> for Event {
                    #[inline]
                    fn from(
                        $(if ev.fields.is_empty() => _)$(if !ev.fields.is_empty() => ev): events::$(&ev.name),
                    ) -> Self {
                        $(if ev.fields.is_empty() =>
                            Self::$(&ev.name)
                        )
                        $(if !ev.fields.is_empty() =>
                            Self::$(&ev.name)(ev)
                        )
                    }
                }

                impl TryFrom<Event> for events::$(&ev.name) {
                    type Error = EventError;

                    #[inline]
                    fn try_from(ev: Event) -> Result<Self, Self::Error> {
                        match ev {
                            $(if ev.fields.is_empty() =>
                                Event::$(&ev.name) => Ok(Self),
                            )
                            $(if !ev.fields.is_empty() =>
                                Event::$(&ev.name)(inner) => Ok(inner),
                            )
                        }
                    }
                }
            )
        )
    }
}

fn emit_event_enum_dispatch_impls(cluster: &ClusterDef, cluster_struct_name: &str) -> Tokens {
    let empty_evs: Vec<_> = cluster.events.iter().filter(|e| e.fields.is_empty()).collect();
    let non_empty_evs: Vec<_> = cluster.events.iter().filter(|e| !e.fields.is_empty()).collect();
    quote! {
        #[cfg(feature = $(quoted("dispatch")))]
        impl EventMeta for Event {
            #[inline]
            fn cluster_id(&self) -> ClusterId {
                Self::CLUSTER_ID
            }

            #[inline]
            fn cluster_name(&self) -> &'static str {
                $cluster_struct_name::NAME
            }

            #[inline]
            fn event_id(&self) -> EventId {
                self.id()
            }

            #[inline]
            fn event_name(&self) -> &'static str {
                match self {
                    $(for ev in &cluster.events join ($['\r']) =>
                        $(if ev.fields.is_empty() =>
                            Self::$(&ev.name) => events::$(&ev.name)::NAME,
                        )
                        $(if !ev.fields.is_empty() =>
                            Self::$(&ev.name)(_) => events::$(&ev.name)::NAME,
                        )
                    )
                }
            }

            #[inline]
            fn priority(&self) -> EventPriority {
                self.priority()
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                self.is_empty_payload()
            }
        }

        #[cfg(feature = $(quoted("dispatch")))]
        impl MatterDescribe for Event {
            const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema::cluster_enum(
                EntityKind::Event,
                $cluster_struct_name::NAME,
                $cluster_struct_name::CLUSTER_ID.raw(),
            );

            #[inline]
            fn descriptor(&self) -> EntityDescriptor {
                EntityDescriptor::Event {
                    cluster: $cluster_struct_name::NAME,
                    cluster_id: $cluster_struct_name::CLUSTER_ID.raw(),
                    event: self.event_name(),
                    event_id: self.id().raw(),
                }
            }

            #[inline]
            fn is_empty_payload(&self) -> bool {
                Self::is_empty_payload(self)
            }

            #[cfg(feature = $(quoted("serde")))]
            fn serialize_payload<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $(if !empty_evs.is_empty() =>
                        $(for ev in &empty_evs join ( | ) => Self::$(&ev.name)) => {
                            Serialize::serialize(&(), serializer)
                        }
                    )
                    $(for ev in &non_empty_evs join ($['\r']) =>
                        Self::$(&ev.name)(inner) => Serialize::serialize(inner, serializer),
                    )
                }
            }

            #[cfg(feature = $(quoted("serde")))]
            fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserialize_enum_described(
                    deserializer,
                    EntityKind::Event,
                    Some(($cluster_struct_name::NAME, $cluster_struct_name::CLUSTER_ID.raw())),
                    |_, _, item_name, item_id, payload| Self::select_described(item_name, item_id, payload),
                )
            }
        }
    }
}
