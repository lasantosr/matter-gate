//! Matter Node entity per CSA Matter Specification Section 7.2.

extern crate alloc;

use alloc::vec::Vec;
use core::{
    fmt::{Display, Formatter, Result as FmtResult},
    mem,
};

use crate::{
    attribute::AttributeId,
    cluster::ClusterId,
    endpoint::{Endpoint, EndpointId},
    types::GroupId,
    value::AttributeValue,
};

define_matter_id! {
    /// Strongly-typed Matter Node identifier (`u64`).
    pub struct NodeId(u64);
    display = hex;
    mei = false;
}

impl NodeId {
    /// Prefix for Case Authenticated Tag (CAT) Node IDs (`0xFFFF_FFFD_0000_0000`) per CSA Matter Core Specification
    /// §2.5.5.1 & §6.6.2.
    const CAT_PREFIX: u64 = 0xFFFF_FFFD_0000_0000;
    /// Prefix for Group Node IDs (`0xFFFF_FFFF_FFFF_0000`) per CSA Matter Core Specification §2.5.5.1.
    const GROUP_PREFIX: u64 = 0xFFFF_FFFF_FFFF_0000;
    /// Unspecified Node ID (`0x0000_0000_0000_0000`) per CSA Matter Core Specification §2.5.5.1.
    pub const UNSPECIFIED: Self = Self(0);

    /// Return `true` if this is the Unspecified Node ID (`0x0000_0000_0000_0000`).
    #[must_use]
    pub const fn is_unspecified(self) -> bool {
        self.0 == 0
    }

    /// Construct a Case Authenticated Tag (CAT) Node ID (`0xFFFF_FFFD_<cat_id><version>`) per §2.5.5.1 & §6.6.2.
    #[must_use]
    pub const fn from_cat(cat_id: u16, version: u16) -> Self {
        let [v0, v1] = version.to_le_bytes();
        let [c0, c1] = cat_id.to_le_bytes();
        let [_, _, _, _, b4, b5, b6, b7] = Self::CAT_PREFIX.to_le_bytes();
        Self(u64::from_le_bytes([v0, v1, c0, c1, b4, b5, b6, b7]))
    }

    /// Extract `(cat_id, version)` if this is a Case Authenticated Tag (CAT) Node ID (`0xFFFF_FFFD_xxxx_xxxx`).
    #[must_use]
    pub const fn to_cat(self) -> Option<(u16, u16)> {
        if self.is_case_authenticated_tag() {
            let [v0, v1, c0, c1, ..] = self.0.to_le_bytes();
            Some((u16::from_le_bytes([c0, c1]), u16::from_le_bytes([v0, v1])))
        } else {
            None
        }
    }

    /// Return `true` if this is a Case Authenticated Tag (CAT) Node ID (`0xFFFF_FFFD_xxxx_xxxx`) per §2.5.5.1.
    #[must_use]
    pub const fn is_case_authenticated_tag(self) -> bool {
        (self.0 >> 32) == 0xFFFF_FFFD
    }

    /// Construct a Group Node ID (`0xFFFF_FFFF_FFFF_0000 | group_id`) per CSA Matter Core Specification §2.5.5.1.
    #[must_use]
    pub const fn from_group(group_id: GroupId) -> Self {
        let [g0, g1] = group_id.raw().to_le_bytes();
        let [_, _, b2, b3, b4, b5, b6, b7] = Self::GROUP_PREFIX.to_le_bytes();
        Self(u64::from_le_bytes([g0, g1, b2, b3, b4, b5, b6, b7]))
    }

    /// Extract the [`GroupId`] if this is a Group Node ID (`0xFFFF_FFFF_FFFF_0000..=0xFFFF_FFFF_FFFF_FFFF`).
    #[must_use]
    pub const fn to_group(self) -> Option<GroupId> {
        if self.is_group() {
            let [b0, b1, ..] = self.0.to_le_bytes();
            Some(GroupId::new(u16::from_le_bytes([b0, b1])))
        } else {
            None
        }
    }

    /// Return `true` if this is a Group Node ID (`0xFFFF_FFFF_FFFF_0000..=0xFFFF_FFFF_FFFF_FFFF`)
    /// per CSA Matter Core Specification §2.5.5.1.
    #[must_use]
    pub const fn is_group(self) -> bool {
        self.0 >= Self::GROUP_PREFIX
    }

    /// Return `true` if this is an operational (unicast) node ID (`0x0000_0000_0000_0001..=0xFFFF_FFEF_FFFF_FFFF`)
    /// per CSA Matter Core Specification §2.5.5.1.
    #[must_use]
    pub const fn is_operational(self) -> bool {
        matches!(self.0, 0x0000_0000_0000_0001..=0xFFFF_FFEF_FFFF_FFFF)
    }
}

/// Stateful wire representation of a Matter Node per Section 7.2.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
#[non_exhaustive]
pub struct Node {
    /// Operational Matter node identifier.
    pub id: NodeId,
    /// Endpoints hosted by this node (including Root Node endpoint `0`).
    pub endpoints: Vec<Endpoint>,
}

impl Node {
    /// Construct a new node with its complete collection of endpoints.
    ///
    /// Callers should provide endpoints with unique [`EndpointId`]s; lookups and updates
    /// operate on the first matching ID.
    #[must_use]
    pub const fn new(id: NodeId, endpoints: Vec<Endpoint>) -> Self {
        Self { id, endpoints }
    }

    /// Construct an empty node with no endpoints.
    #[must_use]
    pub const fn empty(id: NodeId) -> Self {
        Self {
            id,
            endpoints: Vec::new(),
        }
    }

    /// Set or override the node identifier.
    #[must_use]
    pub const fn with_id(mut self, id: NodeId) -> Self {
        self.id = id;
        self
    }

    /// Add or update an endpoint on this node.
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<Endpoint>) -> Self {
        self.set_endpoint(endpoint);
        self
    }

    /// Add or update multiple endpoints on this node.
    #[must_use]
    pub fn with_endpoints(mut self, endpoints: impl IntoIterator<Item = Endpoint>) -> Self {
        for endpoint in endpoints {
            self.set_endpoint(endpoint);
        }
        self
    }

    /// Return `true` if this node hosts an endpoint with the given ID.
    #[must_use]
    pub fn has_endpoint(&self, id: EndpointId) -> bool {
        self.endpoints.iter().any(|ep| ep.id == id)
    }

    /// Find an endpoint on this node by endpoint ID.
    #[must_use]
    pub fn endpoint(&self, id: EndpointId) -> Option<&Endpoint> {
        self.endpoints.iter().find(|ep| ep.id == id)
    }

    /// Return a mutable reference to an endpoint on this node by endpoint ID.
    #[must_use]
    pub fn endpoint_mut(&mut self, id: EndpointId) -> Option<&mut Endpoint> {
        self.endpoints.iter_mut().find(|ep| ep.id == id)
    }

    /// Find the Root Node endpoint (`EndpointId::ROOT` / `0`) on this node.
    #[must_use]
    pub fn root_endpoint(&self) -> Option<&Endpoint> {
        self.endpoint(EndpointId::ROOT)
    }

    /// Return a mutable reference to the Root Node endpoint (`EndpointId::ROOT` / `0`) on this node.
    #[must_use]
    pub fn root_endpoint_mut(&mut self) -> Option<&mut Endpoint> {
        self.endpoint_mut(EndpointId::ROOT)
    }

    /// Insert or update an endpoint on this node in place, returning the previous endpoint if replaced.
    pub fn set_endpoint(&mut self, endpoint: impl Into<Endpoint>) -> Option<Endpoint> {
        let endpoint = endpoint.into();
        if let Some(existing) = self.endpoints.iter_mut().find(|ep| ep.id == endpoint.id) {
            Some(mem::replace(existing, endpoint))
        } else {
            self.endpoints.push(endpoint);
            None
        }
    }

    /// Remove an endpoint by ID from this node, returning it if present.
    pub fn remove_endpoint(&mut self, id: EndpointId) -> Option<Endpoint> {
        self.endpoints
            .iter()
            .position(|ep| ep.id == id)
            .map(|idx| self.endpoints.remove(idx))
    }

    /// Convenience lookup for an attribute value across `endpoint -> cluster -> attribute`.
    #[must_use]
    pub fn attribute(
        &self,
        endpoint_id: EndpointId,
        cluster_id: ClusterId,
        attribute_id: AttributeId,
    ) -> Option<&AttributeValue> {
        self.endpoint(endpoint_id)?.cluster(cluster_id)?.attribute(attribute_id)
    }

    /// Convenience mutable lookup for an attribute value across `endpoint -> cluster -> attribute`.
    #[must_use]
    pub fn attribute_mut(
        &mut self,
        endpoint_id: EndpointId,
        cluster_id: ClusterId,
        attribute_id: AttributeId,
    ) -> Option<&mut AttributeValue> {
        self.endpoint_mut(endpoint_id)?
            .cluster_mut(cluster_id)?
            .attribute_mut(attribute_id)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Node {} ({} endpoints)", self.id, self.endpoints.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cluster::Cluster, endpoint::DeviceTypeId};

    #[test]
    fn node_id_display_and_roundtrip() {
        assert_eq!(NodeId::UNSPECIFIED.raw(), 0);
        assert!(NodeId::UNSPECIFIED.is_unspecified());
        assert!(!NodeId::new(1).is_unspecified());
        assert_eq!(NodeId::new(0x1234_5678).raw(), 0x1234_5678);
        assert_eq!(alloc::format!("{}", NodeId::new(1)), "0x0000000000000001");
        assert_eq!(u64::from(NodeId::new(42)), 42);

        let val_node = AttributeValue::from(NodeId::new(99));
        assert_eq!(val_node, AttributeValue::U64(99));
        assert_eq!(NodeId::try_from(&val_node), Ok(NodeId::new(99)));
        assert_eq!(NodeId::try_from(val_node), Ok(NodeId::new(99)));

        let group_node = NodeId::from_group(GroupId::new(0x1234));
        assert!(group_node.is_group());
        assert!(!group_node.is_operational());
        assert_eq!(group_node.raw(), 0xFFFF_FFFF_FFFF_1234);
        assert_eq!(group_node.to_group(), Some(GroupId::new(0x1234)));
        assert_eq!(NodeId::new(1).to_group(), None);
        assert!(!NodeId::new(1).is_group());

        let cat_node = NodeId::from_cat(0xABCD, 0x0002);
        assert!(cat_node.is_case_authenticated_tag());
        assert!(!cat_node.is_operational());
        assert!(!cat_node.is_group());
        assert_eq!(cat_node.raw(), 0xFFFF_FFFD_ABCD_0002);
        assert_eq!(cat_node.to_cat(), Some((0xABCD, 0x0002)));
        assert_eq!(NodeId::new(1).to_cat(), None);
        assert_eq!("0x+10".parse::<NodeId>().ok(), None);
        assert_eq!("+10".parse::<NodeId>().ok(), None);
    }

    #[test]
    fn node_construction_and_lookups() {
        const ROOT_NODE_DEVICE_TYPE: DeviceTypeId = DeviceTypeId::new(0x0016);
        const ON_OFF_LIGHT_DEVICE_TYPE: DeviceTypeId = DeviceTypeId::new(0x0100);

        let node = Node::new(NodeId::new(1), alloc::vec![])
            .with_endpoint(Endpoint::single_device(
                EndpointId::ROOT,
                ROOT_NODE_DEVICE_TYPE,
                1,
                alloc::vec![],
            ))
            .with_endpoint(Endpoint::single_device(
                EndpointId::new(1),
                ON_OFF_LIGHT_DEVICE_TYPE,
                1,
                alloc::vec![],
            ));

        assert_eq!(node.id, NodeId::new(1));
        assert!(node.id.is_operational());
        assert!(!NodeId::UNSPECIFIED.is_operational());
        assert!(!NodeId::new(0xFFFF_FFFF_FFFF_0001).is_operational());
        assert!(!NodeId::new(0xFFFF_FFF0_0000_0000).is_operational());
        assert!(NodeId::new(0xFFFF_FFEF_FFFF_FFFF).is_operational());
        assert!(node.root_endpoint().is_some());
        assert_eq!(node.root_endpoint().unwrap().id, EndpointId::ROOT);
        assert!(node.endpoint(EndpointId::new(1)).is_some());
        assert!(node.endpoint(EndpointId::new(2)).is_none());

        let mut node = node;
        assert!(node.root_endpoint_mut().is_some());
        assert!(node.endpoint_mut(EndpointId::new(1)).is_some());
        assert!(node.remove_endpoint(EndpointId::new(1)).is_some());
        assert!(node.endpoint(EndpointId::new(1)).is_none());
    }

    #[test]
    fn remove_endpoint_preserves_order() {
        let mut node = Node::new(NodeId::new(1), alloc::vec![])
            .with_endpoint(Endpoint::empty(EndpointId::ROOT))
            .with_endpoint(Endpoint::empty(EndpointId::new(1)))
            .with_endpoint(Endpoint::empty(EndpointId::new(2)))
            .with_endpoint(Endpoint::empty(EndpointId::new(3)));

        assert!(node.remove_endpoint(EndpointId::new(1)).is_some());
        let ids: Vec<u16> = node.endpoints.iter().map(|ep| ep.id.raw()).collect();
        assert_eq!(ids, alloc::vec![0, 2, 3]);

        let mut empty = Node::empty(NodeId::new(0xABCD));
        assert!(!empty.has_endpoint(EndpointId::ROOT));
        empty.set_endpoint(
            Endpoint::empty(EndpointId::ROOT)
                .with_cluster(Cluster::empty(ClusterId::DESCRIPTOR).with_attribute(AttributeId::new(0), 1_u8)),
        );
        assert!(empty.has_endpoint(EndpointId::ROOT));
        if let Some(v) = empty.attribute_mut(EndpointId::ROOT, ClusterId::DESCRIPTOR, AttributeId::new(0)) {
            *v = AttributeValue::U8(7);
        }
        assert_eq!(
            empty.attribute(EndpointId::ROOT, ClusterId::DESCRIPTOR, AttributeId::new(0)),
            Some(&AttributeValue::U8(7))
        );
        assert_eq!(alloc::format!("{empty}"), "Node 0x000000000000abcd (1 endpoints)");
        assert_eq!("0xabcd".parse::<NodeId>(), Ok(NodeId::new(0xABCD)));
        assert_eq!(alloc::format!("{:x}", NodeId::new(0xABCD)), "abcd");
    }
}
