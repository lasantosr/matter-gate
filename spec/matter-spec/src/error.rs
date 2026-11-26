//! Exhaustive specification conformance and wire decoding errors.
//!
//! # Exhaustiveness & `SemVer` Policy
//!
//! All error enums in this module intentionally omit `#[non_exhaustive]` so upgrades that introduce new
//! failure modes surface as compile-time notifications. Downstream callers who prefer forward-compatible
//! matching can include a wildcard `_ =>` arm or map any error to a wire code via [`MatterStatus::from`].

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::{
    AttributeId, ClusterId, CommandDirection, CommandId, DeviceTypeId, EndpointId, EventId, EventPriority, FieldId,
    MatterStatus,
};

/// Error returned when decoding or validating an [`Attribute`](crate::model::Attribute).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub enum AttributeError {
    /// Cluster ID is not supported or registered.
    #[error("Unsupported cluster {0}")]
    UnsupportedCluster(ClusterId),
    /// The attribute's cluster ID did not match the expected cluster schema.
    #[error("Cluster ID mismatch: expected {expected}, got {actual}")]
    ClusterIdMismatch {
        /// Cluster ID expected by the target type.
        expected: ClusterId,
        /// Cluster ID found on the wire attribute.
        actual: ClusterId,
    },
    /// Attribute ID is not supported by the target cluster.
    #[error("Unsupported attribute {0}")]
    UnsupportedAttribute(AttributeId),
    /// Wire [`Attribute::id`](crate::model::Attribute::id) does not match the expected concrete attribute ID.
    #[error("Attribute ID mismatch: expected {expected}, got {actual}")]
    AttributeIdMismatch {
        /// Attribute ID expected by the target type.
        expected: AttributeId,
        /// Attribute ID found on the wire attribute.
        actual: AttributeId,
    },
    /// Attribute value failed data type or constraint validation.
    #[error("Invalid value for attribute {id}: {status}")]
    InvalidValue {
        /// Attribute ID whose value failed validation.
        id: AttributeId,
        /// Innermost struct field ID where validation failed, if applicable.
        field_id: Option<FieldId>,
        /// Matter status code describing the validation failure.
        #[source]
        status: MatterStatus,
    },
}

impl From<AttributeError> for MatterStatus {
    fn from(err: AttributeError) -> Self {
        match err {
            AttributeError::UnsupportedCluster(_) | AttributeError::ClusterIdMismatch { .. } => {
                Self::UnsupportedCluster
            }
            AttributeError::UnsupportedAttribute(_) | AttributeError::AttributeIdMismatch { .. } => {
                Self::UnsupportedAttribute
            }
            AttributeError::InvalidValue { status, .. } => status,
        }
    }
}

/// Error returned when decoding or validating a [`Command`](crate::model::Command).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub enum CommandError {
    /// Cluster ID is not supported.
    #[error("Unsupported cluster {0}")]
    UnsupportedCluster(ClusterId),
    /// Wire [`Command::cluster_id`](crate::model::Command::cluster_id) does not match the expected cluster ID.
    #[error("Cluster ID mismatch: expected {expected}, got {actual}")]
    ClusterIdMismatch {
        /// Cluster ID expected by the target type.
        expected: ClusterId,
        /// Cluster ID found on the wire command.
        actual: ClusterId,
    },
    /// Command ID is not supported on this cluster.
    #[error("Unsupported command {0}")]
    UnsupportedCommand(CommandId),
    /// Wire [`Command::command_id`](crate::model::Command::command_id) does not match the expected command ID.
    #[error("Command ID mismatch: expected {expected}, got {actual}")]
    CommandIdMismatch {
        /// Command ID expected by the target type.
        expected: CommandId,
        /// Command ID found on the wire command.
        actual: CommandId,
    },
    /// Wire [`Command::direction`](crate::model::Command::direction) does not match the expected command direction.
    #[error("Direction mismatch: expected {expected}, got {actual}")]
    DirectionMismatch {
        /// Command direction expected by the target type.
        expected: CommandDirection,
        /// Command direction found on the wire command.
        actual: CommandDirection,
    },
    /// Command payload failed data type or field constraint validation.
    #[error("Invalid payload for command {id}: {status}")]
    InvalidPayload {
        /// Command ID whose payload failed validation.
        id: CommandId,
        /// Innermost struct field ID where validation failed, if applicable.
        field_id: Option<FieldId>,
        /// Matter status code describing the validation failure.
        #[source]
        status: MatterStatus,
    },
}

impl From<CommandError> for MatterStatus {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::UnsupportedCluster(_) | CommandError::ClusterIdMismatch { .. } => Self::UnsupportedCluster,
            CommandError::UnsupportedCommand(_)
            | CommandError::CommandIdMismatch { .. }
            | CommandError::DirectionMismatch { .. } => Self::UnsupportedCommand,
            CommandError::InvalidPayload { status, .. } => status,
        }
    }
}

/// Error returned when decoding or validating an [`Event`](crate::model::Event).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub enum EventError {
    /// Cluster ID is not supported.
    #[error("Unsupported cluster {0}")]
    UnsupportedCluster(ClusterId),
    /// Wire [`Event::cluster_id`](crate::model::Event::cluster_id) does not match the expected cluster ID.
    #[error("Cluster ID mismatch: expected {expected}, got {actual}")]
    ClusterIdMismatch {
        /// Cluster ID expected by the target type.
        expected: ClusterId,
        /// Cluster ID found on the wire event.
        actual: ClusterId,
    },
    /// Event ID is not supported on this cluster.
    #[error("Unsupported event {0}")]
    UnsupportedEvent(EventId),
    /// Wire [`Event::event_id`](crate::model::Event::event_id) does not match the expected event ID.
    #[error("Event ID mismatch: expected {expected}, got {actual}")]
    EventIdMismatch {
        /// Event ID expected by the target type.
        expected: EventId,
        /// Event ID found on the wire event.
        actual: EventId,
    },
    /// Wire [`Event::priority`](crate::model::Event::priority) does not match the expected event priority.
    #[error("Event priority mismatch: expected {expected}, got {actual}")]
    PriorityMismatch {
        /// Priority expected by the target event schema.
        expected: EventPriority,
        /// Priority found on the wire event.
        actual: EventPriority,
    },
    /// Event payload failed data type or field constraint validation.
    #[error("Invalid payload for event {id}: {status}")]
    InvalidPayload {
        /// Event ID whose payload failed validation.
        id: EventId,
        /// Innermost struct field ID where validation failed, if applicable.
        field_id: Option<FieldId>,
        /// Matter status code describing the validation failure.
        #[source]
        status: MatterStatus,
    },
}

impl From<EventError> for MatterStatus {
    fn from(err: EventError) -> Self {
        match err {
            EventError::UnsupportedCluster(_) | EventError::ClusterIdMismatch { .. } => Self::UnsupportedCluster,
            EventError::UnsupportedEvent(_) | EventError::EventIdMismatch { .. } => Self::UnsupportedEvent,
            EventError::PriorityMismatch { .. } => Self::InvalidDataType,
            EventError::InvalidPayload { status, .. } => status,
        }
    }
}

/// Error returned when decoding or validating a [`Cluster`](crate::model::Cluster).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub enum ClusterError {
    /// Cluster ID is not supported or registered.
    #[error("Unsupported cluster {0}")]
    UnsupportedCluster(ClusterId),
    /// Wire [`Cluster::id`](crate::model::Cluster::id) does not match the expected concrete cluster ID.
    #[error("Cluster ID mismatch: expected {expected}, got {actual}")]
    ClusterIdMismatch {
        /// Cluster ID expected by the target type.
        expected: ClusterId,
        /// Cluster ID found on the wire cluster.
        actual: ClusterId,
    },
    /// A mandatory attribute required by the cluster schema is missing.
    #[error("Missing mandatory attribute {0}")]
    MissingMandatoryAttribute(AttributeId),
    /// An attribute on the cluster failed decoding or validation.
    #[error("Invalid attribute in cluster")]
    InvalidAttribute(#[from] AttributeError),
}

impl From<ClusterError> for MatterStatus {
    fn from(err: ClusterError) -> Self {
        match err {
            ClusterError::UnsupportedCluster(_) | ClusterError::ClusterIdMismatch { .. } => Self::UnsupportedCluster,
            ClusterError::MissingMandatoryAttribute(_) => Self::ConstraintError,
            ClusterError::InvalidAttribute(err) => Self::from(err),
        }
    }
}

/// Error returned when decoding or validating an [`Endpoint`](crate::model::Endpoint) against a device type
/// specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub enum EndpointError {
    /// Endpoint carries an invalid [`EndpointId`] (for example, wildcard `0xFFFF`).
    #[error("Invalid endpoint ID {0}")]
    InvalidEndpointId(EndpointId),
    /// Endpoint has an empty `device_types` list.
    #[error("Endpoint contains no device types")]
    EmptyDeviceTypes,
    /// Device Type ID is not supported or registered.
    #[error("Unsupported device type {0}")]
    UnsupportedDeviceType(DeviceTypeId),
    /// Endpoint does not declare the expected [`DeviceTypeId`].
    #[error("Device type mismatch: expected {expected}, got {actual}")]
    DeviceTypeMismatch {
        /// Device type ID expected by the endpoint schema.
        expected: DeviceTypeId,
        /// Device type ID found on the wire endpoint.
        actual: DeviceTypeId,
    },
    /// Endpoint is missing a mandatory server cluster required by the device type.
    #[error("Missing mandatory server cluster {0}")]
    MissingMandatoryServerCluster(ClusterId),
    /// Endpoint is missing a mandatory client cluster required by the device type.
    #[error("Missing mandatory client cluster {0}")]
    MissingMandatoryClientCluster(ClusterId),
    /// Endpoint contains duplicate entries for the same [`DeviceTypeId`].
    #[error("Duplicate device type {0} on endpoint")]
    DuplicateDeviceType(DeviceTypeId),
    /// Endpoint contains duplicate entries for the same [`ClusterId`].
    #[error("Duplicate cluster {0} on endpoint")]
    DuplicateCluster(ClusterId),
    /// Endpoint contains a cluster that is disallowed for this device type.
    #[error("Cluster {0} is disallowed on device type")]
    DisallowedCluster(ClusterId),
    /// A server cluster on the endpoint failed [`Cluster`](crate::model::Cluster) validation.
    #[error("Invalid cluster {id}")]
    InvalidCluster {
        /// Cluster ID that failed validation.
        id: ClusterId,
        /// Underlying cluster validation error.
        #[source]
        source: ClusterError,
    },
}

impl EndpointError {
    /// Builds an [`EndpointError::InvalidCluster`] for a Descriptor (`0x001D`) attribute failure.
    #[must_use]
    pub(crate) const fn descriptor_attr_status(attr_id: u32, status: MatterStatus) -> Self {
        Self::InvalidCluster {
            id: ClusterId::DESCRIPTOR,
            source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: AttributeId::new(attr_id),
                field_id: None,
                status,
            }),
        }
    }
}

impl From<EndpointError> for MatterStatus {
    fn from(err: EndpointError) -> Self {
        match err {
            EndpointError::InvalidEndpointId(_) => Self::UnsupportedEndpoint,
            EndpointError::EmptyDeviceTypes
            | EndpointError::UnsupportedDeviceType(_)
            | EndpointError::DeviceTypeMismatch { .. }
            | EndpointError::MissingMandatoryServerCluster(_)
            | EndpointError::MissingMandatoryClientCluster(_) => Self::UnsupportedCluster,
            EndpointError::DuplicateDeviceType(_)
            | EndpointError::DuplicateCluster(_)
            | EndpointError::DisallowedCluster(..) => Self::ConstraintError,
            EndpointError::InvalidCluster { source, .. } => Self::from(source),
        }
    }
}

/// Error returned when validating a [`Node`](crate::model::Node).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(all(feature = "serde", feature = "deny-unknown-fields"), serde(deny_unknown_fields))]
#[cfg_attr(
    all(feature = "schemars", feature = "deny-unknown-fields"),
    schemars(deny_unknown_fields)
)]
pub enum NodeError {
    /// Node is missing the mandatory Root Node endpoint (`EndpointId::ROOT` / `0`).
    #[error("Node is missing mandatory root endpoint (0)")]
    MissingRootEndpoint,
    /// Multiple endpoints on the node share the same [`EndpointId`].
    #[error("Duplicate endpoint {0} on node")]
    DuplicateEndpoint(EndpointId),
    /// An endpoint on the node failed validation.
    #[error("Invalid endpoint {id}")]
    InvalidEndpoint {
        /// Endpoint ID that failed validation.
        id: EndpointId,
        /// Underlying endpoint validation error.
        #[source]
        source: EndpointError,
    },
}

impl From<NodeError> for MatterStatus {
    fn from(err: NodeError) -> Self {
        match err {
            NodeError::MissingRootEndpoint => Self::UnsupportedEndpoint,
            NodeError::DuplicateEndpoint(_) => Self::ConstraintError,
            NodeError::InvalidEndpoint { source, .. } => Self::from(source),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::format;

    use super::*;

    #[test]
    fn exhaustive_pattern_matching_without_wildcard() {
        let attr_err = AttributeError::UnsupportedAttribute(AttributeId::new(1));
        match attr_err {
            AttributeError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            AttributeError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            AttributeError::UnsupportedAttribute(id) => assert_eq!(id, AttributeId::new(1)),
            AttributeError::AttributeIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            AttributeError::InvalidValue { id, field_id, status } => {
                assert_eq!(
                    (id, field_id, status),
                    (AttributeId::new(0), None, MatterStatus::Success)
                );
            }
        }

        let cmd_err = CommandError::UnsupportedCommand(CommandId::new(2));
        match cmd_err {
            CommandError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            CommandError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            CommandError::UnsupportedCommand(id) => assert_eq!(id, CommandId::new(2)),
            CommandError::CommandIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            CommandError::DirectionMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            CommandError::InvalidPayload { id, field_id, status } => {
                assert_eq!((id, field_id, status), (CommandId::new(0), None, MatterStatus::Success));
            }
        }

        let ev_err = EventError::UnsupportedEvent(EventId::new(3));
        match ev_err {
            EventError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            EventError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            EventError::UnsupportedEvent(id) => assert_eq!(id, EventId::new(3)),
            EventError::EventIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            EventError::PriorityMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            EventError::InvalidPayload { id, field_id, status } => {
                assert_eq!((id, field_id, status), (EventId::new(0), None, MatterStatus::Success));
            }
        }

        let cluster_err = ClusterError::MissingMandatoryAttribute(AttributeId::new(4));
        match cluster_err {
            ClusterError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            ClusterError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            ClusterError::MissingMandatoryAttribute(id) => assert_eq!(id, AttributeId::new(4)),
            ClusterError::InvalidAttribute(err) => {
                assert_eq!(err, AttributeError::UnsupportedAttribute(AttributeId::new(0)));
            }
        }

        let ep_err = EndpointError::EmptyDeviceTypes;
        match ep_err {
            EndpointError::InvalidEndpointId(id) => {
                assert_eq!(id, EndpointId::WILDCARD);
            }
            EndpointError::EmptyDeviceTypes => {}
            EndpointError::UnsupportedDeviceType(id) => {
                assert_eq!(id, DeviceTypeId::new(0));
            }
            EndpointError::DeviceTypeMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            EndpointError::MissingMandatoryServerCluster(id) => {
                assert_eq!(id, ClusterId::new(1));
            }
            EndpointError::MissingMandatoryClientCluster(id) => {
                assert_eq!(id, ClusterId::new(2));
            }
            EndpointError::DuplicateDeviceType(id) => {
                assert_eq!(id, DeviceTypeId::new(4));
            }
            EndpointError::DuplicateCluster(id) => {
                assert_eq!(id, ClusterId::new(5));
            }
            EndpointError::DisallowedCluster(id) => {
                assert_eq!(id, ClusterId::new(3));
            }
            EndpointError::InvalidCluster { id, source } => {
                assert_eq!(
                    (id, source),
                    (ClusterId::new(0), ClusterError::UnsupportedCluster(ClusterId::new(0)))
                );
            }
        }

        let node_err = NodeError::MissingRootEndpoint;
        match node_err {
            NodeError::MissingRootEndpoint => {}
            NodeError::DuplicateEndpoint(id) => {
                assert_eq!(id, EndpointId::new(0));
            }
            NodeError::InvalidEndpoint { id, source } => {
                assert_eq!((id, source), (EndpointId::new(0), EndpointError::EmptyDeviceTypes));
            }
        }
    }

    #[test]
    fn error_display_and_status() {
        let attr_cluster_err = AttributeError::UnsupportedCluster(ClusterId::new(6));
        assert_eq!(
            MatterStatus::from(attr_cluster_err.clone()),
            MatterStatus::UnsupportedCluster
        );
        assert_eq!(format!("{attr_cluster_err}"), "Unsupported cluster 0x0006");

        let attr_err = AttributeError::UnsupportedAttribute(AttributeId::new(1));
        assert_eq!(MatterStatus::from(attr_err.clone()), MatterStatus::UnsupportedAttribute);
        assert_eq!(format!("{attr_err}"), "Unsupported attribute 0x0001");

        let cmd_err = CommandError::UnsupportedCommand(CommandId::new(2));
        assert_eq!(MatterStatus::from(cmd_err.clone()), MatterStatus::UnsupportedCommand);
        assert_eq!(format!("{cmd_err}"), "Unsupported command 0x02");

        let ev_prio_err = EventError::PriorityMismatch {
            expected: EventPriority::Critical,
            actual: EventPriority::Info,
        };
        assert_eq!(MatterStatus::from(ev_prio_err.clone()), MatterStatus::InvalidDataType);
        assert_eq!(
            format!("{ev_prio_err}"),
            "Event priority mismatch: expected Critical, got Info"
        );

        let ev_err = EventError::UnsupportedEvent(EventId::new(3));
        assert_eq!(MatterStatus::from(ev_err.clone()), MatterStatus::UnsupportedEvent);
        assert_eq!(format!("{ev_err}"), "Unsupported event 0x03");

        let cl_err = ClusterError::MissingMandatoryAttribute(AttributeId::new(5));
        assert_eq!(MatterStatus::from(cl_err.clone()), MatterStatus::ConstraintError);
        assert_eq!(format!("{cl_err}"), "Missing mandatory attribute 0x0005");

        let ep_id_err = EndpointError::InvalidEndpointId(EndpointId::WILDCARD);
        assert_eq!(MatterStatus::from(ep_id_err.clone()), MatterStatus::UnsupportedEndpoint);
        assert_eq!(format!("{ep_id_err}"), "Invalid endpoint ID 65535");

        let ep_err = EndpointError::EmptyDeviceTypes;
        assert_eq!(MatterStatus::from(ep_err.clone()), MatterStatus::UnsupportedCluster);
        assert_eq!(format!("{ep_err}"), "Endpoint contains no device types");

        let node_err = NodeError::MissingRootEndpoint;
        assert_eq!(MatterStatus::from(node_err.clone()), MatterStatus::UnsupportedEndpoint);
        assert_eq!(format!("{node_err}"), "Node is missing mandatory root endpoint (0)");
    }
}
