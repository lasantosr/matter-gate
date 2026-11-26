//! Per-cluster dynamic `Attribute` enum and its wire and dispatch trait implementations.

use genco::{quote, tokens::quoted};

use crate::render::{context::Tokens, record::ResolvedMember};

/// Emits the cluster's `Attribute` enum, wire conversions, and `AttributeMeta` / `MatterDescribe` impls.
pub(super) fn emit_attribute_enum_tokens(
    cluster_struct_name: &str,
    resolved_attrs: &[ResolvedMember],
    can_derive_eq_hash: bool,
) -> Tokens {
    let (part1, part2) = resolved_attrs.split_at(resolved_attrs.len() >> 1);
    let has_multiple = resolved_attrs.len() > 1;
    quote! {
        $(format!("/// Dynamic attribute variant matching for [`{cluster_struct_name}`]."))
        #[derive(Debug, Clone, PartialEq $(if can_derive_eq_hash => , Eq, Hash))]
        #[cfg_attr(feature = $(quoted("serde")), derive(Serialize, Deserialize))]
        #[cfg_attr(
            feature = $(quoted("serde")),
            serde(tag = $(quoted("attribute")), content = $(quoted("payload")))
        )]
        #[cfg_attr(feature = $(quoted("schemars")), derive(schemars::JsonSchema))]
        pub enum Attribute {
            $(for attr in resolved_attrs join ($['\r']) =>
                $(&attr.name)(attributes::$(&attr.name)),
            )
        }

        impl Attribute {
            pub const CLUSTER_ID: ClusterId = $cluster_struct_name::CLUSTER_ID;

            #[inline]
            #[must_use]
            pub const fn id(&self) -> AttributeId {
                match self {
                    $(for attr in resolved_attrs join ($['\r']) =>
                        Self::$(&attr.name)(_) => $cluster_struct_name::$(attr.attr_const_ident()),
                    )
                }
            }

            #[inline]
            #[must_use]
            pub fn into_value(self) -> AttributeValue {
                match self {
                    $(for attr in resolved_attrs join ($['\r']) =>
                        Self::$(&attr.name)(attr) => attr.into_value(),
                    )
                }
            }

            #[inline]
            #[must_use]
            pub fn into_pair(self) -> (AttributeId, AttributeValue) {
                (self.id(), self.into_value())
            }

            #[inline]
            pub fn from_model(attr: model::Attribute) -> Result<Self, AttributeError> {
                Self::try_from(attr)
            }

            $(if let [single_attr] = resolved_attrs =>
                pub fn try_from_pair(id: AttributeId, val: AttributeValue) -> Result<Self, MatterStatus> {
                    if id == $cluster_struct_name::$(single_attr.attr_const_ident()) {
                        attributes::$(&single_attr.name)::try_from_value(val)
                            .map(Self::$(&single_attr.name))
                    } else {
                        Err(MatterStatus::UnsupportedAttribute)
                    }
                }
            )
            $(if resolved_attrs.len() > 1 && resolved_attrs.len() <= 30 =>
                pub fn try_from_pair(id: AttributeId, val: AttributeValue) -> Result<Self, MatterStatus> {
                    match id {
                        $(for attr in resolved_attrs join ($['\r']) =>
                            $cluster_struct_name::$(attr.attr_const_ident()) => {
                                attributes::$(&attr.name)::try_from_value(val).map(Self::$(&attr.name))
                            }
                        )
                        _ => Err(MatterStatus::UnsupportedAttribute),
                    }
                }
            )
            $(if resolved_attrs.len() > 30 =>
                fn try_from_pair_part1(id: AttributeId, val: AttributeValue) -> Result<Self, MatterStatus> {
                    match id {
                        $(for attr in part1 join ($['\r']) =>
                            $cluster_struct_name::$(attr.attr_const_ident()) => {
                                attributes::$(&attr.name)::try_from_value(val).map(Self::$(&attr.name))
                            }
                        )
                        _ => Err(MatterStatus::UnsupportedAttribute),
                    }
                }

                fn try_from_pair_part2(id: AttributeId, val: AttributeValue) -> Result<Self, MatterStatus> {
                    match id {
                        $(for attr in part2 join ($['\r']) =>
                            $cluster_struct_name::$(attr.attr_const_ident()) => {
                                attributes::$(&attr.name)::try_from_value(val).map(Self::$(&attr.name))
                            }
                        )
                        _ => Err(MatterStatus::UnsupportedAttribute),
                    }
                }

                pub fn try_from_pair(id: AttributeId, val: AttributeValue) -> Result<Self, MatterStatus> {
                    match id {
                        $(for attr in part1 join ( | ) => $cluster_struct_name::$(attr.attr_const_ident())) => {
                            Self::try_from_pair_part1(id, val)
                        }
                        _ => Self::try_from_pair_part2(id, val),
                    }
                }
            )

            #[cfg(all(feature = $(quoted("dispatch")), feature = $(quoted("serde"))))]
            const DESCRIBED_VARIANTS: &'static [DescribedVariant<Self>] = &[
                $(for attr in resolved_attrs join ($['\r']) =>
                    (
                        $(quoted(&attr.name)),
                        attributes::$(&attr.name)::ATTRIBUTE_ID.raw(),
                        |payload| {
                            payload
                                .deserialize_into::<attributes::$(&attr.name)>()
                                .map(Self::$(&attr.name))
                        },
                    ),
                )
            ];

            crate::impl_select_described!(attr, "attribute");
        }

        $(emit_attribute_enum_wire_impls(resolved_attrs, has_multiple))
        $(emit_attribute_enum_dispatch_impls(cluster_struct_name, resolved_attrs))
    }
}

fn emit_attribute_enum_wire_impls(resolved_attrs: &[ResolvedMember], has_multiple: bool) -> Tokens {
    quote! {
        impl From<Attribute> for model::Attribute {
            #[inline]
            fn from(attr: Attribute) -> Self {
                match attr {
                    $(for attr in resolved_attrs join ($['\r']) =>
                        Attribute::$(&attr.name)(inner) => model::Attribute::from(inner),
                    )
                }
            }
        }

        impl TryFrom<model::Attribute> for Attribute {
            type Error = AttributeError;

            #[inline]
            fn try_from(attr: model::Attribute) -> Result<Self, Self::Error> {
                Self::try_from_pair(attr.id, attr.value).map_err(|status| match status {
                    MatterStatus::UnsupportedAttribute => AttributeError::UnsupportedAttribute(attr.id),
                    status => AttributeError::InvalidValue {
                        id: attr.id,
                        field_id: None,
                        status,
                    },
                })
            }
        }

        impl From<Attribute> for (AttributeId, AttributeValue) {
            #[inline]
            fn from(attr: Attribute) -> Self {
                attr.into_pair()
            }
        }

        $(if has_multiple =>
            $(for attr in resolved_attrs join ($['\r']) =>
                crate::impl_cluster_member_variant!(attribute, Attribute, attributes::$(&attr.name), $(&attr.name));
            )
        )
        $(if !has_multiple =>
            $(for attr in resolved_attrs join ($['\r']) =>
                crate::impl_cluster_member_variant!(
                    attribute_single,
                    Attribute,
                    attributes::$(&attr.name),
                    $(&attr.name)
                );
            )
        )
    }
}

fn emit_attribute_enum_dispatch_impls(cluster_struct_name: &str, resolved_attrs: &[ResolvedMember]) -> Tokens {
    quote! {
        crate::impl_cluster_enum_dispatch!(
            attribute,
            Attribute,
            $cluster_struct_name,
            [
                $(for attr in resolved_attrs join ($['\r']) =>
                    $(&attr.name),
                )
            ]
        );
    }
}
