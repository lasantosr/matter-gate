//! Semantic marker and contract traits for Matter attributes.

mod endpoint;
mod wire_decode;

#[cfg(test)]
mod tests;

use alloc::{string::String, vec::Vec};
use core::fmt::Debug;

pub(crate) use endpoint::*;
pub(crate) use wire_decode::*;

use crate::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError},
    model::{
        Attribute, AttributeId, AttributeValue, Cluster, ClusterId, Command, CommandDirection, CommandId, DeviceTypeId,
        Endpoint, EndpointId, Event, EventId, EventPriority,
    },
    wire::{WireCluster, WireEvent},
};

/// Structural contract implemented by generated cluster state structs (`<Name>Cluster`).
pub trait ClusterSpec: Sized + TryFrom<Cluster, Error = ClusterError> + Into<Cluster> {
    /// Typed single-attribute enum (`Attribute`).
    type Attribute;
    /// Typed `ClientToServer` command enum (`Command`, or `()` if none).
    type Command;
    /// Typed `ServerToClient` command response enum (`CommandResponse`, or `()` if none).
    type CommandResponse;
    /// Typed event enum (`Event`, or `()` if none).
    type Event;
    /// Typed `FeatureMap` representation (`Features` or `u32`).
    type Features: Copy + Default + Into<u32> + TryFrom<u32>;
    /// Sparse/delta patch struct (`<Name>ClusterPatch`).
    type Patch: PartialCluster<Full = Self>;

    /// CSA cluster identifier.
    const CLUSTER_ID: ClusterId;
    /// CSA cluster revision for this specification release.
    const CLUSTER_REVISION: u16;
    /// Human-readable cluster name.
    const NAME: &'static str;
    /// Whether this cluster has no attributes (e.g. empty cluster struct).
    const IS_EMPTY: bool = false;

    /// Applies a sparse patch in place onto `self`.
    fn apply_patch(&mut self, patch: Self::Patch);

    /// Wraps `self` in a lossless [`WireCluster`] with typed [`Self::Features`](ClusterSpec::Features).
    #[inline]
    fn into_wire(self) -> WireCluster<Self, Self::Features> {
        WireCluster::new(self)
    }
}

/// Contract implemented by typed `ClientToServer` command request structs and `ServerToClient` response structs.
pub trait CommandSpec: Sized + TryFrom<Command, Error = CommandError> + Into<Command> {
    /// CSA cluster identifier.
    const CLUSTER_ID: ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    const CLUSTER_NAME: &'static str;
    /// CSA command identifier.
    const COMMAND_ID: CommandId;
    /// Canonical `PascalCase` Matter Command name.
    const COMMAND_NAME: &'static str;
    /// Interaction Model command direction.
    const DIRECTION: CommandDirection = CommandDirection::ClientToServer;
    /// Whether this command carries an empty/unit payload.
    const IS_EMPTY: bool = false;
    /// Expected response type (`()` for commands that return `DefaultResponse` and for `ServerToClient` responses).
    type Response;
}

/// Static identity and wire-conversion contract for a concrete Matter attribute.
pub trait AttributeSpec: Sized + TryFrom<Attribute, Error = AttributeError> + Into<Attribute> {
    /// Owning Matter Cluster ID.
    const CLUSTER_ID: ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    const CLUSTER_NAME: &'static str;
    /// Numeric Matter Attribute ID.
    const ATTRIBUTE_ID: AttributeId;
    /// Canonical `PascalCase` Matter Attribute name.
    const ATTRIBUTE_NAME: &'static str;
    /// Whether the attribute value is nullable per the Matter specification.
    const IS_NULLABLE: bool = false;
    /// Whether the attribute is optional on its cluster.
    const IS_OPTIONAL: bool = false;
    /// Whether the attribute is writable.
    const IS_WRITABLE: bool = false;
}

/// Static identity and wire-conversion contract for a concrete Matter event.
pub trait EventSpec: Sized + TryFrom<Event, Error = EventError> + Into<Event> {
    /// Owning Matter Cluster ID.
    const CLUSTER_ID: ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    const CLUSTER_NAME: &'static str;
    /// Numeric Matter Event ID.
    const EVENT_ID: EventId;
    /// Canonical `PascalCase` Matter Event name.
    const EVENT_NAME: &'static str;
    /// Interaction Model event priority.
    const PRIORITY: EventPriority = EventPriority::Info;
    /// Whether this event carries an empty/unit payload.
    const IS_EMPTY: bool = false;

    /// Wraps `self` in a lossless [`WireEvent`] with [`Self::PRIORITY`](EventSpec::PRIORITY).
    #[inline]
    fn into_wire(self) -> WireEvent<Self> {
        WireEvent::new(Self::PRIORITY, self)
    }
}

/// Contract implemented by sparse/delta cluster patch types (`<Name>ClusterPatch`).
pub trait PartialCluster: Sized + Default {
    /// Full cluster state type corresponding to this patch.
    type Full;

    /// Applies a single wire attribute `(AttributeId, AttributeValue)` to this patch.
    fn apply_attribute(&mut self, id: AttributeId, value: AttributeValue) -> Result<(), AttributeError>;

    /// Merges another patch into `self`, overwriting any fields present in `other`.
    fn merge(&mut self, other: Self);

    /// Attempts to materialize a full cluster state from this patch.
    fn try_into_full(self) -> Result<Self::Full, ClusterError>;

    /// Converts all present fields in this patch into `(AttributeId, AttributeValue)` pairs.
    fn into_attributes(self) -> Vec<(AttributeId, AttributeValue)>;
}

/// Structural contract implemented by generated device-type models (`v1_X::endpoints::<Name>`).
pub trait DeviceTypeSpec: Sized + TryFrom<Endpoint, Error = EndpointError> {
    /// Zero-allocation borrowed view over an [`Endpoint`] for this device type.
    type View<'a>;

    /// CSA device type identifier.
    const DEVICE_TYPE_ID: DeviceTypeId;
    /// CSA device type revision for this specification release.
    const REVISION: u16;
    /// Human-readable device type name.
    const NAME: &'static str;
    /// Whether this device type has an empty/unit payload.
    const IS_EMPTY: bool = false;

    /// Places this typed device on an [`Endpoint`] with the given `id`.
    #[must_use]
    fn into_endpoint(self, id: EndpointId) -> Endpoint;

    /// Constructs a zero-allocation borrowed view over `endpoint`.
    fn view(endpoint: &Endpoint) -> Result<Self::View<'_>, EndpointError>;
}

/// Extension trait for inspecting [`Endpoint`]s as typed Matter device types.
pub trait EndpointExt {
    /// Returns a zero-allocation borrowed view over `self` if `self` declares
    /// [`D::DEVICE_TYPE_ID`](DeviceTypeSpec::DEVICE_TYPE_ID).
    ///
    /// Returns `Ok(None)` when `D` is absent from Descriptor `DeviceTypeList` (`0x0000`), `Ok(Some(view))` when
    /// present and conformant, or `Err(e)` when `D` is listed on the endpoint but fails conformance checks.
    fn view_as<D: DeviceTypeSpec>(&self) -> Result<Option<D::View<'_>>, EndpointError>;
}

impl EndpointExt for Endpoint {
    #[inline]
    fn view_as<D: DeviceTypeSpec>(&self) -> Result<Option<D::View<'_>>, EndpointError> {
        if !self
            .has_device_type(D::DEVICE_TYPE_ID)
            .map_err(|status| EndpointError::descriptor_attr_status(0x0000, status))?
        {
            return Ok(None);
        }
        D::view(self).map(Some)
    }
}

/// Marker and contract trait for clusters derived from the Matter Mode Base cluster schema.
pub trait ModeBaseCluster: Sized {
    /// Associated strongly-typed enum for mode tags.
    type ModeTag: Into<Self::ModeTagStruct> + Copy + PartialEq + Debug;
    /// Associated wire struct representation of a mode tag.
    type ModeTagStruct;
    /// Associated wire struct representation of a mode option.
    type ModeOptionStruct: ModeOptionBuilder<Self::ModeTagStruct>;

    /// Construct a new mode base cluster instance with supported modes and current mode.
    fn new(supported_modes: Vec<Self::ModeOptionStruct>, current_mode: u8) -> Self;
}

/// Builder contract implemented by generated `ModeOptionStruct` types.
pub trait ModeOptionBuilder<TTag> {
    /// Construct a new mode option struct with label, mode number, and associated tags.
    fn build(label: String, mode: u8, mode_tags: Vec<TTag>) -> Self;
}
