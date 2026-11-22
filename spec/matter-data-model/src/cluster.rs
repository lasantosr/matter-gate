//! Matter Cluster entity per CSA Matter Specification Section 7.7.

extern crate alloc;

use alloc::vec::Vec;
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use crate::{
    attribute::{Attribute, AttributeId},
    optional::Optional,
    value::AttributeValue,
};

define_matter_id! {
    /// Strongly-typed Matter Cluster identifier (`u32`).
    pub struct ClusterId(u32);
    display = hex_32_or_16;
    mei = true;
}

impl ClusterId {
    /// Descriptor Cluster ID (`0x001D`) per CSA Matter Core Specification §9.5.
    /// Mandatory on every endpoint to expose device types, server clusters, client clusters, and parts list.
    pub const DESCRIPTOR: Self = Self(0x001D);
    /// Wildcard Cluster ID (`0xFFFF_FFFF`) per CSA Matter Interaction Model §10.5.
    pub const WILDCARD: Self = Self(0xFFFF_FFFF);

    /// Return `true` if this is the Wildcard cluster (`0xFFFF_FFFF`).
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.0 == 0xFFFF_FFFF
    }

    /// Return `true` if this cluster ID lies within CSA Matter Core Spec §7.18.2.37 / §2.5.7 allocation ranges
    /// (standard `0x0000..=0x7FFF`, or valid manufacturer vendor with `sub_id` in `0xFC00..=0xFFFE`).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        if self.is_standard() {
            self.sub_id() <= 0x7FFF
        } else {
            self.vendor_id().is_valid() && matches!(self.sub_id(), 0xFC00..=0xFFFE)
        }
    }
}

define_matter_id! {
    /// Strongly-typed Matter `DataVersion` identifier (`u32`).
    pub struct DataVersion(u32);
    display = decimal;
    mei = false;
}

/// Stateful wire representation of a Matter Cluster instance per Section 7.7.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Cluster {
    /// Matter cluster identifier.
    pub id: ClusterId,
    /// Optional monotonic data version counter for change detection.
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub data_version: Optional<DataVersion>,
    /// Attributes belonging to this cluster instance.
    pub attributes: Vec<Attribute>,
}

impl Cluster {
    /// Construct a cluster with attributes and omitted data version.
    ///
    /// Callers should provide attributes with unique [`AttributeId`]s; lookups and updates
    /// operate on the first matching ID.
    #[must_use]
    pub const fn new(id: ClusterId, attributes: Vec<Attribute>) -> Self {
        Self {
            id,
            data_version: Optional::Omitted,
            attributes,
        }
    }

    /// Construct an empty cluster without attributes and omitted data version.
    #[must_use]
    pub const fn empty(id: ClusterId) -> Self {
        Self {
            id,
            data_version: Optional::Omitted,
            attributes: Vec::new(),
        }
    }

    /// Set or override the cluster identifier.
    #[must_use]
    pub const fn with_id(mut self, id: ClusterId) -> Self {
        self.id = id;
        self
    }

    /// Set the data version of this cluster.
    #[must_use]
    pub const fn with_data_version(mut self, version: DataVersion) -> Self {
        self.data_version = Optional::Present(version);
        self
    }

    /// Set or omit the optional data version of this cluster.
    #[must_use]
    pub fn with_optional_data_version(mut self, version: impl Into<Optional<DataVersion>>) -> Self {
        self.data_version = version.into();
        self
    }

    /// Add or update an attribute on this cluster.
    #[must_use]
    pub fn with_attribute(mut self, id: AttributeId, value: impl Into<AttributeValue>) -> Self {
        self.set_attribute(id, value);
        self
    }

    /// Add or update multiple attributes on this cluster.
    #[must_use]
    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = Attribute>) -> Self {
        for attr in attributes {
            self.set_attribute(attr.id, attr.value);
        }
        self
    }

    /// Return `true` if this cluster contains an attribute with the given ID.
    #[must_use]
    pub fn has_attribute(&self, id: AttributeId) -> bool {
        self.attributes.iter().any(|attr| attr.id == id)
    }

    /// Find an attribute's value by ID on this cluster.
    #[must_use]
    pub fn attribute(&self, id: AttributeId) -> Option<&AttributeValue> {
        self.attributes
            .iter()
            .find(|attr| attr.id == id)
            .map(|attr| &attr.value)
    }

    /// Return a mutable reference to an attribute's value by ID on this cluster.
    #[must_use]
    pub fn attribute_mut(&mut self, id: AttributeId) -> Option<&mut AttributeValue> {
        self.attributes
            .iter_mut()
            .find(|attr| attr.id == id)
            .map(|attr| &mut attr.value)
    }

    /// Insert or update an attribute on this cluster in place, returning the previous value if replaced.
    pub fn set_attribute(&mut self, id: AttributeId, value: impl Into<AttributeValue>) -> Option<AttributeValue> {
        let val = value.into();
        if let Some(existing) = self.attributes.iter_mut().find(|attr| attr.id == id) {
            Some(mem::replace(&mut existing.value, val))
        } else {
            self.attributes.push(Attribute::new(id, val));
            None
        }
    }

    /// Remove an attribute by ID from this cluster, returning its value if present.
    pub fn remove_attribute(&mut self, id: AttributeId) -> Option<AttributeValue> {
        self.attributes
            .iter()
            .position(|attr| attr.id == id)
            .map(|idx| self.attributes.remove(idx).value)
    }
}

impl From<(ClusterId, Vec<Attribute>)> for Cluster {
    fn from((id, attributes): (ClusterId, Vec<Attribute>)) -> Self {
        Self {
            id,
            data_version: Optional::Omitted,
            attributes,
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Cluster {} ({} attributes)", self.id, self.attributes.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VendorId;

    #[test]
    fn cluster_id_and_data_version() {
        assert_eq!(ClusterId::WILDCARD.raw(), 0xFFFF_FFFF);

        let cid = ClusterId::new(0x0006);
        assert_eq!(cid.raw(), 0x0006);
        assert_eq!(alloc::format!("{cid}"), "0x0006");
        assert_eq!(u32::from(cid), 0x0006);

        let dv = DataVersion::new(42);
        assert_eq!(dv.raw(), 42);
        assert_eq!(alloc::format!("{dv}"), "42");
        assert_eq!(u32::from(dv), 42);

        let val_cid = AttributeValue::from(cid);
        assert_eq!(ClusterId::try_from(&val_cid), Ok(cid));
        assert_eq!(ClusterId::try_from(val_cid), Ok(cid));

        let val_dv = AttributeValue::from(dv);
        assert_eq!(DataVersion::try_from(&val_dv), Ok(dv));
        assert_eq!(DataVersion::try_from(val_dv), Ok(dv));
    }

    #[test]
    fn cluster_construction_and_lookup() {
        const ON_OFF_CLUSTER_ID: ClusterId = ClusterId::new(0x0006);

        let cluster = Cluster::empty(ON_OFF_CLUSTER_ID)
            .with_data_version(DataVersion::new(1))
            .with_attribute(AttributeId::new(0), true);

        assert_eq!(cluster.id, ON_OFF_CLUSTER_ID);
        assert_eq!(cluster.data_version, Optional::Present(DataVersion::new(1)));
        assert_eq!(
            cluster.attribute(AttributeId::new(0)),
            Some(&AttributeValue::Bool(true))
        );
        assert_eq!(cluster.attribute(AttributeId::new(1)), None);

        // Update attribute
        let mut cluster = cluster.with_attribute(AttributeId::new(0), false);
        assert_eq!(
            cluster.attribute(AttributeId::new(0)),
            Some(&AttributeValue::Bool(false))
        );
        assert!(cluster.has_attribute(AttributeId::new(0)));

        // In-place mutation
        let prev = cluster.set_attribute(AttributeId::new(0), true);
        assert_eq!(prev, Some(AttributeValue::Bool(false)));
        if let Some(val) = cluster.attribute_mut(AttributeId::new(0)) {
            *val = AttributeValue::U8(99);
        }
        assert_eq!(cluster.attribute(AttributeId::new(0)), Some(&AttributeValue::U8(99)));
        assert_eq!(
            cluster.remove_attribute(AttributeId::new(0)),
            Some(AttributeValue::U8(99))
        );
        assert!(!cluster.has_attribute(AttributeId::new(0)));
    }

    #[test]
    fn mei_helpers() {
        use crate::types::VendorId;
        let mei = ClusterId::from_mei(VendorId::new(0xFFF1), 0x1234);
        assert_eq!(mei.vendor_id(), VendorId::new(0xFFF1));
        assert_eq!(mei.sub_id(), 0x1234);
        assert!(mei.is_manufacturer_specific());
        assert!(!mei.is_standard());

        let std_id = ClusterId::DESCRIPTOR;
        assert_eq!(std_id.vendor_id(), VendorId::new(0));
        assert!(std_id.is_standard());
        assert!(!std_id.is_manufacturer_specific());
        assert!(!std_id.is_wildcard());

        assert!(ClusterId::WILDCARD.is_wildcard());
        assert!(!ClusterId::WILDCARD.is_standard());
        assert!(!ClusterId::WILDCARD.is_manufacturer_specific());
    }

    #[test]
    fn remove_attribute_preserves_order_and_formatting() {
        let mut cluster = Cluster::empty(ClusterId::new(0x0006))
            .with_optional_data_version(Some(DataVersion::new(5)))
            .with_attribute(AttributeId::new(1), true)
            .with_attribute(AttributeId::new(2), 2_u8)
            .with_attribute(AttributeId::new(3), 3_u8);
        assert_eq!(cluster.data_version, Optional::Present(DataVersion::new(5)));
        assert_eq!(alloc::format!("{cluster}"), "Cluster 0x0006 (3 attributes)");

        assert_eq!(
            cluster.remove_attribute(AttributeId::new(2)),
            Some(AttributeValue::U8(2))
        );
        let ids: Vec<u32> = cluster.attributes.iter().map(|a| a.id.raw()).collect();
        assert_eq!(ids, alloc::vec![1, 3]);

        assert_eq!("0x0006".parse::<ClusterId>(), Ok(ClusterId::new(0x0006)));
        assert_eq!("42".parse::<DataVersion>(), Ok(DataVersion::new(42)));
        assert_eq!(alloc::format!("{:04X}", ClusterId::new(0x001D)), "001D");

        let renamed = cluster.with_id(ClusterId::DESCRIPTOR);
        assert_eq!(renamed.id, ClusterId::DESCRIPTOR);
        assert!(ClusterId::DESCRIPTOR.is_valid());
        assert!(!ClusterId::new(0x8000).is_valid());
        assert!(ClusterId::from_mei(VendorId::new(0xFFF1), 0xFC00).is_valid());
        assert!(!ClusterId::from_mei(VendorId::new(0xFFF1), 0x0001).is_valid());
        assert!(!ClusterId::WILDCARD.is_valid());
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn cluster_json_schema_marks_data_version_optional() {
        let mut generator = schemars::SchemaGenerator::default();
        let schema = <Cluster as schemars::JsonSchema>::json_schema(&mut generator);
        let required = schema
            .as_object()
            .and_then(|o| o.get("required"))
            .and_then(|v| v.as_array());
        assert!(
            required.is_some_and(|req| !req.iter().any(|v| v.as_str() == Some("data_version"))),
            "data_version must not be required in Cluster JSON schema"
        );
    }
}
