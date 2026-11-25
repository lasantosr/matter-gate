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
    optional::{Nullable, Optional},
    status::MatterStatus,
    tag::SemanticTag,
    types::VendorId,
    value::{AttributeValue, FieldId},
};

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
                    let AttributeValue::U32(raw) = val else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    id = Some(DeviceTypeId::new(*raw));
                }
                1 => {
                    if revision.is_some() {
                        return Err(MatterStatus::ConstraintError);
                    }
                    let AttributeValue::U16(rev) = val else {
                        return Err(MatterStatus::InvalidDataType);
                    };
                    revision = Some(*rev);
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
    /// Device types implemented by this endpoint (Descriptor `DeviceTypeList`).
    pub device_types: Vec<DeviceTypeEntry>,
    /// Server clusters hosted on this endpoint (Descriptor `ServerList`).
    pub server_clusters: Vec<Cluster>,
    /// Client cluster IDs referenced by this endpoint (Descriptor `ClientList`).
    pub client_clusters: Vec<ClusterId>,
    /// Child endpoint IDs composed under this endpoint (Descriptor `PartsList`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub parts: Vec<EndpointId>,
    /// Semantic tags disambiguating this endpoint (Descriptor `TagList`).
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub tags: Vec<SemanticTag>,
    /// Optional persistent unique identifier disambiguating this endpoint (Descriptor `EndpointUniqueID`, `0x0005`,
    /// Matter `v1.4`+).
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Optional::is_omitted"))]
    #[cfg_attr(all(not(feature = "serde"), feature = "schemars"), schemars(default))]
    pub unique_id: Optional<String>,
}

impl Endpoint {
    /// Construct an endpoint with its complete collection of device types and clusters,
    /// and empty `parts` and `tags` lists.
    ///
    /// Callers should provide unique device type, server cluster, and client cluster IDs;
    /// lookups and updates operate on the first matching ID.
    #[must_use]
    pub const fn new(
        id: EndpointId,
        device_types: Vec<DeviceTypeEntry>,
        server_clusters: Vec<Cluster>,
        client_clusters: Vec<ClusterId>,
    ) -> Self {
        Self {
            id,
            device_types,
            server_clusters,
            client_clusters,
            parts: Vec::new(),
            tags: Vec::new(),
            unique_id: Optional::Omitted,
        }
    }

    /// Construct an empty endpoint with no device types, clusters, parts, or tags.
    #[must_use]
    pub const fn empty(id: EndpointId) -> Self {
        Self {
            id,
            device_types: Vec::new(),
            server_clusters: Vec::new(),
            client_clusters: Vec::new(),
            parts: Vec::new(),
            tags: Vec::new(),
            unique_id: Optional::Omitted,
        }
    }

    /// Construct a single-device-type endpoint with empty `parts` and `tags` lists.
    #[must_use]
    pub fn single_device(
        id: EndpointId,
        device_type_id: DeviceTypeId,
        revision: u16,
        server_clusters: Vec<Cluster>,
        client_clusters: Vec<ClusterId>,
    ) -> Self {
        Self {
            id,
            device_types: alloc::vec![DeviceTypeEntry::new(device_type_id, revision)],
            server_clusters,
            client_clusters,
            parts: Vec::new(),
            tags: Vec::new(),
            unique_id: Optional::Omitted,
        }
    }

    /// Set or override the endpoint identifier.
    #[must_use]
    pub const fn with_id(mut self, id: EndpointId) -> Self {
        self.id = id;
        self
    }

    /// Add or update a device type entry on this endpoint.
    #[must_use]
    pub fn with_device_type(mut self, id: DeviceTypeId, revision: u16) -> Self {
        self.set_device_type(id, revision);
        self
    }

    /// Add or update multiple device type entries on this endpoint.
    #[must_use]
    pub fn with_device_types(mut self, entries: impl IntoIterator<Item = DeviceTypeEntry>) -> Self {
        for entry in entries {
            self.set_device_type(entry.id, entry.revision);
        }
        self
    }

    /// Add or update a server cluster instance on this endpoint.
    #[must_use]
    pub fn with_server_cluster(mut self, cluster: impl Into<Cluster>) -> Self {
        self.set_server_cluster(cluster);
        self
    }

    /// Add or update multiple server cluster instances on this endpoint.
    #[must_use]
    pub fn with_server_clusters(mut self, clusters: impl IntoIterator<Item = Cluster>) -> Self {
        for cluster in clusters {
            self.set_server_cluster(cluster);
        }
        self
    }

    /// Add a client cluster reference to this endpoint if not already present.
    #[must_use]
    pub fn with_client_cluster(mut self, cluster_id: ClusterId) -> Self {
        self.add_client_cluster(cluster_id);
        self
    }

    /// Add multiple client cluster references to this endpoint if not already present.
    #[must_use]
    pub fn with_client_clusters(mut self, cluster_ids: impl IntoIterator<Item = ClusterId>) -> Self {
        for cluster_id in cluster_ids {
            self.add_client_cluster(cluster_id);
        }
        self
    }

    /// Add a child endpoint ID to `parts` (`PartsList`) if not already present.
    #[must_use]
    pub fn with_part(mut self, endpoint_id: EndpointId) -> Self {
        self.add_part(endpoint_id);
        self
    }

    /// Add multiple child endpoint IDs to `parts` (`PartsList`) if not already present.
    #[must_use]
    pub fn with_parts(mut self, endpoint_ids: impl IntoIterator<Item = EndpointId>) -> Self {
        for endpoint_id in endpoint_ids {
            self.add_part(endpoint_id);
        }
        self
    }

    /// Add or update a semantic tag in `tags` (`TagList`).
    #[must_use]
    pub fn with_tag(mut self, tag: SemanticTag) -> Self {
        self.set_tag(tag);
        self
    }

    /// Add or update multiple semantic tags in `tags` (`TagList`).
    #[must_use]
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = SemanticTag>) -> Self {
        for tag in tags {
            self.set_tag(tag);
        }
        self
    }

    /// Set the persistent unique identifier (`EndpointUniqueID`, `0x0005`, Matter `v1.4`+).
    #[must_use]
    pub fn with_unique_id(mut self, unique_id: impl Into<String>) -> Self {
        self.unique_id = Optional::Present(unique_id.into());
        self
    }

    /// Set or omit the optional persistent unique identifier (`EndpointUniqueID`, `0x0005`, Matter `v1.4`+).
    #[must_use]
    pub fn with_optional_unique_id(mut self, unique_id: impl Into<Optional<String>>) -> Self {
        self.unique_id = unique_id.into();
        self
    }

    /// Returns `true` if this endpoint exposes the given device type.
    #[must_use]
    pub fn has_device_type(&self, id: DeviceTypeId) -> bool {
        self.device_types.iter().any(|entry| entry.id == id)
    }

    /// Returns the primary device type declared on this endpoint (first entry in `device_types`).
    #[must_use]
    pub fn primary_device_type(&self) -> Option<DeviceTypeEntry> {
        self.device_types.first().copied()
    }

    /// Insert or update a device type entry on this endpoint in place, returning the previous revision if updated.
    pub fn set_device_type(&mut self, id: DeviceTypeId, revision: u16) -> Option<u16> {
        if let Some(existing) = self.device_types.iter_mut().find(|entry| entry.id == id) {
            Some(mem::replace(&mut existing.revision, revision))
        } else {
            self.device_types.push(DeviceTypeEntry::new(id, revision));
            None
        }
    }

    /// Remove a device type entry by ID from this endpoint, returning it if present.
    pub fn remove_device_type(&mut self, id: DeviceTypeId) -> Option<DeviceTypeEntry> {
        self.device_types
            .iter()
            .position(|entry| entry.id == id)
            .map(|idx| self.device_types.remove(idx))
    }

    /// Return `true` if this endpoint hosts a server cluster with the given ID.
    #[must_use]
    pub fn has_server_cluster(&self, id: ClusterId) -> bool {
        self.server_clusters.iter().any(|c| c.id == id)
    }

    /// Find a server cluster on this endpoint by cluster ID.
    #[must_use]
    pub fn server_cluster(&self, id: ClusterId) -> Option<&Cluster> {
        self.server_clusters.iter().find(|c| c.id == id)
    }

    /// Return a mutable reference to a server cluster on this endpoint by cluster ID.
    #[must_use]
    pub fn server_cluster_mut(&mut self, id: ClusterId) -> Option<&mut Cluster> {
        self.server_clusters.iter_mut().find(|c| c.id == id)
    }

    /// Insert or update a server cluster on this endpoint in place, returning the previous cluster if replaced.
    pub fn set_server_cluster(&mut self, cluster: impl Into<Cluster>) -> Option<Cluster> {
        let cluster = cluster.into();
        if let Some(existing) = self.server_clusters.iter_mut().find(|c| c.id == cluster.id) {
            Some(mem::replace(existing, cluster))
        } else {
            self.server_clusters.push(cluster);
            None
        }
    }

    /// Remove a server cluster by ID from this endpoint, returning it if present.
    pub fn remove_server_cluster(&mut self, id: ClusterId) -> Option<Cluster> {
        self.server_clusters
            .iter()
            .position(|c| c.id == id)
            .map(|idx| self.server_clusters.remove(idx))
    }

    /// Return `true` if this endpoint declares a client cluster with the given ID.
    #[must_use]
    pub fn has_client_cluster(&self, id: ClusterId) -> bool {
        self.client_clusters.contains(&id)
    }

    /// Add a client cluster reference to this endpoint in place if not already present, returning `true` if inserted.
    pub fn add_client_cluster(&mut self, cluster_id: ClusterId) -> bool {
        if self.client_clusters.contains(&cluster_id) {
            false
        } else {
            self.client_clusters.push(cluster_id);
            true
        }
    }

    /// Remove a client cluster reference by ID from this endpoint, returning `true` if removed.
    pub fn remove_client_cluster(&mut self, cluster_id: ClusterId) -> bool {
        if let Some(idx) = self.client_clusters.iter().position(|&id| id == cluster_id) {
            self.client_clusters.remove(idx);
            true
        } else {
            false
        }
    }

    /// Return `true` if `parts` (`PartsList`) contains the given child endpoint ID.
    #[must_use]
    pub fn has_part(&self, endpoint_id: EndpointId) -> bool {
        self.parts.contains(&endpoint_id)
    }

    /// Add a child endpoint ID to `parts` (`PartsList`) in place if not already present, returning `true` if inserted.
    pub fn add_part(&mut self, endpoint_id: EndpointId) -> bool {
        if self.parts.contains(&endpoint_id) {
            false
        } else {
            self.parts.push(endpoint_id);
            true
        }
    }

    /// Remove a child endpoint ID from `parts` (`PartsList`), returning `true` if removed.
    pub fn remove_part(&mut self, endpoint_id: EndpointId) -> bool {
        if let Some(idx) = self.parts.iter().position(|&id| id == endpoint_id) {
            self.parts.remove(idx);
            true
        } else {
            false
        }
    }

    /// Return `true` if `tags` (`TagList`) contains a tag matching `(mfg_code, namespace_id, tag)`.
    #[must_use]
    pub fn has_tag(&self, mfg_code: Nullable<VendorId>, namespace_id: u8, tag: u8) -> bool {
        self.tags
            .iter()
            .any(|t| t.mfg_code == mfg_code && t.namespace_id == namespace_id && t.tag == tag)
    }

    /// Insert or update a semantic tag in `tags` (`TagList`) in place, matching on
    /// `(mfg_code, namespace_id, tag)` and returning the previous entry if replaced.
    pub fn set_tag(&mut self, tag: SemanticTag) -> Option<SemanticTag> {
        if let Some(existing) = self
            .tags
            .iter_mut()
            .find(|t| t.mfg_code == tag.mfg_code && t.namespace_id == tag.namespace_id && t.tag == tag.tag)
        {
            Some(mem::replace(existing, tag))
        } else {
            self.tags.push(tag);
            None
        }
    }

    /// Remove a semantic tag by `(mfg_code, namespace_id, tag)` from `tags` (`TagList`), returning it if present.
    pub fn remove_tag(&mut self, mfg_code: Nullable<VendorId>, namespace_id: u8, tag: u8) -> Option<SemanticTag> {
        self.tags
            .iter()
            .position(|t| t.mfg_code == mfg_code && t.namespace_id == namespace_id && t.tag == tag)
            .map(|idx| self.tags.remove(idx))
    }

    /// Return `true` if this endpoint has a persistent unique identifier (`EndpointUniqueID`, `0x0005`).
    #[must_use]
    pub const fn has_unique_id(&self) -> bool {
        self.unique_id.is_present()
    }

    /// Return the persistent unique identifier (`EndpointUniqueID`, `0x0005`), or `Optional::Omitted` if absent.
    #[must_use]
    pub fn unique_id(&self) -> Optional<&str> {
        self.unique_id.as_deref()
    }

    /// Set or replace the persistent unique identifier (`EndpointUniqueID`, `0x0005`), returning the previous value if
    /// present.
    pub fn set_unique_id(&mut self, unique_id: impl Into<String>) -> Option<String> {
        mem::replace(&mut self.unique_id, Optional::Present(unique_id.into())).into_option()
    }

    /// Remove the persistent unique identifier (`EndpointUniqueID`, `0x0005`), returning it if present.
    pub fn remove_unique_id(&mut self) -> Option<String> {
        mem::take(&mut self.unique_id).into_option()
    }

    /// Convenience lookup for an attribute value across `server_cluster -> attribute`.
    #[must_use]
    pub fn attribute(&self, cluster_id: ClusterId, attribute_id: AttributeId) -> Option<&AttributeValue> {
        self.server_cluster(cluster_id)?.attribute(attribute_id)
    }

    /// Convenience mutable lookup for an attribute value across `server_cluster -> attribute`.
    #[must_use]
    pub fn attribute_mut(&mut self, cluster_id: ClusterId, attribute_id: AttributeId) -> Option<&mut AttributeValue> {
        self.server_cluster_mut(cluster_id)?.attribute_mut(attribute_id)
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Endpoint {} ({} server clusters, {} client clusters)",
            self.id,
            self.server_clusters.len(),
            self.client_clusters.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cluster::DataVersion, optional::Optional, types::VendorId};

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

        let ep = Endpoint::single_device(
            EndpointId::new(1),
            ON_OFF_LIGHT,
            1,
            alloc::vec![Cluster::empty(ON_OFF)],
            alloc::vec![BASIC_INFO],
        )
        .with_device_type(DIMMABLE_LIGHT, 2)
        .with_client_cluster(IDENTIFY);

        assert_eq!(ep.id, EndpointId::new(1));
        assert!(ep.has_device_type(ON_OFF_LIGHT));
        assert!(ep.has_device_type(DIMMABLE_LIGHT));
        assert!(!ep.has_device_type(THERMOSTAT_DEVICE));

        assert_eq!(ep.primary_device_type(), Some(DeviceTypeEntry::new(ON_OFF_LIGHT, 1)));
        assert!(ep.server_cluster(ON_OFF).is_some());
        assert!(ep.server_cluster(THERMOSTAT_CLUSTER).is_none());
        assert_eq!(ep.client_clusters.len(), 2);

        // Deduplicating builders & in-place mutation
        let mut ep = ep
            .with_device_type(ON_OFF_LIGHT, 3)
            .with_client_cluster(IDENTIFY)
            .with_server_cluster(Cluster::empty(ON_OFF).with_data_version(DataVersion::new(2)));
        assert_eq!(ep.device_types.len(), 2);
        assert_eq!(ep.primary_device_type(), Some(DeviceTypeEntry::new(ON_OFF_LIGHT, 3)));
        assert_eq!(ep.client_clusters.len(), 2);
        assert_eq!(ep.server_clusters.len(), 1);
        assert_eq!(
            ep.server_cluster(ON_OFF).unwrap().data_version,
            Optional::Present(DataVersion::new(2))
        );

        assert!(ep.has_server_cluster(ON_OFF));
        assert!(ep.has_client_cluster(IDENTIFY));
        assert!(ep.server_cluster_mut(ON_OFF).is_some());
        assert!(ep.remove_server_cluster(ON_OFF).is_some());
        assert!(!ep.has_server_cluster(ON_OFF));
        assert!(EndpointId::ROOT.is_root());
        assert!(EndpointId::WILDCARD.is_wildcard());
    }

    #[test]
    fn remove_server_cluster_preserves_order() {
        let mut ep = Endpoint::new(EndpointId::new(1), alloc::vec![], alloc::vec![], alloc::vec![])
            .with_server_cluster(Cluster::empty(ClusterId::new(1)))
            .with_server_cluster(Cluster::empty(ClusterId::new(2)))
            .with_server_cluster(Cluster::empty(ClusterId::new(3)))
            .with_server_cluster(Cluster::empty(ClusterId::new(4)));

        assert!(ep.remove_server_cluster(ClusterId::new(2)).is_some());
        let ids: Vec<u32> = ep.server_clusters.iter().map(|c| c.id.raw()).collect();
        assert_eq!(ids, alloc::vec![1, 3, 4]);
    }

    #[test]
    fn endpoint_helpers_and_formatting() {
        let mei_dt = DeviceTypeId::from_mei(VendorId::new(0x0001), 0x0002);
        assert_eq!(alloc::format!("{mei_dt}"), "0x00010002");
        assert_eq!("0x00010002".parse::<DeviceTypeId>(), Ok(mei_dt));
        assert_eq!("12".parse::<EndpointId>(), Ok(EndpointId::new(12)));

        const AGGREGATOR: DeviceTypeId = DeviceTypeId::new(0x000E);
        const BASIC_INFORMATION: ClusterId = ClusterId::new(0x0028);

        let mut ep = Endpoint::empty(EndpointId::new(2));
        assert_eq!(ep.set_device_type(AGGREGATOR, 1), None);
        assert_eq!(ep.set_device_type(AGGREGATOR, 2), Some(1));
        assert!(ep.add_client_cluster(BASIC_INFORMATION));
        assert!(!ep.add_client_cluster(BASIC_INFORMATION));
        ep.set_server_cluster(Cluster::empty(ClusterId::DESCRIPTOR).with_attribute(AttributeId::new(0), 1_u8));
        assert_eq!(
            ep.attribute(ClusterId::DESCRIPTOR, AttributeId::new(0)),
            Some(&AttributeValue::U8(1))
        );
        if let Some(v) = ep.attribute_mut(ClusterId::DESCRIPTOR, AttributeId::new(0)) {
            *v = AttributeValue::U8(2);
        }
        assert_eq!(
            ep.attribute(ClusterId::DESCRIPTOR, AttributeId::new(0)),
            Some(&AttributeValue::U8(2))
        );
        assert_eq!(
            alloc::format!("{ep}"),
            "Endpoint 2 (1 server clusters, 1 client clusters)"
        );
        assert!(ep.remove_client_cluster(BASIC_INFORMATION));
        assert!(!ep.remove_client_cluster(BASIC_INFORMATION));
        assert!(ep.remove_device_type(AGGREGATOR).is_some());

        // PartsList & TagList helpers
        ep = ep
            .with_part(EndpointId::new(3))
            .with_parts([EndpointId::new(3), EndpointId::new(4)])
            .with_tag(SemanticTag::new(1, 2).with_label("Left"));
        assert!(ep.has_part(EndpointId::new(3)));
        assert!(ep.has_part(EndpointId::new(4)));
        assert_eq!(ep.parts.len(), 2);
        assert!(ep.remove_part(EndpointId::new(3)));
        assert!(!ep.remove_part(EndpointId::new(3)));
        assert!(ep.has_tag(Nullable::Null, 1, 2));
        let prev_tag = ep.set_tag(SemanticTag::new(1, 2).with_label("Top"));
        assert!(prev_tag.is_some());
        assert_eq!(ep.tags.len(), 1);
        assert!(ep.remove_tag(Nullable::Null, 1, 2).is_some());
        assert!(!ep.has_tag(Nullable::Null, 1, 2));

        // DeviceTypeEntry wire roundtrip + ID validity
        let entry = DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 2);
        let entry_val = AttributeValue::from(entry);
        assert_eq!(DeviceTypeEntry::try_from(&entry_val), Ok(entry));
        assert_eq!(DeviceTypeEntry::try_from(entry_val), Ok(entry));
        assert!(EndpointId::ROOT.is_valid());
        assert!(!EndpointId::WILDCARD.is_valid());
        assert!(DeviceTypeId::new(0x0100).is_valid());
        assert!(DeviceTypeId::from_mei(VendorId::new(0xFFF1), 0xBFFF).is_valid());
        assert!(!DeviceTypeId::new(0xC000).is_valid());
    }

    #[test]
    fn endpoint_unique_id_lifecycle() {
        let mut ep = Endpoint::empty(EndpointId::new(1));
        assert!(!ep.has_unique_id());
        assert_eq!(ep.unique_id(), Optional::Omitted);

        ep = ep.with_unique_id("uid-001");
        assert!(ep.has_unique_id());
        assert_eq!(ep.unique_id(), Optional::Present("uid-001"));

        assert_eq!(ep.set_unique_id("uid-002"), Some(String::from("uid-001")));
        assert_eq!(ep.unique_id(), Optional::Present("uid-002"));

        assert_eq!(ep.remove_unique_id(), Some(String::from("uid-002")));
        assert!(!ep.has_unique_id());
        assert_eq!(ep.unique_id(), Optional::Omitted);
        assert_eq!(ep.remove_unique_id(), None);

        let ep_opt = Endpoint::empty(EndpointId::new(2)).with_optional_unique_id(Some(String::from("uid-opt")));
        assert_eq!(ep_opt.unique_id(), Optional::Present("uid-opt"));
        let ep_omitted = ep_opt.with_optional_unique_id(Optional::Omitted);
        assert!(!ep_omitted.has_unique_id());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn endpoint_unique_id_serde_omission_and_presence() {
        use serde::ser::{Error as SerError, Impossible, Serialize, SerializeStruct, Serializer, StdError};

        #[derive(Debug, PartialEq)]
        struct SerErr;
        impl Display for SerErr {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_str("ser error")
            }
        }
        impl StdError for SerErr {}
        impl SerError for SerErr {
            fn custom<T: Display>(_msg: T) -> Self {
                Self
            }
        }

        struct FieldRecorder<'a>(&'a mut Vec<&'static str>);
        impl SerializeStruct for FieldRecorder<'_> {
            type Error = SerErr;
            type Ok = ();

            fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, _value: &T) -> Result<(), SerErr> {
                self.0.push(key);
                Ok(())
            }

            fn end(self) -> Result<(), SerErr> {
                Ok(())
            }
        }
        impl<'a> Serializer for &'a mut FieldRecorder<'_> {
            type Error = SerErr;
            type Ok = ();
            type SerializeMap = Impossible<(), SerErr>;
            type SerializeSeq = Impossible<(), SerErr>;
            type SerializeStruct = FieldRecorder<'a>;
            type SerializeStructVariant = Impossible<(), SerErr>;
            type SerializeTuple = Impossible<(), SerErr>;
            type SerializeTupleStruct = Impossible<(), SerErr>;
            type SerializeTupleVariant = Impossible<(), SerErr>;

            fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, SerErr> {
                Ok(FieldRecorder(self.0))
            }

            fn serialize_bool(self, _: bool) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_i8(self, _: i8) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_i16(self, _: i16) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_i32(self, _: i32) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_i64(self, _: i64) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_u8(self, _: u8) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_u16(self, _: u16) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_u32(self, _: u32) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_u64(self, _: u64) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_f32(self, _: f32) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_f64(self, _: f64) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_char(self, _: char) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_str(self, _: &str) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_bytes(self, _: &[u8]) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_none(self) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_unit(self) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_unit_struct(self, _: &'static str) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, _: &T) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_newtype_variant<T: ?Sized + Serialize>(
                self,
                _: &'static str,
                _: u32,
                _: &'static str,
                _: &T,
            ) -> Result<(), SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_tuple_variant(
                self,
                _: &'static str,
                _: u32,
                _: &'static str,
                _: usize,
            ) -> Result<Self::SerializeTupleVariant, SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, SerErr> {
                Err(SerErr::custom("unexpected"))
            }

            fn serialize_struct_variant(
                self,
                _: &'static str,
                _: u32,
                _: &'static str,
                _: usize,
            ) -> Result<Self::SerializeStructVariant, SerErr> {
                Err(SerErr::custom("unexpected"))
            }
        }

        let mut fields = Vec::new();
        let omitted = Endpoint::empty(EndpointId::new(1));
        assert_eq!(omitted.serialize(&mut FieldRecorder(&mut fields)), Ok(()));
        assert!(!fields.contains(&"unique_id"));

        fields.clear();
        let present = Endpoint::empty(EndpointId::new(2)).with_unique_id("matter-uid-42");
        assert_eq!(present.serialize(&mut FieldRecorder(&mut fields)), Ok(()));
        assert!(fields.contains(&"unique_id"));
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn endpoint_json_schema_marks_unique_id_optional() {
        let mut generator = schemars::SchemaGenerator::default();
        let schema = <Endpoint as schemars::JsonSchema>::json_schema(&mut generator);
        let required = schema
            .as_object()
            .and_then(|o| o.get("required"))
            .and_then(|v| v.as_array());
        assert!(
            required.is_some_and(|req| !req.iter().any(|v| v.as_str() == Some("unique_id"))),
            "unique_id must not be required in Endpoint JSON schema"
        );
    }
}
