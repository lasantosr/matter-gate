//! Version-independent unit & behavioral integration tests for traits, decoders, extensions, and errors.

use matter_spec::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError, NodeError},
    model::{
        AttributeId, ClusterId, CommandDirection, CommandId, DeviceTypeId, EndpointId, EventId, EventPriority,
        MatterStatus, Nullable,
    },
    types::Temperature,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_extension_roundtrip() {
        let room = Temperature::from_celsius(21.5);
        assert_eq!(room.raw(), 2150);
        assert!((room.to_celsius() - 21.5).abs() < 0.01);
        let fahrenheit = room.to_fahrenheit();
        assert!((fahrenheit - 70.7).abs() < 0.1);

        let subzero = Temperature::from_celsius(-10.5);
        assert_eq!(subzero.raw(), -1050);
        assert!((subzero.to_celsius() - (-10.5)).abs() < 0.01);
        let subzero_f = subzero.to_fahrenheit();
        assert!((subzero_f - 13.1).abs() < 0.1);

        let null_temp: Nullable<Temperature> = Nullable::Null;
        assert_eq!(null_temp.map(Temperature::to_celsius), Nullable::Null);
        assert_eq!(null_temp.map(Temperature::to_fahrenheit), Nullable::Null);
    }

    #[test]
    fn exhaustive_attribute_error_matching() {
        let err = AttributeError::AttributeIdMismatch {
            expected: AttributeId::new(1),
            actual: AttributeId::new(2),
        };
        match err {
            AttributeError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            AttributeError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            AttributeError::UnsupportedAttribute(id) => {
                assert_eq!(id, AttributeId::new(0));
            }
            AttributeError::AttributeIdMismatch { expected, actual } => {
                assert_eq!(expected, AttributeId::new(1));
                assert_eq!(actual, AttributeId::new(2));
            }
            AttributeError::InvalidValue { id, field_id, status } => {
                assert_eq!(
                    (id, field_id, status),
                    (AttributeId::new(0), None, MatterStatus::Success)
                );
            }
        }
        assert_eq!(
            MatterStatus::from(AttributeError::UnsupportedCluster(ClusterId::new(99))),
            MatterStatus::UnsupportedCluster
        );
    }

    #[test]
    fn error_source_chains_and_cluster_from_attribute_error() {
        use core::error::Error;

        let attr_err = AttributeError::InvalidValue {
            id: AttributeId::new(1),
            field_id: None,
            status: MatterStatus::ConstraintError,
        };
        assert_eq!(
            attr_err.source().and_then(|s| s.downcast_ref::<MatterStatus>()),
            Some(&MatterStatus::ConstraintError)
        );
        assert!(
            AttributeError::UnsupportedAttribute(AttributeId::new(1))
                .source()
                .is_none()
        );
        assert!(
            AttributeError::ClusterIdMismatch {
                expected: ClusterId::new(6),
                actual: ClusterId::new(8),
            }
            .source()
            .is_none()
        );

        let cluster_err = ClusterError::from(attr_err.clone());
        assert_eq!(cluster_err, ClusterError::InvalidAttribute(attr_err.clone()));
        assert_eq!(
            cluster_err.source().and_then(|s| s.downcast_ref::<AttributeError>()),
            Some(&attr_err)
        );
        assert!(
            ClusterError::MissingMandatoryAttribute(AttributeId::new(1))
                .source()
                .is_none()
        );

        let ep_err = EndpointError::InvalidCluster {
            id: ClusterId::new(6),
            source: cluster_err.clone(),
        };
        assert_eq!(
            ep_err.source().and_then(|s| s.downcast_ref::<ClusterError>()),
            Some(&cluster_err)
        );
        assert!(EndpointError::EmptyDeviceTypes.source().is_none());

        let node_err = NodeError::InvalidEndpoint {
            id: EndpointId::new(1),
            source: ep_err.clone(),
        };
        assert_eq!(
            node_err.source().and_then(|s| s.downcast_ref::<EndpointError>()),
            Some(&ep_err)
        );
        assert!(NodeError::MissingRootEndpoint.source().is_none());

        // Walk full chain: NodeError -> EndpointError -> ClusterError -> AttributeError -> MatterStatus
        let mut depth: usize = 0;
        let mut cur: Option<&(dyn Error + 'static)> = Some(&node_err);
        while let Some(err) = cur {
            depth = depth.saturating_add(1);
            cur = err.source();
        }
        assert_eq!(depth, 5);

        let cmd_err = CommandError::InvalidPayload {
            id: CommandId::new(0),
            field_id: None,
            status: MatterStatus::InvalidDataType,
        };
        assert_eq!(
            cmd_err.source().and_then(|s| s.downcast_ref::<MatterStatus>()),
            Some(&MatterStatus::InvalidDataType)
        );
        assert!(CommandError::UnsupportedCommand(CommandId::new(0)).source().is_none());

        let ev_err = EventError::InvalidPayload {
            id: EventId::new(0),
            field_id: None,
            status: MatterStatus::ConstraintError,
        };
        assert_eq!(
            ev_err.source().and_then(|s| s.downcast_ref::<MatterStatus>()),
            Some(&MatterStatus::ConstraintError)
        );
        assert!(EventError::UnsupportedEvent(EventId::new(0)).source().is_none());
    }

    #[cfg(all(feature = "v1_6", feature = "dispatch", feature = "cluster-on-off"))]
    #[test]
    fn cluster_attribute_from_wire_routes_on_off() {
        use matter_spec::{
            model::AttributeValue,
            v1_6::{clusters::on_off, dispatch::ClusterAttribute},
        };

        let res = ClusterAttribute::from_wire(
            on_off::OnOffCluster::CLUSTER_ID,
            on_off::OnOffCluster::ATTR_ON_OFF,
            AttributeValue::Bool(true),
        );
        assert_eq!(
            res,
            Ok(ClusterAttribute::OnOff(on_off::Attribute::OnOff(
                on_off::attributes::OnOff::new(true)
            )))
        );
    }

    #[test]
    fn exhaustive_command_error_matching() {
        let err = CommandError::DirectionMismatch {
            expected: CommandDirection::ClientToServer,
            actual: CommandDirection::ServerToClient,
        };
        match err {
            CommandError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            CommandError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            CommandError::UnsupportedCommand(id) => {
                assert_eq!(id, CommandId::new(1));
            }
            CommandError::CommandIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            CommandError::DirectionMismatch { expected, actual } => {
                assert_eq!(expected, CommandDirection::ClientToServer);
                assert_eq!(actual, CommandDirection::ServerToClient);
            }
            CommandError::InvalidPayload { id, field_id, status } => {
                assert_eq!((id, field_id, status), (CommandId::new(0), None, MatterStatus::Success));
            }
        }
    }

    #[test]
    fn exhaustive_event_error_matching() {
        let err = EventError::UnsupportedEvent(EventId::new(99));
        match err {
            EventError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            EventError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, actual);
            }
            EventError::UnsupportedEvent(id) => {
                assert_eq!(id, EventId::new(99));
            }
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
        assert_eq!(
            MatterStatus::from(EventError::PriorityMismatch {
                expected: EventPriority::Critical,
                actual: EventPriority::Debug,
            }),
            MatterStatus::InvalidDataType
        );
    }

    #[test]
    fn exhaustive_cluster_error_matching() {
        let err = ClusterError::ClusterIdMismatch {
            expected: ClusterId::new(1),
            actual: ClusterId::new(2),
        };
        match err {
            ClusterError::UnsupportedCluster(id) => {
                assert_eq!(id, ClusterId::new(0));
            }
            ClusterError::ClusterIdMismatch { expected, actual } => {
                assert_eq!(expected, ClusterId::new(1));
                assert_eq!(actual, ClusterId::new(2));
            }
            ClusterError::MissingMandatoryAttribute(id) => {
                assert_eq!(id, AttributeId::new(0));
            }
            ClusterError::InvalidAttribute(source) => {
                assert_eq!(source, AttributeError::UnsupportedAttribute(AttributeId::new(0)));
            }
        }
    }

    #[test]
    fn exhaustive_endpoint_error_matching() {
        let err = EndpointError::EmptyDeviceTypes;
        match err {
            EndpointError::EmptyDeviceTypes => {}
            EndpointError::UnsupportedDeviceType(id) => {
                assert_eq!(id, DeviceTypeId::new(1));
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
            EndpointError::DisallowedCluster(id) => {
                assert_eq!(id, ClusterId::new(3));
            }
            EndpointError::DuplicateDeviceType(id) => {
                assert_eq!(id, DeviceTypeId::new(4));
            }
            EndpointError::DuplicateCluster(id) => {
                assert_eq!(id, ClusterId::new(5));
            }
            EndpointError::InvalidCluster { id, source } => {
                assert_eq!(
                    (id, source),
                    (ClusterId::new(0), ClusterError::UnsupportedCluster(ClusterId::new(0)))
                );
            }
        }
    }

    #[test]
    fn exhaustive_node_error_matching() {
        let err = NodeError::DuplicateEndpoint(EndpointId::new(1));
        match err {
            NodeError::MissingRootEndpoint => {}
            NodeError::DuplicateEndpoint(id) => {
                assert_eq!(id, EndpointId::new(1));
            }
            NodeError::InvalidEndpoint { id, source } => {
                assert_eq!((id, source), (EndpointId::new(0), EndpointError::EmptyDeviceTypes));
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_deny_unknown_fields_with_flatten() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Inner {
            a: u32,
        }

        #[derive(Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Outer {
            #[serde(flatten)]
            inner: Inner,
            b: u32,
        }

        let valid_json = r#"{"a": 1, "b": 2}"#;
        let valid_res: Result<Outer, _> = serde_json::from_str(valid_json);
        valid_res.unwrap();

        let invalid_json = r#"{"a": 1, "b": 2, "c": 3}"#;
        let invalid_res: Result<Outer, _> = serde_json::from_str(invalid_json);
        assert!(invalid_res.is_err());
    }

    #[cfg(feature = "schemars")]
    #[test]
    fn schemars_deny_unknown_fields() {
        use schemars::JsonSchema;

        #[derive(JsonSchema)]
        #[schemars(deny_unknown_fields)]
        struct MyStruct {
            _a: u32,
        }

        let schema = schemars::schema_for!(MyStruct);
        let schema_json = serde_json::to_string(&schema).unwrap();
        // Schemars outputs additionalProperties: false
        assert!(
            schema_json.contains("\"additionalProperties\":false")
                || schema_json.contains("\"additionalProperties\": false")
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn attribute_error_unknown_fields_behavior() {
        let json = r#"{"AttributeIdMismatch": {"expected": 1, "actual": 2, "extra_unknown_field": 42}}"#;
        let res: Result<AttributeError, _> = serde_json::from_str(json);
        #[cfg(feature = "deny-unknown-fields")]
        let _err = res.expect_err("expected error");
        #[cfg(not(feature = "deny-unknown-fields"))]
        res.unwrap();
    }
}
