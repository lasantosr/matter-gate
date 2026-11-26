use alloc::vec::Vec;
use core::{
    mem,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(all(feature = "dispatch", feature = "serde"))]
use serde::{Deserializer, de::DeserializeOwned};

use super::global_attributes::GlobalAttributes;
#[cfg(all(feature = "dispatch", feature = "serde"))]
use crate::dispatch::deserialize_concrete_described;
#[cfg(feature = "dispatch")]
use crate::dispatch::{EntityDescriptor, MatterDescribe, StaticEntitySchema};
use crate::{
    error::{AttributeError, ClusterError},
    model::{
        Attribute as WireAttribute, AttributeId, AttributeValue, Cluster as ModelCluster, DataVersion, MatterStatus,
        Optional,
    },
};

/// Thin wire-metadata envelope around a cluster domain model `T`.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(deserialize = "T: Deserialize<'de>, F: Deserialize<'de> + Default"))
)]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "schemars", schemars(bound = "T: JsonSchema, F: JsonSchema + Default"))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub struct WireCluster<T, F = u32> {
    /// Instance-level `DataVersion`.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub data_version: Optional<DataVersion>,
    /// Global attributes (`0xFFF8..=0xFFFD`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub globals: GlobalAttributes<F>,
    /// Typed cluster domain state or patch.
    pub domain: T,
    /// Manufacturer-specific extension attributes (valid MEI IDs).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub extra_attributes: Vec<WireAttribute>,
}

impl<T, F> WireCluster<T, F> {
    /// Wraps `domain` in an empty [`WireCluster`].
    #[must_use]
    pub const fn new(domain: T) -> Self {
        Self {
            data_version: Optional::Omitted,
            globals: GlobalAttributes::empty(),
            domain,
            extra_attributes: Vec::new(),
        }
    }

    /// Sets `data_version`.
    #[must_use]
    pub fn with_data_version(mut self, data_version: DataVersion) -> Self {
        self.data_version = Optional::Present(data_version);
        self
    }

    /// Sets `globals`.
    #[must_use]
    pub fn with_globals(mut self, globals: GlobalAttributes<F>) -> Self {
        self.globals = globals;
        self
    }

    /// Sets `globals.feature_map`.
    #[must_use]
    pub fn with_feature_map(mut self, feature_map: F) -> Self {
        self.globals.feature_map = Optional::Present(feature_map);
        self
    }

    /// Appends a manufacturer-specific attribute to `extra_attributes`.
    #[must_use]
    pub fn with_extra_attribute(mut self, attr: WireAttribute) -> Self {
        self.extra_attributes.push(attr);
        self
    }

    /// Returns a shared reference to the inner domain value.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.domain
    }

    /// Returns a mutable reference to the inner domain value.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.domain
    }

    /// Consumes the wrapper and returns the inner domain value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.domain
    }

    /// Routes an attribute ID that was not handled by the domain cluster.
    ///
    /// - Global attributes (`0xFFF8..=0xFFFD`) are applied to `self.globals`.
    /// - Valid MEI attributes (`id.is_manufacturer_specific() && id.is_valid()`) are stored in `self.extra_attributes`.
    /// - Unknown standard attributes (`0x0000..=0x4FFF`), reserved vendor prefixes (`0xFFF5..=0xFFFF`), and unallocated
    ///   ranges (`0x5000..=0xEFFF`, `0xF000..=0xFFF7`, `0xFFFE`) fail fast with [`MatterStatus::UnsupportedAttribute`].
    pub fn route_unknown_attribute(&mut self, id: AttributeId, value: AttributeValue) -> Result<(), MatterStatus>
    where
        F: TryFrom<u32>,
    {
        if GlobalAttributes::<F>::is_global_id(id) {
            let _applied: bool = self.globals.apply_global_attribute(id, value)?;
            return Ok(());
        }
        if id.is_manufacturer_specific() && id.is_valid() {
            self.extra_attributes.push(WireAttribute::new(id, value));
            return Ok(());
        }
        Err(MatterStatus::UnsupportedAttribute)
    }
}

impl<T, F> Deref for WireCluster<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.domain
    }
}

impl<T, F> DerefMut for WireCluster<T, F> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.domain
    }
}

impl<T, F> From<WireCluster<T, F>> for ModelCluster
where
    T: Into<ModelCluster>,
    F: Into<u32>,
{
    fn from(wire: WireCluster<T, F>) -> Self {
        let mut cluster: Self = wire.domain.into();
        if wire.data_version.is_present() {
            cluster.data_version = wire.data_version;
        }
        let mut global_attrs = Vec::new();
        wire.globals.into_attributes(&mut global_attrs);
        for (id, value) in global_attrs {
            if !cluster.attributes.iter().any(|attr| attr.id == id) {
                cluster.attributes.push(WireAttribute::new(id, value));
            }
        }
        for attr in wire.extra_attributes {
            if !cluster.attributes.iter().any(|existing| existing.id == attr.id) {
                cluster.attributes.push(attr);
            }
        }
        cluster
    }
}

impl<T, F> TryFrom<ModelCluster> for WireCluster<T, F>
where
    T: TryFrom<ModelCluster, Error = ClusterError>,
    F: TryFrom<u32>,
{
    type Error = ClusterError;

    fn try_from(mut cluster: ModelCluster) -> Result<Self, Self::Error> {
        let data_version = cluster.data_version;
        let mut globals = GlobalAttributes::<F>::empty();
        let mut extra_attributes = Vec::new();
        let mut standard_attributes = Vec::with_capacity(cluster.attributes.len());

        for attr in mem::take(&mut cluster.attributes) {
            if GlobalAttributes::<F>::is_global_id(attr.id) {
                globals.apply_global_attribute(attr.id, attr.value).map_err(|status| {
                    ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                        id: attr.id,
                        field_id: None,
                        status,
                    })
                })?;
            } else if attr.id.is_manufacturer_specific() && attr.id.is_valid() {
                extra_attributes.push(attr);
            } else {
                standard_attributes.push(attr);
            }
        }

        cluster.attributes = standard_attributes;
        let domain = T::try_from(cluster)?;
        Ok(Self {
            data_version,
            globals,
            domain,
            extra_attributes,
        })
    }
}

#[cfg(all(feature = "dispatch", not(feature = "serde")))]
impl<T, F> MatterDescribe for WireCluster<T, F>
where
    T: MatterDescribe,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.domain.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }
}

#[cfg(all(feature = "dispatch", feature = "serde"))]
impl<T, F> MatterDescribe for WireCluster<T, F>
where
    T: MatterDescribe + DeserializeOwned,
    F: Default + DeserializeOwned,
{
    const STATIC_SCHEMA: StaticEntitySchema = StaticEntitySchema {
        is_empty: false,
        ..T::STATIC_SCHEMA
    };

    #[inline]
    fn descriptor(&self) -> EntityDescriptor {
        self.domain.descriptor()
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        false
    }

    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some((cluster, cluster_id)) = T::STATIC_SCHEMA.scope {
            deserialize_concrete_described(deserializer, EntityDescriptor::Cluster { cluster, cluster_id }, false)
        } else {
            super::deserialize_envelope_described(deserializer)
        }
    }
}
