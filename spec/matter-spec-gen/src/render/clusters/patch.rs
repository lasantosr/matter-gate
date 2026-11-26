//! Sparse delta `<Name>ClusterPatch` struct and `PartialCluster` / `ClusterSpec` trait implementations.

use genco::{quote, tokens::quoted};

use crate::render::{
    context::Tokens,
    record::{MemberPartition, StructAttrs},
};

/// Emits `<Name>ClusterPatch`, `PartialCluster`, `TryFrom<Patch> for Cluster`, and `ClusterSpec`.
pub(super) fn emit_cluster_patch_tokens(
    cluster_struct_name: &str,
    patch_name: &str,
    patch_attrs: StructAttrs,
    partition: &MemberPartition<'_>,
    features_ty: &str,
    has_client_commands: bool,
    has_server_commands: bool,
    has_events: bool,
) -> Tokens {
    quote! {
        $patch_attrs
        pub struct $patch_name $(if partition.is_empty() { ; } else { {
            $(for attr in partition.all join ($['\r']) =>
                $(if let Some(with_type) = attr.schemars_with_type() =>
                    #[cfg_attr(feature = $(quoted("schemars")), schemars(with = $(quoted(with_type))))]
                )
                $(if attr.is_nullable =>
                    #[cfg_attr(
                        feature = $(quoted("serde")),
                        serde(
                            default,
                            skip_serializing_if = $(quoted("Optional::is_omitted")),
                            deserialize_with = $(quoted("crate::model::serde::deserialize_optional_nullable"))
                        )
                    )]
                    #[cfg_attr(all(not(feature = $(quoted("serde"))), feature = $(quoted("schemars"))), schemars(default))]
                )
                $(if !attr.is_nullable =>
                    #[cfg_attr(
                        feature = $(quoted("serde")),
                        serde(default, skip_serializing_if = $(quoted("Optional::is_omitted")))
                    )]
                    #[cfg_attr(all(not(feature = $(quoted("serde"))), feature = $(quoted("schemars"))), schemars(default))]
                )
                pub $(&attr.fname): Optional<$(&attr.inner_type)>,
            )
        } })

        $(if !partition.is_empty() =>
            impl $patch_name {
                /// Applies a single decoded [`Attribute`] to this patch.
                pub fn apply(&mut self, attr: Attribute) {
                    match attr {
                        $(for attr in partition.all join ($['\r']) =>
                            Attribute::$(&attr.name)(v) => {
                                self.$(&attr.fname) = Optional::Present(v.into());
                            }
                        )
                    }
                }
            }
        )

        impl crate::traits::PartialCluster for $patch_name {
            type Full = $cluster_struct_name;

            fn apply_attribute(
                &mut self,
                id: AttributeId,
                $(if partition.is_empty() => _value: AttributeValue)
                $(if !partition.is_empty() => value: AttributeValue)
            ) -> Result<(), AttributeError> {
                $(if !partition.is_empty() =>
                    let attr = Attribute::try_from(model::Attribute::new(id, value))?;
                    self.apply(attr);
                    Ok(())
                )
                $(if partition.is_empty() =>
                    Err(AttributeError::UnsupportedAttribute(id))
                )
            }

            fn merge(&mut self, $(if partition.is_empty() => _other: Self) $(if !partition.is_empty() => other: Self)) {
                $(for attr in partition.all join ($['\r']) =>
                    if let Optional::Present(v) = other.$(&attr.fname) {
                        self.$(&attr.fname) = Optional::Present(v);
                    }
                )
            }

            fn try_into_full(self) -> Result<Self::Full, ClusterError> {
                $(if partition.is_empty() =>
                    Ok($cluster_struct_name)
                )
                $(if !partition.is_empty() =>
                    Ok($cluster_struct_name {
                        $(for attr in &partition.mandatory join ($['\r']) =>
                            $(&attr.fname): match self.$(&attr.fname) {
                                Optional::Present(v) => v,
                                Optional::Omitted => return Err(ClusterError::MissingMandatoryAttribute(
                                    $cluster_struct_name::$(attr.attr_const_ident()),
                                )),
                            },
                        )
                        $(for attr in &partition.optional join ($['\r']) =>
                            $(&attr.fname): self.$(&attr.fname),
                        )
                    })
                )
            }

            fn into_attributes(self) -> Vec<(AttributeId, AttributeValue)> {
                $(if partition.is_empty() =>
                    Vec::new()
                )
                $(if !partition.is_empty() =>
                    let mut out = Vec::new();
                    $(for attr in partition.all join ($['\r']) =>
                        if let Optional::Present(v) = self.$(&attr.fname) {
                            out.push((
                                $cluster_struct_name::$(attr.attr_const_ident()),
                                attributes::$(&attr.name)::from(v).into_value(),
                            ));
                        }
                    )
                    out
                )
            }
        }

        impl TryFrom<$patch_name> for $cluster_struct_name {
            type Error = ClusterError;

            #[inline]
            fn try_from(patch: $patch_name) -> Result<Self, Self::Error> {
                crate::traits::PartialCluster::try_into_full(patch)
            }
        }

        impl crate::traits::ClusterSpec for $cluster_struct_name {
            const CLUSTER_ID: ClusterId = Self::CLUSTER_ID;
            const NAME: &'static str = Self::NAME;
            const CLUSTER_REVISION: u16 = Self::CLUSTER_REVISION;
            const IS_EMPTY: bool = $(if partition.is_empty() { true } else { false });
            type Features = $features_ty;
            type Patch = $patch_name;
            type Attribute = $(if !partition.is_empty() { Attribute } else { () });
            type Command = $(if has_client_commands { Command } else { () });
            type CommandResponse = $(if has_server_commands { CommandResponse } else { () });
            type Event = $(if has_events { Event } else { () });

            #[inline]
            fn apply_patch(&mut self, patch: Self::Patch) {
                Self::apply_patch(self, patch);
            }
        }
    }
}
