//! Matter Cluster entity per CSA Matter Specification Section 7.7.

extern crate alloc;

use alloc::vec::Vec;
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use crate::{
    attribute::{Attribute, AttributeId},
    command::CommandId,
    event::EventId,
    iter::ResultListIter,
    optional::Optional,
    status::MatterStatus,
    value::{AttributeValue, decode_unsigned},
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

    /// Set the `GeneratedCommandList` global attribute (`0xFFF8`, CSA §7.13), deduplicating IDs in first-seen order.
    #[must_use]
    pub fn with_generated_command_list(mut self, commands: impl IntoIterator<Item = CommandId>) -> Self {
        self.set_generated_command_list(commands);
        self
    }

    /// Set the `AcceptedCommandList` global attribute (`0xFFF9`, CSA §7.13), deduplicating IDs in first-seen order.
    #[must_use]
    pub fn with_accepted_command_list(mut self, commands: impl IntoIterator<Item = CommandId>) -> Self {
        self.set_accepted_command_list(commands);
        self
    }

    /// Set the `EventList` global attribute (`0xFFFA`, CSA §7.13, Matter `v1.1`–`v1.3`), deduplicating IDs in
    /// first-seen order.
    #[must_use]
    pub fn with_event_list(mut self, events: impl IntoIterator<Item = EventId>) -> Self {
        self.set_event_list(events);
        self
    }

    /// Set the `AttributeList` global attribute (`0xFFFB`, CSA §7.13), deduplicating IDs in first-seen order.
    #[must_use]
    pub fn with_attribute_list(mut self, attributes: impl IntoIterator<Item = AttributeId>) -> Self {
        self.set_attribute_list(attributes);
        self
    }

    /// Set the `FeatureMap` global attribute (`0xFFFC`, CSA §7.13).
    #[must_use]
    pub fn with_feature_map(mut self, feature_map: u32) -> Self {
        self.set_feature_map(feature_map);
        self
    }

    /// Set the `ClusterRevision` global attribute (`0xFFFD`, CSA §7.13).
    #[must_use]
    pub fn with_cluster_revision(mut self, revision: u16) -> Self {
        self.set_cluster_revision(revision);
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

    /// Return `true` if `GeneratedCommandList` (`0xFFF8`) is present on this cluster.
    #[must_use]
    pub fn has_generated_command_list(&self) -> bool {
        self.has_attribute(AttributeId::GENERATED_COMMAND_LIST)
    }

    /// Read `GeneratedCommandList` (`0xFFF8`, CSA §7.13), returning `Optional::Omitted` if absent.
    ///
    /// Returns a zero-allocation, single-pass [`ResultListIter`] over the wire list elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::InvalidDataType`] if the attribute is not [`AttributeValue::List`].
    pub fn generated_command_list(&self) -> Result<Optional<ResultListIter<'_, CommandId>>, MatterStatus> {
        self.read_id_list(AttributeId::GENERATED_COMMAND_LIST)
    }

    /// Set `GeneratedCommandList` (`0xFFF8`, CSA §7.13) in place, deduplicating IDs in first-seen order.
    pub fn set_generated_command_list(
        &mut self,
        commands: impl IntoIterator<Item = CommandId>,
    ) -> Option<AttributeValue> {
        self.set_id_list(AttributeId::GENERATED_COMMAND_LIST, commands)
    }

    /// Remove `GeneratedCommandList` (`0xFFF8`) from this cluster, returning its previous value if present.
    pub fn remove_generated_command_list(&mut self) -> Option<AttributeValue> {
        self.remove_attribute(AttributeId::GENERATED_COMMAND_LIST)
    }

    /// Return `true` if `AcceptedCommandList` (`0xFFF9`) is present on this cluster.
    #[must_use]
    pub fn has_accepted_command_list(&self) -> bool {
        self.has_attribute(AttributeId::ACCEPTED_COMMAND_LIST)
    }

    /// Read `AcceptedCommandList` (`0xFFF9`, CSA §7.13), returning `Optional::Omitted` if absent.
    ///
    /// Returns a zero-allocation, single-pass [`ResultListIter`] over the wire list elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::InvalidDataType`] if the attribute is not [`AttributeValue::List`].
    pub fn accepted_command_list(&self) -> Result<Optional<ResultListIter<'_, CommandId>>, MatterStatus> {
        self.read_id_list(AttributeId::ACCEPTED_COMMAND_LIST)
    }

    /// Set `AcceptedCommandList` (`0xFFF9`, CSA §7.13) in place, deduplicating IDs in first-seen order.
    pub fn set_accepted_command_list(
        &mut self,
        commands: impl IntoIterator<Item = CommandId>,
    ) -> Option<AttributeValue> {
        self.set_id_list(AttributeId::ACCEPTED_COMMAND_LIST, commands)
    }

    /// Remove `AcceptedCommandList` (`0xFFF9`) from this cluster, returning its previous value if present.
    pub fn remove_accepted_command_list(&mut self) -> Option<AttributeValue> {
        self.remove_attribute(AttributeId::ACCEPTED_COMMAND_LIST)
    }

    /// Return `true` if `EventList` (`0xFFFA`) is present on this cluster.
    #[must_use]
    pub fn has_event_list(&self) -> bool {
        self.has_attribute(AttributeId::EVENT_LIST)
    }

    /// Read `EventList` (`0xFFFA`, CSA §7.13, Matter `v1.1`–`v1.3`), returning `Optional::Omitted` if absent.
    ///
    /// Returns a zero-allocation, single-pass [`ResultListIter`] over the wire list elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::InvalidDataType`] if the attribute is not [`AttributeValue::List`].
    pub fn event_list(&self) -> Result<Optional<ResultListIter<'_, EventId>>, MatterStatus> {
        self.read_id_list(AttributeId::EVENT_LIST)
    }

    /// Set `EventList` (`0xFFFA`, CSA §7.13, Matter `v1.1`–`v1.3`) in place, deduplicating IDs in first-seen order.
    pub fn set_event_list(&mut self, events: impl IntoIterator<Item = EventId>) -> Option<AttributeValue> {
        self.set_id_list(AttributeId::EVENT_LIST, events)
    }

    /// Remove `EventList` (`0xFFFA`) from this cluster, returning its previous value if present.
    pub fn remove_event_list(&mut self) -> Option<AttributeValue> {
        self.remove_attribute(AttributeId::EVENT_LIST)
    }

    /// Return `true` if `AttributeList` (`0xFFFB`) is present on this cluster.
    #[must_use]
    pub fn has_attribute_list(&self) -> bool {
        self.has_attribute(AttributeId::ATTRIBUTE_LIST)
    }

    /// Read `AttributeList` (`0xFFFB`, CSA §7.13), returning `Optional::Omitted` if absent.
    ///
    /// Returns a zero-allocation, single-pass [`ResultListIter`] over the wire list elements.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::InvalidDataType`] if the attribute is not [`AttributeValue::List`].
    pub fn attribute_list(&self) -> Result<Optional<ResultListIter<'_, AttributeId>>, MatterStatus> {
        self.read_id_list(AttributeId::ATTRIBUTE_LIST)
    }

    /// Set `AttributeList` (`0xFFFB`, CSA §7.13) in place, deduplicating IDs in first-seen order.
    pub fn set_attribute_list(&mut self, attributes: impl IntoIterator<Item = AttributeId>) -> Option<AttributeValue> {
        self.set_id_list(AttributeId::ATTRIBUTE_LIST, attributes)
    }

    /// Remove `AttributeList` (`0xFFFB`) from this cluster, returning its previous value if present.
    pub fn remove_attribute_list(&mut self) -> Option<AttributeValue> {
        self.remove_attribute(AttributeId::ATTRIBUTE_LIST)
    }

    /// Return `true` if `FeatureMap` (`0xFFFC`) is present on this cluster.
    #[must_use]
    pub fn has_feature_map(&self) -> bool {
        self.has_attribute(AttributeId::FEATURE_MAP)
    }

    /// Read `FeatureMap` (`0xFFFC`, CSA §7.13), returning `Optional::Omitted` if absent.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::InvalidDataType`] if the attribute is not an unsigned integer, or
    /// [`MatterStatus::ConstraintError`] if an unsigned integer overflows `u32`.
    pub fn feature_map(&self) -> Result<Optional<u32>, MatterStatus> {
        self.attribute(AttributeId::FEATURE_MAP)
            .map(decode_unsigned::<u32>)
            .transpose()
            .map(Optional::from)
    }

    /// Set `FeatureMap` (`0xFFFC`, CSA §7.13) in place, returning its previous value if replaced.
    pub fn set_feature_map(&mut self, feature_map: u32) -> Option<AttributeValue> {
        self.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(feature_map))
    }

    /// Remove `FeatureMap` (`0xFFFC`) from this cluster, returning its previous value if present.
    pub fn remove_feature_map(&mut self) -> Option<AttributeValue> {
        self.remove_attribute(AttributeId::FEATURE_MAP)
    }

    /// Return `true` if `ClusterRevision` (`0xFFFD`) is present on this cluster.
    #[must_use]
    pub fn has_cluster_revision(&self) -> bool {
        self.has_attribute(AttributeId::CLUSTER_REVISION)
    }

    /// Read `ClusterRevision` (`0xFFFD`, CSA §7.13), returning `Optional::Omitted` if absent.
    ///
    /// # Errors
    ///
    /// Returns [`MatterStatus::InvalidDataType`] if the attribute is not an unsigned integer, or
    /// [`MatterStatus::ConstraintError`] if an unsigned integer overflows `u16`.
    pub fn cluster_revision(&self) -> Result<Optional<u16>, MatterStatus> {
        self.attribute(AttributeId::CLUSTER_REVISION)
            .map(decode_unsigned::<u16>)
            .transpose()
            .map(Optional::from)
    }

    /// Set `ClusterRevision` (`0xFFFD`, CSA §7.13) in place, returning its previous value if replaced.
    pub fn set_cluster_revision(&mut self, revision: u16) -> Option<AttributeValue> {
        self.set_attribute(AttributeId::CLUSTER_REVISION, AttributeValue::U16(revision))
    }

    /// Remove `ClusterRevision` (`0xFFFD`) from this cluster, returning its previous value if present.
    pub fn remove_cluster_revision(&mut self) -> Option<AttributeValue> {
        self.remove_attribute(AttributeId::CLUSTER_REVISION)
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

impl Cluster {
    fn read_id_list<T>(&self, id: AttributeId) -> Result<Optional<ResultListIter<'_, T>>, MatterStatus> {
        match self.attribute(id) {
            None => Ok(Optional::Omitted),
            Some(AttributeValue::List(items)) => Ok(Optional::Present(ResultListIter::new(items))),
            Some(_) => Err(MatterStatus::InvalidDataType),
        }
    }

    fn set_id_list<T>(&mut self, id: AttributeId, items: impl IntoIterator<Item = T>) -> Option<AttributeValue>
    where
        T: Into<AttributeValue>,
    {
        let iter = items.into_iter();
        let mut values = Vec::with_capacity(iter.size_hint().0);
        for item in iter {
            let val = item.into();
            if !values.contains(&val) {
                values.push(val);
            }
        }
        self.set_attribute(id, AttributeValue::List(values))
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

    #[test]
    fn cluster_global_attributes_omitted_empty_and_populated() {
        let mut cluster = Cluster::empty(ClusterId::new(0x0006));
        assert!(!cluster.has_generated_command_list());
        assert!(!cluster.has_accepted_command_list());
        assert!(!cluster.has_event_list());
        assert!(!cluster.has_attribute_list());
        assert!(!cluster.has_feature_map());
        assert!(!cluster.has_cluster_revision());

        assert_eq!(
            cluster.generated_command_list().unwrap().to_vec(),
            Ok(Optional::Omitted)
        );
        assert_eq!(cluster.accepted_command_list().unwrap().to_vec(), Ok(Optional::Omitted));
        assert_eq!(cluster.event_list().unwrap().to_vec(), Ok(Optional::Omitted));
        assert_eq!(cluster.attribute_list().unwrap().to_vec(), Ok(Optional::Omitted));
        assert_eq!(cluster.feature_map(), Ok(Optional::Omitted));
        assert_eq!(cluster.cluster_revision(), Ok(Optional::Omitted));

        // Explicitly empty lists must be Present(empty), not Omitted.
        cluster = cluster
            .with_generated_command_list([])
            .with_accepted_command_list([])
            .with_event_list([])
            .with_attribute_list([]);
        assert!(cluster.has_generated_command_list());
        assert!(cluster.has_accepted_command_list());
        assert!(cluster.has_event_list());
        assert!(cluster.has_attribute_list());
        assert_eq!(
            cluster.generated_command_list().unwrap().to_vec(),
            Ok(Optional::Present(Vec::new()))
        );
        assert_eq!(
            cluster.accepted_command_list().unwrap().to_vec(),
            Ok(Optional::Present(Vec::new()))
        );
        assert_eq!(
            cluster.event_list().unwrap().to_vec(),
            Ok(Optional::Present(Vec::new()))
        );
        assert_eq!(
            cluster.attribute_list().unwrap().to_vec(),
            Ok(Optional::Present(Vec::new()))
        );

        // Builders with populated values + order-preserving deduplication.
        let cmd0 = CommandId::new(0x00);
        let cmd1 = CommandId::new(0x01);
        let cmd2 = CommandId::new(0x02);
        let evt1 = EventId::new(0x10);
        let evt2 = EventId::new(0x20);
        let attr0 = AttributeId::new(0x0000);
        let attr1 = AttributeId::new(0x0001);

        let populated = Cluster::empty(ClusterId::new(0x0006))
            .with_generated_command_list([cmd2, cmd0, cmd2, cmd1])
            .with_accepted_command_list([cmd1, cmd0, cmd1])
            .with_event_list([evt2, evt1, evt2])
            .with_attribute_list([attr1, attr0, attr1, AttributeId::CLUSTER_REVISION])
            .with_feature_map(0x0000_0003)
            .with_cluster_revision(6);

        assert_eq!(
            populated.generated_command_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![cmd2, cmd0, cmd1]))
        );
        assert_eq!(
            populated.accepted_command_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![cmd1, cmd0]))
        );
        assert_eq!(
            populated.event_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![evt2, evt1]))
        );
        assert_eq!(
            populated.attribute_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![
                attr1,
                attr0,
                AttributeId::CLUSTER_REVISION
            ]))
        );
        assert_eq!(populated.accepted_command_list().unwrap().contains(cmd0), Ok(true));
        assert_eq!(populated.accepted_command_list().unwrap().contains(cmd2), Ok(false));
        assert_eq!(populated.feature_map(), Ok(Optional::Present(3)));
        assert_eq!(populated.cluster_revision(), Ok(Optional::Present(6)));
    }

    #[test]
    fn cluster_global_attributes_set_remove_and_wire_read() {
        let cmd0 = CommandId::new(0);
        let cmd1 = CommandId::new(1);
        // Raw wire list with duplicates is preserved on read without deduplicating.
        let mut cluster = Cluster::new(
            ClusterId::new(0x0006),
            alloc::vec![Attribute::new(
                AttributeId::ACCEPTED_COMMAND_LIST,
                AttributeValue::List(alloc::vec![cmd1.into(), cmd0.into(), cmd1.into()]),
            )],
        );
        assert_eq!(
            cluster.accepted_command_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![cmd1, cmd0, cmd1]))
        );

        // In-place replacement returns previous value; verify DoubleEndedIterator & ExactSizeIterator on
        // non-palindromic list.
        let prev = cluster.set_accepted_command_list([cmd0, cmd1, cmd0]);
        assert_eq!(
            prev,
            Some(AttributeValue::List(alloc::vec![cmd1.into(), cmd0.into(), cmd1.into()]))
        );
        assert_eq!(
            cluster.accepted_command_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![cmd0, cmd1]))
        );
        let mut iter = cluster.accepted_command_list().unwrap().unwrap();
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next_back(), Some(Ok(cmd1)));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next_back(), Some(Ok(cmd0)));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next_back(), None);

        // Removal returns previous value and restores Omitted.
        let removed = cluster.remove_accepted_command_list();
        assert_eq!(
            removed,
            Some(AttributeValue::List(alloc::vec![cmd0.into(), cmd1.into()]))
        );
        assert!(!cluster.has_accepted_command_list());
        assert_eq!(cluster.accepted_command_list().unwrap().to_vec(), Ok(Optional::Omitted));

        // FeatureMap & ClusterRevision set/remove.
        assert_eq!(cluster.set_feature_map(7), None);
        assert_eq!(cluster.set_feature_map(9), Some(AttributeValue::U32(7)));
        assert_eq!(cluster.remove_feature_map(), Some(AttributeValue::U32(9)));
        assert!(!cluster.has_feature_map());

        assert_eq!(cluster.set_cluster_revision(4), None);
        assert_eq!(cluster.set_cluster_revision(5), Some(AttributeValue::U16(4)));
        assert_eq!(cluster.remove_cluster_revision(), Some(AttributeValue::U16(5)));
        assert!(!cluster.has_cluster_revision());
    }

    #[test]
    fn cluster_global_attributes_invalid_data_type_and_overflow() {
        let cluster = Cluster::new(
            ClusterId::new(0x0006),
            alloc::vec![
                Attribute::new(AttributeId::GENERATED_COMMAND_LIST, AttributeValue::I32(1)),
                Attribute::new(
                    AttributeId::ACCEPTED_COMMAND_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::I16(1)]),
                ),
                Attribute::new(AttributeId::EVENT_LIST, AttributeValue::Bool(true)),
                Attribute::new(
                    AttributeId::ATTRIBUTE_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U32(0), AttributeValue::Bool(false)]),
                ),
                Attribute::new(AttributeId::FEATURE_MAP, AttributeValue::I32(-1)),
                Attribute::new(AttributeId::CLUSTER_REVISION, AttributeValue::String("1".into())),
            ],
        );

        assert_eq!(
            cluster.generated_command_list().err(),
            Some(MatterStatus::InvalidDataType)
        );
        assert_eq!(
            cluster.accepted_command_list().unwrap().to_vec(),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(cluster.event_list().err(), Some(MatterStatus::InvalidDataType));
        assert_eq!(
            cluster.attribute_list().unwrap().to_vec(),
            Err(MatterStatus::InvalidDataType)
        );
        assert_eq!(cluster.feature_map(), Err(MatterStatus::InvalidDataType));
        assert_eq!(cluster.cluster_revision(), Err(MatterStatus::InvalidDataType));

        let overflow_cluster = Cluster::new(
            ClusterId::new(0x0006),
            alloc::vec![
                Attribute::new(
                    AttributeId::ACCEPTED_COMMAND_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U64(u64::from(u32::MAX) + 1)]),
                ),
                Attribute::new(AttributeId::FEATURE_MAP, AttributeValue::U64(u64::from(u32::MAX) + 1)),
                Attribute::new(AttributeId::CLUSTER_REVISION, AttributeValue::U32(70_000)),
            ],
        );

        assert_eq!(
            overflow_cluster.accepted_command_list().unwrap().to_vec(),
            Err(MatterStatus::ConstraintError)
        );
        assert_eq!(overflow_cluster.feature_map(), Err(MatterStatus::ConstraintError));
        assert_eq!(overflow_cluster.cluster_revision(), Err(MatterStatus::ConstraintError));
    }

    #[test]
    fn cluster_global_attributes_unsigned_width_flexibility() {
        let cluster = Cluster::new(
            ClusterId::new(0x0006),
            alloc::vec![
                Attribute::new(
                    AttributeId::GENERATED_COMMAND_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U8(1), AttributeValue::U64(2)]),
                ),
                Attribute::new(
                    AttributeId::ACCEPTED_COMMAND_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U16(3)]),
                ),
                Attribute::new(
                    AttributeId::EVENT_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U8(4)]),
                ),
                Attribute::new(
                    AttributeId::ATTRIBUTE_LIST,
                    AttributeValue::List(alloc::vec![AttributeValue::U16(5), AttributeValue::U64(6)]),
                ),
                Attribute::new(AttributeId::FEATURE_MAP, AttributeValue::U8(7)),
                Attribute::new(AttributeId::CLUSTER_REVISION, AttributeValue::U64(8)),
            ],
        );

        assert_eq!(
            cluster.generated_command_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![CommandId::new(1), CommandId::new(2)]))
        );
        assert_eq!(
            cluster.accepted_command_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![CommandId::new(3)]))
        );
        assert_eq!(
            cluster.event_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![EventId::new(4)]))
        );
        assert_eq!(
            cluster.attribute_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![AttributeId::new(5), AttributeId::new(6)]))
        );
        assert_eq!(cluster.feature_map(), Ok(Optional::Present(7)));
        assert_eq!(cluster.cluster_revision(), Ok(Optional::Present(8)));
    }
}
