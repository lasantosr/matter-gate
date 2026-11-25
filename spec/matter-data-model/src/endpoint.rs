//! Matter Endpoint and Device Type entities per CSA Matter Specification Sections 7.3, 7.4 & 9.5.

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use crate::{
    attribute::AttributeId,
    cluster::{Cluster, ClusterId},
    iter::ResultListIter,
    optional::Optional,
    status::MatterStatus,
    tag::SemanticTag,
    value::{AttributeValue, FieldId, decode_unsigned},
};

const DESCRIPTOR_DEVICE_TYPE_LIST: AttributeId = AttributeId::new(0x0000);
const DESCRIPTOR_SERVER_LIST: AttributeId = AttributeId::new(0x0001);
const DESCRIPTOR_CLIENT_LIST: AttributeId = AttributeId::new(0x0002);
const DESCRIPTOR_PARTS_LIST: AttributeId = AttributeId::new(0x0003);
const DESCRIPTOR_TAG_LIST: AttributeId = AttributeId::new(0x0004);
const DESCRIPTOR_ENDPOINT_UNIQUE_ID: AttributeId = AttributeId::new(0x0005);

define_matter_id! {
    /// Strongly-typed Matter Endpoint identifier (`u16`).
    pub struct EndpointId(u16);
    display = decimal;
    mei = false;
}

impl EndpointId {
    /// Root Node Endpoint ID (`0`) per CSA Matter Core Specification §7.3 & §9.1.
    /// Mandatory on every node; hosts node-wide operational and commissioning clusters.
    pub const ROOT: Self = Self(0);
    /// Wildcard Endpoint ID (`0xFFFF`) per CSA Matter Interaction Model §10.5.
    pub const WILDCARD: Self = Self(0xFFFF);

    /// Return `true` if this is the Root Node endpoint (`0`).
    #[must_use]
    pub const fn is_root(self) -> bool {
        self.0 == 0
    }

    /// Return `true` if this is the Wildcard endpoint (`0xFFFF`).
    #[must_use]
    pub const fn is_wildcard(self) -> bool {
        self.0 == 0xFFFF
    }

    /// Return `true` if this is a concrete (non-wildcard) endpoint ID (`!= 0xFFFF`).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != 0xFFFF
    }
}

define_matter_id! {
    /// Strongly-typed Matter Device Type identifier (`u32`).
    pub struct DeviceTypeId(u32);
    display = hex_32_or_16;
    mei = true;
}

impl DeviceTypeId {
    /// Return `true` if this device type ID lies within CSA Matter Core Specification §7.4 / §2.5.7
    /// allocation ranges (`sub_id` in `0x0000..=0xBFFF` with standard or valid manufacturer vendor prefix).
    #[must_use]
    pub const fn is_valid(self) -> bool {
        (self.is_standard() || self.vendor_id().is_valid()) && self.sub_id() <= 0xBFFF
    }
}

/// Device type identifier and specification revision on an [`Endpoint`] (Descriptor `DeviceTypeList`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct DeviceTypeEntry {
    /// Matter device type identifier.
    pub id: DeviceTypeId,
    /// Specification revision of the device type.
    pub revision: u16,
}

impl DeviceTypeEntry {
    /// Construct a new device type entry with an identifier and specification revision.
    #[must_use]
    pub const fn new(id: DeviceTypeId, revision: u16) -> Self {
        Self { id, revision }
    }
}

impl Display for DeviceTypeEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} (rev {})", self.id, self.revision)
    }
}

impl From<DeviceTypeEntry> for AttributeValue {
    fn from(entry: DeviceTypeEntry) -> Self {
        Self::Struct(alloc::vec![
            (FieldId::new(0), Self::U32(entry.id.raw())),
            (FieldId::new(1), Self::U16(entry.revision)),
        ])
    }
}

impl TryFrom<&AttributeValue> for DeviceTypeEntry {
    type Error = MatterStatus;

    fn try_from(value: &AttributeValue) -> Result<Self, Self::Error> {
        let AttributeValue::Struct(fields) = value else {
            return Err(MatterStatus::InvalidDataType);
        };

        let mut id: Option<DeviceTypeId> = None;
        let mut revision: Option<u16> = None;
        let mut seen_unknown: Vec<FieldId> = Vec::new();

        for (field_id, val) in fields {
            match field_id.raw() {
                0 => {
                    if id.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    id = Some(DeviceTypeId::try_from(val)?);
                }
                1 => {
                    if revision.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    revision = Some(decode_unsigned::<u16>(val)?);
                }
                _ => {
                    if seen_unknown.contains(field_id) {
                        return Err(MatterStatus::ConstraintError);
                    }
                    seen_unknown.push(*field_id);
                }
            }
        }

        Ok(Self {
            id: id.ok_or(MatterStatus::ConstraintError)?,
            revision: revision.ok_or(MatterStatus::ConstraintError)?,
        })
    }
}

impl TryFrom<AttributeValue> for DeviceTypeEntry {
    type Error = MatterStatus;

    fn try_from(value: AttributeValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

/// Stateful wire representation of a Matter Endpoint on a [`crate::node::Node`] per Section 7.3.
///
/// An endpoint has no native wire attributes of its own; endpoint-level metadata (`DeviceTypeList`,
/// `ServerList`, `ClientList`, `PartsList`, `TagList`, and `EndpointUniqueID`) is carried by its
/// Descriptor cluster (`0x001D`) inside [`Self::clusters`].
///
/// Adding or removing clusters via [`Self::with_cluster`] / [`Self::set_cluster`] does not automatically mutate
/// Descriptor `ServerList` (`0x0001`), so partial/delta snapshots never fabricate Descriptor state.
///
/// When authoring a complete endpoint snapshot, either set `ServerList` explicitly with [`Self::with_server_list`] or
/// call [`Self::with_synced_server_list`] / [`Self::sync_server_list`] once all clusters have been added.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Endpoint {
    /// Matter endpoint identifier.
    pub id: EndpointId,
    /// Server cluster instances hosted on this endpoint (including Descriptor `0x001D`).
    pub clusters: Vec<Cluster>,
}

impl Endpoint {
    /// Construct an endpoint with its collection of server cluster instances.
    ///
    /// Callers should provide clusters with unique [`ClusterId`]s; lookups and updates
    /// operate on the first matching ID.
    #[must_use]
    pub const fn new(id: EndpointId, clusters: Vec<Cluster>) -> Self {
        Self { id, clusters }
    }

    /// Construct an empty endpoint with no clusters.
    #[must_use]
    pub const fn empty(id: EndpointId) -> Self {
        Self {
            id,
            clusters: Vec::new(),
        }
    }

    /// Construct a single-device-type endpoint, ensuring its Descriptor cluster (`0x001D`)
    /// declares `DeviceTypeList` (`0x0000`) and a synchronized `ServerList` (`0x0001`).
    #[must_use]
    pub fn single_device(id: EndpointId, device_type_id: DeviceTypeId, revision: u16, clusters: Vec<Cluster>) -> Self {
        let mut ep = Self::new(id, clusters);
        ep.set_device_type_list([DeviceTypeEntry::new(device_type_id, revision)]);
        ep.sync_server_list();
        ep
    }

    /// Set or override the endpoint identifier.
    #[must_use]
    pub const fn with_id(mut self, id: EndpointId) -> Self {
        self.id = id;
        self
    }

    /// Add or update a server cluster instance on this endpoint.
    ///
    /// Does not modify Descriptor `ServerList` (`0x0001`); call [`Self::with_synced_server_list`] at the end of fluent
    /// construction when authoring a complete endpoint snapshot.
    #[must_use]
    pub fn with_cluster(mut self, cluster: impl Into<Cluster>) -> Self {
        self.set_cluster(cluster);
        self
    }

    /// Add or update multiple server cluster instances on this endpoint.
    ///
    /// Does not modify Descriptor `ServerList` (`0x0001`); call [`Self::with_synced_server_list`] at the end of fluent
    /// construction when authoring a complete endpoint snapshot.
    #[must_use]
    pub fn with_clusters(mut self, clusters: impl IntoIterator<Item = Cluster>) -> Self {
        for cluster in clusters {
            self.set_cluster(cluster);
        }
        self
    }

    /// Set Descriptor `DeviceTypeList` (`0x0000`) on this endpoint's Descriptor cluster (`0x001D`).
    #[must_use]
    pub fn with_device_type_list(mut self, entries: impl IntoIterator<Item = DeviceTypeEntry>) -> Self {
        self.set_device_type_list(entries);
        self
    }

    /// Set Descriptor `ServerList` (`0x0001`) on this endpoint's Descriptor cluster (`0x001D`).
    #[must_use]
    pub fn with_server_list(mut self, cluster_ids: impl IntoIterator<Item = ClusterId>) -> Self {
        self.set_server_list(cluster_ids);
        self
    }

    /// Synchronize Descriptor `ServerList` (`0x0001`) with the cluster IDs currently hosted in [`Self::clusters`].
    ///
    /// Ensures Descriptor (`0x001D`) is present in [`Self::clusters`] and deduplicates IDs in first-seen order.
    /// This operation is idempotent and intended to be called at the end of fluent construction when authoring a
    /// complete endpoint snapshot.
    #[must_use]
    pub fn with_synced_server_list(mut self) -> Self {
        self.sync_server_list();
        self
    }

    /// Set Descriptor `ClientList` (`0x0002`) on this endpoint's Descriptor cluster (`0x001D`).
    #[must_use]
    pub fn with_client_list(mut self, cluster_ids: impl IntoIterator<Item = ClusterId>) -> Self {
        self.set_client_list(cluster_ids);
        self
    }

    /// Set Descriptor `PartsList` (`0x0003`) on this endpoint's Descriptor cluster (`0x001D`).
    #[must_use]
    pub fn with_parts_list(mut self, endpoint_ids: impl IntoIterator<Item = EndpointId>) -> Self {
        self.set_parts_list(endpoint_ids);
        self
    }

    /// Set Descriptor `TagList` (`0x0004`) on this endpoint's Descriptor cluster (`0x001D`).
    #[must_use]
    pub fn with_tag_list(mut self, tags: impl IntoIterator<Item = SemanticTag>) -> Self {
        self.set_tag_list(tags);
        self
    }

    /// Set Descriptor `EndpointUniqueID` (`0x0005`, Matter `v1.4`+) on this endpoint's Descriptor cluster (`0x001D`).
    #[must_use]
    pub fn with_unique_id(mut self, unique_id: impl Into<String>) -> Self {
        self.set_unique_id(unique_id);
        self
    }

    /// Set or omit Descriptor `EndpointUniqueID` (`0x0005`, Matter `v1.4`+) on this endpoint's Descriptor cluster
    /// (`0x001D`).
    #[must_use]
    pub fn with_optional_unique_id(mut self, unique_id: impl Into<Optional<String>>) -> Self {
        match unique_id.into() {
            Optional::Present(uid) => {
                self.set_unique_id(uid);
            }
            Optional::Omitted => {
                self.remove_unique_id();
            }
        }
        self
    }

    /// Return `true` if this endpoint hosts a server cluster with the given ID.
    #[must_use]
    pub fn has_cluster(&self, id: ClusterId) -> bool {
        self.clusters.iter().any(|c| c.id == id)
    }

    /// Find a server cluster on this endpoint by cluster ID.
    #[must_use]
    pub fn cluster(&self, id: ClusterId) -> Option<&Cluster> {
        self.clusters.iter().find(|c| c.id == id)
    }

    /// Return a mutable reference to a server cluster on this endpoint by cluster ID.
    #[must_use]
    pub fn cluster_mut(&mut self, id: ClusterId) -> Option<&mut Cluster> {
        self.clusters.iter_mut().find(|c| c.id == id)
    }

    /// Insert or update a server cluster on this endpoint in place, returning the previous cluster if replaced.
    pub fn set_cluster(&mut self, cluster: impl Into<Cluster>) -> Option<Cluster> {
        let cluster = cluster.into();
        if let Some(existing) = self.clusters.iter_mut().find(|c| c.id == cluster.id) {
            Some(mem::replace(existing, cluster))
        } else {
            self.clusters.push(cluster);
            None
        }
    }

    /// Remove a server cluster by ID from this endpoint, returning it if present.
    pub fn remove_cluster(&mut self, id: ClusterId) -> Option<Cluster> {
        self.clusters
            .iter()
            .position(|c| c.id == id)
            .map(|idx| self.clusters.remove(idx))
    }

    /// Return the Descriptor cluster (`0x001D`) on this endpoint, if present.
    #[must_use]
    pub fn descriptor(&self) -> Option<&Cluster> {
        self.cluster(ClusterId::DESCRIPTOR)
    }

    /// Return a mutable reference to the Descriptor cluster (`0x001D`) on this endpoint, if present.
    #[must_use]
    pub fn descriptor_mut(&mut self) -> Option<&mut Cluster> {
        self.cluster_mut(ClusterId::DESCRIPTOR)
    }

    /// Return `true` if Descriptor `DeviceTypeList` (`0x0000`) is present on this endpoint.
    #[must_use]
    pub fn has_device_type_list(&self) -> bool {
        self.descriptor()
            .is_some_and(|c| c.has_attribute(DESCRIPTOR_DEVICE_TYPE_LIST))
    }

    /// Decode Descriptor `DeviceTypeList` (`0x0000`) as a zero-allocation [`ResultListIter`].
    ///
    /// Returns `Ok(Optional::Omitted)` when Descriptor or `DeviceTypeList` is absent, or
    /// `Err(MatterStatus::InvalidDataType)` if the attribute is not a wire list.
    pub fn device_type_list(&self) -> Result<Optional<ResultListIter<'_, DeviceTypeEntry>>, MatterStatus> {
        decode_list_attr(self, DESCRIPTOR_DEVICE_TYPE_LIST)
    }

    /// Return `true` if Descriptor `DeviceTypeList` (`0x0000`) contains the given device type ID.
    ///
    /// Returns `Ok(false)` when Descriptor or `DeviceTypeList` is absent.
    pub fn has_device_type(&self, id: DeviceTypeId) -> Result<bool, MatterStatus> {
        let Optional::Present(iter) = self.device_type_list()? else {
            return Ok(false);
        };
        for entry in iter {
            if entry?.id == id {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Return the primary device type declared on this endpoint (first entry in Descriptor `DeviceTypeList`, `0x0000`),
    /// or `Optional::Omitted` if absent or empty.
    pub fn primary_device_type(&self) -> Result<Optional<DeviceTypeEntry>, MatterStatus> {
        let Optional::Present(mut iter) = self.device_type_list()? else {
            return Ok(Optional::Omitted);
        };
        match iter.next() {
            Some(res) => res.map(Optional::Present),
            None => Ok(Optional::Omitted),
        }
    }

    /// Set Descriptor `DeviceTypeList` (`0x0000`) in place, inserting Descriptor (`0x001D`) if absent.
    pub fn set_device_type_list(
        &mut self,
        entries: impl IntoIterator<Item = DeviceTypeEntry>,
    ) -> Option<AttributeValue> {
        let list = AttributeValue::List(entries.into_iter().map(AttributeValue::from).collect());
        self.set_descriptor_attribute(DESCRIPTOR_DEVICE_TYPE_LIST, list)
    }

    /// Remove Descriptor `DeviceTypeList` (`0x0000`) in place, returning the previous value if present.
    pub fn remove_device_type_list(&mut self) -> Option<AttributeValue> {
        self.descriptor_mut()?.remove_attribute(DESCRIPTOR_DEVICE_TYPE_LIST)
    }

    /// Return `true` if Descriptor `ServerList` (`0x0001`) is present on this endpoint.
    #[must_use]
    pub fn has_server_list(&self) -> bool {
        self.descriptor()
            .is_some_and(|c| c.has_attribute(DESCRIPTOR_SERVER_LIST))
    }

    /// Decode Descriptor `ServerList` (`0x0001`) as a zero-allocation [`ResultListIter`].
    ///
    /// Returns `Ok(Optional::Omitted)` when Descriptor or `ServerList` is absent, or
    /// `Err(MatterStatus::InvalidDataType)` if the attribute is not a wire list.
    pub fn server_list(&self) -> Result<Optional<ResultListIter<'_, ClusterId>>, MatterStatus> {
        decode_list_attr(self, DESCRIPTOR_SERVER_LIST)
    }

    /// Set Descriptor `ServerList` (`0x0001`) in place, inserting Descriptor (`0x001D`) if absent and deduplicating
    /// IDs in first-seen order.
    pub fn set_server_list(&mut self, cluster_ids: impl IntoIterator<Item = ClusterId>) -> Option<AttributeValue> {
        self.set_descriptor_id_list(DESCRIPTOR_SERVER_LIST, cluster_ids)
    }

    /// Remove Descriptor `ServerList` (`0x0001`) in place, returning the previous value if present.
    pub fn remove_server_list(&mut self) -> Option<AttributeValue> {
        self.descriptor_mut()?.remove_attribute(DESCRIPTOR_SERVER_LIST)
    }

    /// Synchronize Descriptor `ServerList` (`0x0001`) with the cluster IDs currently hosted in [`Self::clusters`],
    /// deduplicating IDs in first-seen order.
    ///
    /// Ensures Descriptor (`0x001D`) is present in [`Self::clusters`] before writing `ServerList`, returning the
    /// previous attribute value if replaced. This operation is idempotent.
    pub fn sync_server_list(&mut self) -> Option<AttributeValue> {
        let has_desc = self.has_cluster(ClusterId::DESCRIPTOR);
        let mut values = Vec::with_capacity(self.clusters.len().saturating_add(usize::from(!has_desc)));
        if !has_desc {
            values.push(AttributeValue::from(ClusterId::DESCRIPTOR));
        }
        for cluster in &self.clusters {
            let val = AttributeValue::from(cluster.id);
            if !values.contains(&val) {
                values.push(val);
            }
        }
        self.set_descriptor_attribute(DESCRIPTOR_SERVER_LIST, AttributeValue::List(values))
    }

    /// Return `true` if Descriptor `ClientList` (`0x0002`) is present on this endpoint.
    #[must_use]
    pub fn has_client_list(&self) -> bool {
        self.descriptor()
            .is_some_and(|c| c.has_attribute(DESCRIPTOR_CLIENT_LIST))
    }

    /// Decode Descriptor `ClientList` (`0x0002`) as a zero-allocation [`ResultListIter`].
    ///
    /// Returns `Ok(Optional::Omitted)` when Descriptor or `ClientList` is absent, or
    /// `Err(MatterStatus::InvalidDataType)` if the attribute is not a wire list.
    pub fn client_list(&self) -> Result<Optional<ResultListIter<'_, ClusterId>>, MatterStatus> {
        decode_list_attr(self, DESCRIPTOR_CLIENT_LIST)
    }

    /// Set Descriptor `ClientList` (`0x0002`) in place, inserting Descriptor (`0x001D`) if absent and deduplicating
    /// IDs in first-seen order.
    pub fn set_client_list(&mut self, cluster_ids: impl IntoIterator<Item = ClusterId>) -> Option<AttributeValue> {
        self.set_descriptor_id_list(DESCRIPTOR_CLIENT_LIST, cluster_ids)
    }

    /// Remove Descriptor `ClientList` (`0x0002`) in place, returning the previous value if present.
    pub fn remove_client_list(&mut self) -> Option<AttributeValue> {
        self.descriptor_mut()?.remove_attribute(DESCRIPTOR_CLIENT_LIST)
    }

    /// Return `true` if Descriptor `PartsList` (`0x0003`) is present on this endpoint.
    #[must_use]
    pub fn has_parts_list(&self) -> bool {
        self.descriptor()
            .is_some_and(|c| c.has_attribute(DESCRIPTOR_PARTS_LIST))
    }

    /// Decode Descriptor `PartsList` (`0x0003`) as a zero-allocation [`ResultListIter`].
    ///
    /// Returns `Ok(Optional::Omitted)` when Descriptor or `PartsList` is absent, or
    /// `Err(MatterStatus::InvalidDataType)` if the attribute is not a wire list.
    pub fn parts_list(&self) -> Result<Optional<ResultListIter<'_, EndpointId>>, MatterStatus> {
        decode_list_attr(self, DESCRIPTOR_PARTS_LIST)
    }

    /// Set Descriptor `PartsList` (`0x0003`) in place, inserting Descriptor (`0x001D`) if absent and deduplicating
    /// IDs in first-seen order.
    pub fn set_parts_list(&mut self, endpoint_ids: impl IntoIterator<Item = EndpointId>) -> Option<AttributeValue> {
        self.set_descriptor_id_list(DESCRIPTOR_PARTS_LIST, endpoint_ids)
    }

    /// Remove Descriptor `PartsList` (`0x0003`) in place, returning the previous value if present.
    pub fn remove_parts_list(&mut self) -> Option<AttributeValue> {
        self.descriptor_mut()?.remove_attribute(DESCRIPTOR_PARTS_LIST)
    }

    /// Return `true` if Descriptor `TagList` (`0x0004`) is present on this endpoint.
    #[must_use]
    pub fn has_tag_list(&self) -> bool {
        self.descriptor().is_some_and(|c| c.has_attribute(DESCRIPTOR_TAG_LIST))
    }

    /// Decode Descriptor `TagList` (`0x0004`) as a lazy [`ResultListIter`].
    ///
    /// Returns `Ok(Optional::Omitted)` when Descriptor or `TagList` is absent, or
    /// `Err(MatterStatus::InvalidDataType)` if the attribute is not a wire list.
    pub fn tag_list(&self) -> Result<Optional<ResultListIter<'_, SemanticTag>>, MatterStatus> {
        decode_list_attr(self, DESCRIPTOR_TAG_LIST)
    }

    /// Set Descriptor `TagList` (`0x0004`) in place, inserting Descriptor (`0x001D`) if absent.
    pub fn set_tag_list(&mut self, tags: impl IntoIterator<Item = SemanticTag>) -> Option<AttributeValue> {
        let list = AttributeValue::List(tags.into_iter().map(AttributeValue::from).collect());
        self.set_descriptor_attribute(DESCRIPTOR_TAG_LIST, list)
    }

    /// Remove Descriptor `TagList` (`0x0004`) in place, returning the previous value if present.
    pub fn remove_tag_list(&mut self) -> Option<AttributeValue> {
        self.descriptor_mut()?.remove_attribute(DESCRIPTOR_TAG_LIST)
    }

    /// Return `true` if Descriptor `EndpointUniqueID` (`0x0005`) is present on this endpoint.
    #[must_use]
    pub fn has_unique_id(&self) -> bool {
        self.descriptor()
            .is_some_and(|c| c.has_attribute(DESCRIPTOR_ENDPOINT_UNIQUE_ID))
    }

    /// Borrow Descriptor `EndpointUniqueID` (`0x0005`), or `Optional::Omitted` if absent.
    pub fn unique_id(&self) -> Result<Optional<&str>, MatterStatus> {
        let Some(val) = self
            .descriptor()
            .and_then(|c| c.attribute(DESCRIPTOR_ENDPOINT_UNIQUE_ID))
        else {
            return Ok(Optional::Omitted);
        };
        match val {
            AttributeValue::String(s) => Ok(Optional::Present(s.as_str())),
            _ => Err(MatterStatus::InvalidDataType),
        }
    }

    /// Set Descriptor `EndpointUniqueID` (`0x0005`) in place, inserting Descriptor (`0x001D`) if absent.
    pub fn set_unique_id(&mut self, unique_id: impl Into<String>) -> Option<AttributeValue> {
        self.set_descriptor_attribute(DESCRIPTOR_ENDPOINT_UNIQUE_ID, AttributeValue::String(unique_id.into()))
    }

    /// Remove Descriptor `EndpointUniqueID` (`0x0005`) in place, returning the previous value if present.
    pub fn remove_unique_id(&mut self) -> Option<AttributeValue> {
        self.descriptor_mut()?.remove_attribute(DESCRIPTOR_ENDPOINT_UNIQUE_ID)
    }

    /// Convenience lookup for an attribute value across `cluster -> attribute`.
    #[must_use]
    pub fn attribute(&self, cluster_id: ClusterId, attribute_id: AttributeId) -> Option<&AttributeValue> {
        self.cluster(cluster_id)?.attribute(attribute_id)
    }

    /// Convenience mutable lookup for an attribute value across `cluster -> attribute`.
    #[must_use]
    pub fn attribute_mut(&mut self, cluster_id: ClusterId, attribute_id: AttributeId) -> Option<&mut AttributeValue> {
        self.cluster_mut(cluster_id)?.attribute_mut(attribute_id)
    }

    fn set_descriptor_id_list<T: Into<AttributeValue>>(
        &mut self,
        attr_id: AttributeId,
        ids: impl IntoIterator<Item = T>,
    ) -> Option<AttributeValue> {
        let iter = ids.into_iter();
        let mut values = Vec::with_capacity(iter.size_hint().0);
        for id in iter {
            let val = id.into();
            if !values.contains(&val) {
                values.push(val);
            }
        }
        self.set_descriptor_attribute(attr_id, AttributeValue::List(values))
    }

    fn set_descriptor_attribute(&mut self, attr_id: AttributeId, value: AttributeValue) -> Option<AttributeValue> {
        if let Some(desc) = self.descriptor_mut() {
            desc.set_attribute(attr_id, value)
        } else {
            let desc = Cluster::empty(ClusterId::DESCRIPTOR).with_attribute(attr_id, value);
            self.clusters.insert(0, desc);
            None
        }
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Endpoint {} ({} clusters)", self.id, self.clusters.len())
    }
}

fn decode_list_attr<T>(ep: &Endpoint, attr_id: AttributeId) -> Result<Optional<ResultListIter<'_, T>>, MatterStatus> {
    let Some(val) = ep.descriptor().and_then(|c| c.attribute(attr_id)) else {
        return Ok(Optional::Omitted);
    };
    match val {
        AttributeValue::List(items) => Ok(Optional::Present(ResultListIter::new(items))),
        _ => Err(MatterStatus::InvalidDataType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cluster::DataVersion, types::VendorId};

    #[test]
    fn endpoint_id_and_device_type_id() {
        assert_eq!(EndpointId::ROOT.raw(), 0);
        assert_eq!(EndpointId::WILDCARD.raw(), 0xFFFF);
        assert_eq!(EndpointId::new(1).raw(), 1);
        assert_eq!(alloc::format!("{}", EndpointId::ROOT), "0");
        assert_eq!(u16::from(EndpointId::new(1)), 1);

        assert_eq!(DeviceTypeId::new(0x0100).raw(), 0x0100);
        assert_eq!(alloc::format!("{}", DeviceTypeId::new(0x0100)), "0x0100");
        assert_eq!(u32::from(DeviceTypeId::new(0x0100)), 0x0100);

        let val_ep = AttributeValue::from(EndpointId::new(1));
        assert_eq!(EndpointId::try_from(&val_ep), Ok(EndpointId::new(1)));
        assert_eq!(EndpointId::try_from(val_ep), Ok(EndpointId::new(1)));

        let val_dt = AttributeValue::from(DeviceTypeId::new(0x0100));
        assert_eq!(DeviceTypeId::try_from(&val_dt), Ok(DeviceTypeId::new(0x0100)));
        assert_eq!(DeviceTypeId::try_from(val_dt), Ok(DeviceTypeId::new(0x0100)));
    }

    #[test]
    fn endpoint_construction_and_lookups() {
        const ON_OFF_LIGHT: DeviceTypeId = DeviceTypeId::new(0x0100);
        const DIMMABLE_LIGHT: DeviceTypeId = DeviceTypeId::new(0x0101);
        const THERMOSTAT_DEVICE: DeviceTypeId = DeviceTypeId::new(0x0301);
        const ON_OFF: ClusterId = ClusterId::new(0x0006);
        const BASIC_INFO: ClusterId = ClusterId::new(0x0028);
        const IDENTIFY: ClusterId = ClusterId::new(0x0003);
        const THERMOSTAT_CLUSTER: ClusterId = ClusterId::new(0x0201);

        let ep = Endpoint::single_device(EndpointId::new(1), ON_OFF_LIGHT, 1, alloc::vec![Cluster::empty(ON_OFF)])
            .with_device_type_list([
                DeviceTypeEntry::new(ON_OFF_LIGHT, 1),
                DeviceTypeEntry::new(DIMMABLE_LIGHT, 2),
            ])
            .with_client_list([BASIC_INFO, IDENTIFY]);

        assert_eq!(ep.id, EndpointId::new(1));
        assert_eq!(ep.has_device_type(ON_OFF_LIGHT), Ok(true));
        assert_eq!(ep.has_device_type(DIMMABLE_LIGHT), Ok(true));
        assert_eq!(ep.has_device_type(THERMOSTAT_DEVICE), Ok(false));

        assert_eq!(
            ep.primary_device_type(),
            Ok(Optional::Present(DeviceTypeEntry::new(ON_OFF_LIGHT, 1)))
        );
        assert!(ep.cluster(ON_OFF).is_some());
        assert!(ep.cluster(ClusterId::DESCRIPTOR).is_some());
        assert!(ep.cluster(THERMOSTAT_CLUSTER).is_none());
        assert_eq!(
            ep.server_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![ClusterId::DESCRIPTOR, ON_OFF]))
        );
        assert_eq!(
            ep.client_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![BASIC_INFO, IDENTIFY]))
        );

        let mut ep = ep.with_cluster(Cluster::empty(ON_OFF).with_data_version(DataVersion::new(2)));
        assert_eq!(ep.clusters.len(), 2);
        assert_eq!(
            ep.cluster(ON_OFF).unwrap().data_version,
            Optional::Present(DataVersion::new(2))
        );

        assert!(ep.has_cluster(ON_OFF));
        assert!(ep.cluster_mut(ON_OFF).is_some());
        assert!(ep.remove_cluster(ON_OFF).is_some());
        assert!(!ep.has_cluster(ON_OFF));
        assert!(EndpointId::ROOT.is_root());
        assert!(EndpointId::WILDCARD.is_wildcard());
    }

    #[test]
    fn remove_cluster_preserves_order() {
        let mut ep = Endpoint::new(EndpointId::new(1), alloc::vec![])
            .with_cluster(Cluster::empty(ClusterId::new(1)))
            .with_cluster(Cluster::empty(ClusterId::new(2)))
            .with_cluster(Cluster::empty(ClusterId::new(3)))
            .with_cluster(Cluster::empty(ClusterId::new(4)));

        assert!(ep.remove_cluster(ClusterId::new(2)).is_some());
        let ids: Vec<u32> = ep.clusters.iter().map(|c| c.id.raw()).collect();
        assert_eq!(ids, alloc::vec![1, 3, 4]);
    }

    #[test]
    fn endpoint_descriptor_helpers_and_formatting() {
        let mei_dt = DeviceTypeId::from_mei(VendorId::new(0x0001), 0x0002);
        assert_eq!(alloc::format!("{mei_dt}"), "0x00010002");
        assert_eq!("0x00010002".parse::<DeviceTypeId>(), Ok(mei_dt));
        assert_eq!("12".parse::<EndpointId>(), Ok(EndpointId::new(12)));

        const AGGREGATOR: DeviceTypeId = DeviceTypeId::new(0x000E);
        const BASIC_INFORMATION: ClusterId = ClusterId::new(0x0028);

        let mut ep = Endpoint::empty(EndpointId::new(2));
        assert!(!ep.has_device_type_list());
        assert!(!ep.has_server_list());
        assert!(!ep.has_client_list());
        assert!(!ep.has_parts_list());
        assert!(!ep.has_tag_list());
        assert!(!ep.has_unique_id());
        assert_eq!(ep.has_device_type(AGGREGATOR), Ok(false));
        assert_eq!(ep.primary_device_type(), Ok(Optional::Omitted));

        assert_eq!(ep.set_device_type_list([DeviceTypeEntry::new(AGGREGATOR, 1)]), None);
        assert!(ep.has_device_type_list());
        assert_eq!(
            ep.primary_device_type(),
            Ok(Optional::Present(DeviceTypeEntry::new(AGGREGATOR, 1)))
        );
        assert!(ep.set_client_list([BASIC_INFORMATION]).is_none());
        assert_eq!(ep.client_list().unwrap().contains(BASIC_INFORMATION), Ok(true));
        assert!(ep.sync_server_list().is_none());
        assert_eq!(
            ep.server_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![ClusterId::DESCRIPTOR]))
        );

        assert_eq!(
            ep.attribute(ClusterId::DESCRIPTOR, DESCRIPTOR_CLIENT_LIST),
            Some(&AttributeValue::List(alloc::vec![AttributeValue::from(
                BASIC_INFORMATION
            )]))
        );
        assert_eq!(alloc::format!("{ep}"), "Endpoint 2 (1 clusters)");

        assert!(ep.remove_client_list().is_some());
        assert!(!ep.has_client_list());
        assert!(ep.remove_device_type_list().is_some());
        assert!(!ep.has_device_type_list());

        // PartsList & TagList helpers
        let tag_left = SemanticTag::new(1, 2).with_label("Left");
        ep = ep
            .with_parts_list([EndpointId::new(3), EndpointId::new(4)])
            .with_tag_list([tag_left.clone()]);
        assert!(ep.has_parts_list());
        assert_eq!(ep.parts_list().unwrap().contains(EndpointId::new(3)), Ok(true));
        assert_eq!(ep.parts_list().unwrap().contains(EndpointId::new(4)), Ok(true));
        assert!(ep.remove_parts_list().is_some());
        assert!(!ep.has_parts_list());

        assert!(ep.has_tag_list());
        assert_eq!(
            ep.tag_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![tag_left]))
        );
        assert!(ep.remove_tag_list().is_some());
        assert!(!ep.has_tag_list());

        // DeviceTypeEntry wire roundtrip + ID validity
        let entry = DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 2);
        let entry_val = AttributeValue::from(entry);
        assert_eq!(DeviceTypeEntry::try_from(&entry_val), Ok(entry));
        assert_eq!(DeviceTypeEntry::try_from(entry_val), Ok(entry));
        let flex_entry = AttributeValue::Struct(alloc::vec![
            (FieldId::new(0), AttributeValue::U64(0x0100)),
            (FieldId::new(1), AttributeValue::U8(2)),
        ]);
        assert_eq!(DeviceTypeEntry::try_from(&flex_entry), Ok(entry));
        let overflow_entry = AttributeValue::Struct(alloc::vec![
            (FieldId::new(0), AttributeValue::U32(0x0100)),
            (FieldId::new(1), AttributeValue::U32(70_000)),
        ]);
        assert_eq!(
            DeviceTypeEntry::try_from(&overflow_entry),
            Err(MatterStatus::ConstraintError)
        );
        assert!(EndpointId::ROOT.is_valid());
        assert!(!EndpointId::WILDCARD.is_valid());
        assert!(DeviceTypeId::new(0x0100).is_valid());
        assert!(DeviceTypeId::from_mei(VendorId::new(0xFFF1), 0xBFFF).is_valid());
        assert!(!DeviceTypeId::new(0xC000).is_valid());
    }

    #[test]
    fn endpoint_unique_id_lifecycle_and_type_errors() {
        let mut ep = Endpoint::empty(EndpointId::new(1));
        assert!(!ep.has_unique_id());
        assert_eq!(ep.unique_id(), Ok(Optional::Omitted));

        ep = ep.with_unique_id("uid-001");
        assert!(ep.has_unique_id());
        assert_eq!(ep.unique_id(), Ok(Optional::Present("uid-001")));

        assert_eq!(
            ep.set_unique_id("uid-002"),
            Some(AttributeValue::String(String::from("uid-001")))
        );
        assert_eq!(ep.unique_id(), Ok(Optional::Present("uid-002")));

        assert_eq!(
            ep.remove_unique_id(),
            Some(AttributeValue::String(String::from("uid-002")))
        );
        assert!(!ep.has_unique_id());
        assert_eq!(ep.unique_id(), Ok(Optional::Omitted));
        assert_eq!(ep.remove_unique_id(), None);

        let ep_opt = Endpoint::empty(EndpointId::new(2)).with_optional_unique_id(Some(String::from("uid-opt")));
        assert_eq!(ep_opt.unique_id(), Ok(Optional::Present("uid-opt")));
        let ep_omitted = ep_opt.with_optional_unique_id(Optional::Omitted);
        assert!(!ep_omitted.has_unique_id());

        // Malformed wire types surface InvalidDataType
        let bad = Endpoint::empty(EndpointId::new(3)).with_cluster(
            Cluster::empty(ClusterId::DESCRIPTOR)
                .with_attribute(DESCRIPTOR_DEVICE_TYPE_LIST, true)
                .with_attribute(DESCRIPTOR_ENDPOINT_UNIQUE_ID, 42_u32),
        );
        assert_eq!(bad.device_type_list().unwrap_err(), MatterStatus::InvalidDataType);
        assert_eq!(bad.unique_id(), Err(MatterStatus::InvalidDataType));
    }

    #[test]
    fn endpoint_sync_server_list_is_idempotent() {
        const ON_OFF: ClusterId = ClusterId::new(0x0006);
        const IDENTIFY: ClusterId = ClusterId::new(0x0003);

        let mut ep = Endpoint::empty(EndpointId::new(1))
            .with_cluster(Cluster::empty(ON_OFF))
            .with_cluster(Cluster::empty(IDENTIFY))
            .with_synced_server_list();

        let expected = alloc::vec![ClusterId::DESCRIPTOR, ON_OFF, IDENTIFY];
        assert_eq!(
            ep.server_list().unwrap().to_vec(),
            Ok(Optional::Present(expected.clone()))
        );

        let prev = ep.sync_server_list();
        assert!(prev.is_some());
        assert_eq!(ep.server_list().unwrap().to_vec(), Ok(Optional::Present(expected)));
        assert_eq!(ep.clusters.len(), 3);

        // Deduplicates duplicate cluster IDs in first-seen order
        ep.clusters.push(Cluster::empty(ON_OFF));
        ep.sync_server_list();
        assert_eq!(
            ep.server_list().unwrap().to_vec(),
            Ok(Optional::Present(alloc::vec![ClusterId::DESCRIPTOR, ON_OFF, IDENTIFY]))
        );
    }
}
