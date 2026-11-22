//! Runtime Matter metadata traits.

use crate::{
    dispatch::{Described, Invocation},
    model::{AttributeId, ClusterId, CommandDirection, CommandId, DeviceTypeId, EndpointId, EventId, EventPriority},
    traits::{AttributeSpec, ClusterSpec, CommandSpec, DeviceTypeSpec, EventSpec},
};

/// Runtime metadata contract for Matter clusters and polymorphic cluster enums.
pub trait ClusterMeta {
    /// Numeric Matter Cluster ID.
    fn cluster_id(&self) -> ClusterId;
    /// Canonical `PascalCase` Matter Cluster name.
    fn cluster_name(&self) -> &'static str;
    /// Whether this cluster has an empty payload.
    fn is_empty_payload(&self) -> bool {
        false
    }
}

impl<T: ClusterSpec> ClusterMeta for T {
    #[inline]
    fn cluster_id(&self) -> ClusterId {
        T::CLUSTER_ID
    }

    #[inline]
    fn cluster_name(&self) -> &'static str {
        T::NAME
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        T::IS_EMPTY
    }
}

/// Runtime metadata contract for Matter commands and polymorphic command enums.
pub trait CommandMeta {
    /// Owning Matter Cluster ID.
    fn cluster_id(&self) -> ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    fn cluster_name(&self) -> &'static str;
    /// Numeric Matter Command ID.
    fn command_id(&self) -> CommandId;
    /// Canonical `PascalCase` Matter Command name.
    fn command_name(&self) -> &'static str;
    /// Interaction Model command direction.
    fn direction(&self) -> CommandDirection {
        CommandDirection::ClientToServer
    }
    /// Whether this command carries an empty/unit payload.
    fn is_empty_payload(&self) -> bool {
        false
    }

    /// Wraps `self` in an [`Invocation`] targeting `endpoint_id`.
    #[inline]
    fn invoke_on(self, endpoint_id: EndpointId) -> Invocation<Self>
    where
        Self: Sized,
    {
        Invocation::new(endpoint_id, self)
    }
}

impl<T: CommandSpec> CommandMeta for T {
    #[inline]
    fn cluster_id(&self) -> ClusterId {
        T::CLUSTER_ID
    }

    #[inline]
    fn cluster_name(&self) -> &'static str {
        T::CLUSTER_NAME
    }

    #[inline]
    fn command_id(&self) -> CommandId {
        T::COMMAND_ID
    }

    #[inline]
    fn command_name(&self) -> &'static str {
        T::COMMAND_NAME
    }

    #[inline]
    fn direction(&self) -> CommandDirection {
        T::DIRECTION
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        T::IS_EMPTY
    }
}

/// Runtime metadata contract for Matter attributes and polymorphic attribute enums.
pub trait AttributeMeta {
    /// Owning Matter Cluster ID.
    fn cluster_id(&self) -> ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    fn cluster_name(&self) -> &'static str;
    /// Numeric Matter Attribute ID.
    fn attribute_id(&self) -> AttributeId;
    /// Canonical `PascalCase` Matter Attribute name.
    fn attribute_name(&self) -> &'static str;
}

impl<T: AttributeSpec> AttributeMeta for T {
    #[inline]
    fn cluster_id(&self) -> ClusterId {
        T::CLUSTER_ID
    }

    #[inline]
    fn cluster_name(&self) -> &'static str {
        T::CLUSTER_NAME
    }

    #[inline]
    fn attribute_id(&self) -> AttributeId {
        T::ATTRIBUTE_ID
    }

    #[inline]
    fn attribute_name(&self) -> &'static str {
        T::ATTRIBUTE_NAME
    }
}

/// Runtime metadata contract for Matter events and polymorphic event enums.
pub trait EventMeta {
    /// Owning Matter Cluster ID.
    fn cluster_id(&self) -> ClusterId;
    /// Canonical `PascalCase` owning Matter Cluster name.
    fn cluster_name(&self) -> &'static str;
    /// Numeric Matter Event ID.
    fn event_id(&self) -> EventId;
    /// Canonical `PascalCase` Matter Event name.
    fn event_name(&self) -> &'static str;
    /// Interaction Model event priority.
    fn priority(&self) -> EventPriority {
        EventPriority::Info
    }
    /// Whether this event carries an empty/unit payload.
    fn is_empty_payload(&self) -> bool {
        false
    }
}

impl<T: EventSpec> EventMeta for T {
    #[inline]
    fn cluster_id(&self) -> ClusterId {
        T::CLUSTER_ID
    }

    #[inline]
    fn cluster_name(&self) -> &'static str {
        T::CLUSTER_NAME
    }

    #[inline]
    fn event_id(&self) -> EventId {
        T::EVENT_ID
    }

    #[inline]
    fn event_name(&self) -> &'static str {
        T::EVENT_NAME
    }

    #[inline]
    fn priority(&self) -> EventPriority {
        T::PRIORITY
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        T::IS_EMPTY
    }
}

/// Runtime metadata contract for Matter device types and polymorphic device enums.
pub trait DeviceMeta {
    /// Numeric Matter Device Type ID.
    fn device_type_id(&self) -> DeviceTypeId;
    /// Canonical `PascalCase` Matter Device Type name.
    fn device_type_name(&self) -> &'static str;
    /// Device Type revision number.
    fn revision(&self) -> u16;
    /// Whether this device type has an empty payload.
    fn is_empty_payload(&self) -> bool {
        false
    }
}

impl<T: DeviceTypeSpec> DeviceMeta for T {
    #[inline]
    fn device_type_id(&self) -> DeviceTypeId {
        T::DEVICE_TYPE_ID
    }

    #[inline]
    fn device_type_name(&self) -> &'static str {
        T::NAME
    }

    #[inline]
    fn revision(&self) -> u16 {
        T::REVISION
    }

    #[inline]
    fn is_empty_payload(&self) -> bool {
        T::IS_EMPTY
    }
}

/// Discriminator kind for a self-describing Matter entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    /// A Matter cluster.
    Cluster,
    /// A Matter cluster command.
    Command,
    /// A Matter cluster attribute.
    Attribute,
    /// A Matter cluster event.
    Event,
    /// A Matter device type / endpoint.
    Device,
}

/// Runtime identity descriptor emitted by [`MatterDescribe::descriptor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityDescriptor {
    /// Cluster identity (`"cluster"`, `"cluster_id"`).
    Cluster {
        /// Canonical Matter cluster name.
        cluster: &'static str,
        /// Numeric 32-bit Matter cluster ID.
        cluster_id: u32,
    },
    /// Command identity (`"cluster"`, `"cluster_id"`, `"command"`, `"command_id"`).
    Command {
        /// Canonical Matter cluster name.
        cluster: &'static str,
        /// Numeric 32-bit Matter cluster ID.
        cluster_id: u32,
        /// Canonical Matter command name.
        command: &'static str,
        /// Numeric 32-bit Matter command ID.
        command_id: u32,
    },
    /// Attribute identity (`"cluster"`, `"cluster_id"`, `"attribute"`, `"attribute_id"`).
    Attribute {
        /// Canonical Matter cluster name.
        cluster: &'static str,
        /// Numeric 32-bit Matter cluster ID.
        cluster_id: u32,
        /// Canonical Matter attribute name.
        attribute: &'static str,
        /// Numeric 32-bit Matter attribute ID.
        attribute_id: u32,
    },
    /// Event identity (`"cluster"`, `"cluster_id"`, `"event"`, `"event_id"`).
    Event {
        /// Canonical Matter cluster name.
        cluster: &'static str,
        /// Numeric 32-bit Matter cluster ID.
        cluster_id: u32,
        /// Canonical Matter event name.
        event: &'static str,
        /// Numeric 32-bit Matter event ID.
        event_id: u32,
    },
    /// Device Type identity (`"device_type"`, `"device_type_id"`).
    Device {
        /// Canonical Matter device type name.
        device_type: &'static str,
        /// Numeric 32-bit Matter device type ID.
        device_type_id: u32,
    },
}

impl EntityDescriptor {
    /// Returns the [`EntityKind`] for this descriptor.
    #[must_use]
    pub const fn kind(&self) -> EntityKind {
        match self {
            Self::Cluster { .. } => EntityKind::Cluster,
            Self::Command { .. } => EntityKind::Command,
            Self::Attribute { .. } => EntityKind::Attribute,
            Self::Event { .. } => EntityKind::Event,
            Self::Device { .. } => EntityKind::Device,
        }
    }

    /// Returns the primary scope `(name, id)` (`cluster` for cluster-scoped entities, `device_type` for devices).
    #[must_use]
    pub const fn scope(&self) -> (&'static str, u32) {
        match *self {
            Self::Cluster { cluster, cluster_id }
            | Self::Command {
                cluster, cluster_id, ..
            }
            | Self::Attribute {
                cluster, cluster_id, ..
            }
            | Self::Event {
                cluster, cluster_id, ..
            } => (cluster, cluster_id),
            Self::Device {
                device_type,
                device_type_id,
            } => (device_type, device_type_id),
        }
    }

    /// Returns the secondary item `(name, id)` for commands, attributes, and events, or `None` for clusters/devices.
    #[must_use]
    pub const fn item(&self) -> Option<(&'static str, u32)> {
        match *self {
            Self::Command {
                command, command_id, ..
            } => Some((command, command_id)),
            Self::Attribute {
                attribute,
                attribute_id,
                ..
            } => Some((attribute, attribute_id)),
            Self::Event { event, event_id, .. } => Some((event, event_id)),
            Self::Cluster { .. } | Self::Device { .. } => None,
        }
    }
}

/// Compile-time schema metadata used by [`Described<T>`](crate::dispatch::Described) and
/// [`Invocation<T>`](crate::dispatch::Invocation) when generating `schemars` JSON schemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StaticEntitySchema {
    /// Entity kind.
    pub kind: EntityKind,
    /// Statically known `(name, id)` for the cluster (or device type), if fixed for all values of `Self`.
    pub scope: Option<(&'static str, u32)>,
    /// Statically known `(name, id)` for the command/attribute/event, if fixed for all values of `Self`.
    pub item: Option<(&'static str, u32)>,
    /// Whether the payload is statically known to be empty for all values of `Self`.
    pub is_empty: bool,
}

impl StaticEntitySchema {
    /// Build a [`StaticEntitySchema`] for a concrete type from its [`EntityDescriptor`].
    #[must_use]
    pub const fn concrete(descriptor: EntityDescriptor, is_empty: bool) -> Self {
        Self {
            kind: descriptor.kind(),
            scope: Some(descriptor.scope()),
            item: descriptor.item(),
            is_empty,
        }
    }

    /// Build a [`StaticEntitySchema`] for a single-cluster polymorphic enum.
    #[must_use]
    pub const fn cluster_enum(kind: EntityKind, cluster: &'static str, cluster_id: u32) -> Self {
        Self {
            kind,
            scope: Some((cluster, cluster_id)),
            item: None,
            is_empty: false,
        }
    }

    /// Build a [`StaticEntitySchema`] for a cross-cluster polymorphic enum.
    #[must_use]
    pub const fn open_enum(kind: EntityKind) -> Self {
        Self {
            kind,
            scope: None,
            item: None,
            is_empty: false,
        }
    }
}

/// Unified self-description contract powering [`Described<T>`](crate::dispatch::Described) and
/// [`Invocation<T>`](crate::dispatch::Invocation) across both concrete types and polymorphic enums.
pub trait MatterDescribe {
    /// Compile-time schema metadata for `JsonSchema` generation.
    const STATIC_SCHEMA: StaticEntitySchema;

    /// Runtime metadata descriptor for serialization.
    fn descriptor(&self) -> EntityDescriptor;

    /// Whether this value has an empty/unit payload (omitting `"payload"` on serialize).
    fn is_empty_payload(&self) -> bool {
        false
    }

    /// Wraps `self` in a self-describing JSON wrapper [`Described<Self>`](crate::dispatch::Described).
    #[inline]
    fn described(self) -> Described<Self>
    where
        Self: Sized,
    {
        Described::new(self)
    }

    /// Serialize only the inner payload (without outer enum tag wrappers) for
    /// [`Described<Self>`](crate::dispatch::Described) and [`Invocation<Self>`](crate::dispatch::Invocation).
    #[cfg(feature = "serde")]
    fn serialize_payload<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        Self: serde::Serialize,
        S: serde::Serializer,
    {
        self.serialize(serializer)
    }

    /// Deserialize a [`Described<Self>`](crate::dispatch::Described) envelope.
    #[cfg(feature = "serde")]
    fn deserialize_described<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        Self: Sized,
        D: serde::Deserializer<'de>;

    /// Deserialize an [`Invocation<Self>`](crate::dispatch::Invocation) envelope (commands only).
    #[cfg(feature = "serde")]
    fn deserialize_invocation<'de, D>(_deserializer: D) -> Result<(EndpointId, Self), D::Error>
    where
        Self: Sized + CommandMeta,
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;
        Err(D::Error::custom(
            "Invocation deserialization is only supported for command types",
        ))
    }
}
