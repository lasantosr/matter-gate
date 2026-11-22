#![cfg(all(
    feature = "v1_6",
    feature = "dispatch",
    feature = "serde",
    feature = "cluster-on-off",
    feature = "endpoint-on-off-light"
))]

use matter_spec::{
    dispatch::{Described, EntityDescriptor, Invocation, MatterDescribe as _},
    model::{AttributeValue, EndpointId},
    v1_6::{
        clusters::on_off::{self, OnOffCluster, OnOffControlBitmap},
        dispatch::{ClusterAttribute, ClusterCommand, ClusterSnapshot, EndpointDevice},
        endpoints::OnOffLight,
    },
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zst_command_json_omits_payload() {
        let local = on_off::Command::Off;
        let local_json = serde_json::to_value(&local).expect("serialize local command");
        assert_eq!(local_json, serde_json::json!({ "command": "Off" }));
        let local_back: on_off::Command = serde_json::from_value(local_json).expect("deserialize local command");
        assert_eq!(local_back, local);

        let global = ClusterCommand::OnOff(on_off::Command::Off);
        let global_json = serde_json::to_value(&global).expect("serialize global command");
        assert_eq!(
            global_json,
            serde_json::json!({ "cluster": "OnOff", "payload": { "command": "Off" } })
        );
        let global_back: ClusterCommand = serde_json::from_value(global_json).expect("deserialize global command");
        assert_eq!(global_back, global);
    }

    #[test]
    fn parameterized_command_json_round_trip() {
        let cmd = on_off::commands::OnWithTimedOff::new(OnOffControlBitmap::EMPTY, 10_u16, 20_u16);
        let global = ClusterCommand::OnOff(on_off::Command::OnWithTimedOff(cmd.clone()));
        let json = serde_json::to_value(&global).expect("serialize parameterized ClusterCommand");
        let obj = json.as_object().expect("JSON object");
        assert_eq!(obj.get("cluster").and_then(|v| v.as_str()), Some("OnOff"));
        let inner = obj.get("payload").and_then(|v| v.as_object()).expect("payload object");
        assert_eq!(inner.get("command").and_then(|v| v.as_str()), Some("OnWithTimedOff"));
        assert!(inner.get("payload").is_some_and(serde_json::Value::is_object));

        let back: ClusterCommand = serde_json::from_value(json).expect("deserialize parameterized ClusterCommand");
        assert_eq!(back, global);
        let ClusterCommand::OnOff(on_off_cmd) = back else {
            panic!("expected OnOff ClusterCommand");
        };
        let extracted: on_off::commands::OnWithTimedOff = on_off_cmd.try_into().expect("extract concrete command");
        assert_eq!(extracted, cmd);
    }

    #[test]
    fn described_concrete_and_enum_round_trip() {
        let described_zst = Described::new(on_off::commands::Off);
        let zst_json = serde_json::to_value(described_zst).expect("serialize Described<Off>");
        let zst_obj = zst_json.as_object().expect("JSON object");
        assert_eq!(zst_obj.get("cluster").and_then(|v| v.as_str()), Some("OnOff"));
        assert_eq!(zst_obj.get("command").and_then(|v| v.as_str()), Some("Off"));
        assert!(zst_obj.get("payload").is_none());
        let zst_back: Described<on_off::commands::Off> =
            serde_json::from_value(zst_json).expect("deserialize Described<Off>");
        assert_eq!(zst_back.into_inner(), on_off::commands::Off);

        let described_enum_zst = Described::new(ClusterCommand::OnOff(on_off::Command::Off));
        let enum_zst_json =
            serde_json::to_value(&described_enum_zst).expect("serialize Described<ClusterCommand::Off>");
        let enum_zst_obj = enum_zst_json.as_object().expect("JSON object");
        assert_eq!(enum_zst_obj.get("cluster").and_then(|v| v.as_str()), Some("OnOff"));
        assert_eq!(enum_zst_obj.get("command").and_then(|v| v.as_str()), Some("Off"));
        assert!(enum_zst_obj.get("payload").is_none());
        let enum_zst_back: Described<ClusterCommand> =
            serde_json::from_value(enum_zst_json).expect("deserialize Described<ClusterCommand::Off>");
        assert_eq!(enum_zst_back.into_inner(), ClusterCommand::OnOff(on_off::Command::Off));

        let timed = on_off::commands::OnWithTimedOff::new(OnOffControlBitmap::EMPTY, 5_u16, 15_u16);
        let described_enum_param =
            Described::new(ClusterCommand::OnOff(on_off::Command::OnWithTimedOff(timed.clone())));
        let enum_param_json =
            serde_json::to_value(&described_enum_param).expect("serialize Described<ClusterCommand::OnWithTimedOff>");
        let enum_param_obj = enum_param_json.as_object().expect("JSON object");
        assert_eq!(enum_param_obj.get("cluster").and_then(|v| v.as_str()), Some("OnOff"));
        assert_eq!(
            enum_param_obj.get("command").and_then(|v| v.as_str()),
            Some("OnWithTimedOff")
        );
        assert!(enum_param_obj.get("payload").is_some_and(serde_json::Value::is_object));
        let enum_param_back: Described<ClusterCommand> =
            serde_json::from_value(enum_param_json).expect("deserialize Described<ClusterCommand::OnWithTimedOff>");
        assert_eq!(
            enum_param_back.into_inner(),
            ClusterCommand::OnOff(on_off::Command::OnWithTimedOff(timed))
        );
    }

    #[test]
    fn invocation_cluster_command_round_trip() {
        let inv_zst = Invocation::new(EndpointId::new(1), ClusterCommand::OnOff(on_off::Command::Off));
        assert_eq!(
            inv_zst,
            Invocation::new(1_u16, ClusterCommand::OnOff(on_off::Command::Off))
        );
        let zst_json = serde_json::to_value(&inv_zst).expect("serialize Invocation ZST");
        let zst_obj = zst_json.as_object().expect("JSON object");
        assert_eq!(zst_obj.get("endpoint_id").and_then(serde_json::Value::as_u64), Some(1));
        assert_eq!(zst_obj.get("cluster").and_then(|v| v.as_str()), Some("OnOff"));
        assert_eq!(zst_obj.get("command").and_then(|v| v.as_str()), Some("Off"));
        assert!(zst_obj.get("payload").is_none());
        let zst_back: Invocation<ClusterCommand> =
            serde_json::from_value(zst_json).expect("deserialize Invocation ZST");
        assert_eq!(zst_back.endpoint_id, EndpointId::new(1));
        assert_eq!(zst_back.payload, ClusterCommand::OnOff(on_off::Command::Off));

        let timed = on_off::commands::OnWithTimedOff::new(OnOffControlBitmap::EMPTY, 1_u16, 2_u16);
        let inv_param = Invocation::new(
            EndpointId::new(2),
            ClusterCommand::OnOff(on_off::Command::OnWithTimedOff(timed.clone())),
        );
        let param_json = serde_json::to_value(&inv_param).expect("serialize Invocation param");
        let param_back: Invocation<ClusterCommand> =
            serde_json::from_value(param_json).expect("deserialize Invocation param");
        assert_eq!(param_back.endpoint_id, EndpointId::new(2));
        assert_eq!(
            param_back.payload,
            ClusterCommand::OnOff(on_off::Command::OnWithTimedOff(timed))
        );
    }

    #[test]
    fn described_endpoint_device_round_trip() {
        let light = OnOffLight::default();
        let device = EndpointDevice::OnOffLight(Box::new(light.clone()));
        assert!(matches!(
            device.descriptor(),
            EntityDescriptor::Device {
                device_type: "OnOffLight",
                ..
            }
        ));

        let described = Described::new(device);
        let json = serde_json::to_value(&described).expect("serialize Described<EndpointDevice>");
        let obj = json.as_object().expect("JSON object");
        assert_eq!(obj.get("device_type").and_then(|v| v.as_str()), Some("OnOffLight"));
        assert!(obj.get("payload").is_some_and(serde_json::Value::is_object));

        let back: Described<EndpointDevice> =
            serde_json::from_value(json).expect("deserialize Described<EndpointDevice>");
        assert_eq!(back.into_inner(), EndpointDevice::OnOffLight(Box::new(light)));
    }

    #[test]
    fn wire_to_json_gateway_flow() {
        use matter_spec::model::{Command as WireCommand, CommandDirection};

        let cmd = ClusterCommand::from_wire(WireCommand::new(
            OnOffCluster::CLUSTER_ID,
            on_off::commands::Off::COMMAND_ID,
            CommandDirection::ClientToServer,
            AttributeValue::Null,
        ))
        .expect("from_wire Off");
        assert_eq!(cmd, ClusterCommand::OnOff(on_off::Command::Off));

        let inv = Invocation::new(EndpointId::new(3), cmd);
        let json_str = serde_json::to_string(&inv).expect("serialize Invocation to JSON string");
        let back: Invocation<ClusterCommand> = serde_json::from_str(&json_str).expect("deserialize Invocation");
        assert_eq!(back.endpoint_id, EndpointId::new(3));
        let ClusterCommand::OnOff(on_off_cmd) = back.payload else {
            panic!("expected OnOff ClusterCommand");
        };
        let concrete: on_off::commands::Off = on_off_cmd.try_into().expect("try_into Off");
        assert_eq!(concrete, on_off::commands::Off);
    }

    #[test]
    fn cluster_snapshot_and_attribute_round_trip() {
        let snap = Described::new(ClusterSnapshot::OnOff(OnOffCluster::default()));
        let snap_json = serde_json::to_value(&snap).expect("serialize ClusterSnapshot");
        let snap_back: Described<ClusterSnapshot> =
            serde_json::from_value(snap_json).expect("deserialize ClusterSnapshot");
        assert_eq!(snap_back.into_inner(), ClusterSnapshot::OnOff(OnOffCluster::default()));

        let attr = Described::new(ClusterAttribute::OnOff(on_off::Attribute::OnOff(
            on_off::attributes::OnOff::new(true),
        )));
        let attr_json = serde_json::to_value(&attr).expect("serialize ClusterAttribute");
        let attr_back: Described<ClusterAttribute> =
            serde_json::from_value(attr_json).expect("deserialize ClusterAttribute");
        assert_eq!(
            attr_back.into_inner(),
            ClusterAttribute::OnOff(on_off::Attribute::OnOff(on_off::attributes::OnOff::new(true)))
        );
    }

    #[test]
    fn local_and_global_enum_conversions_round_trip() {
        use matter_spec::model::Command as WireCommand;

        let global_cmd = ClusterCommand::OnOff(on_off::Command::Off);
        let wire_cmd = WireCommand::from(global_cmd.clone());
        let cmd_back = ClusterCommand::from_wire(wire_cmd).expect("from_wire ClusterCommand");
        assert_eq!(cmd_back, global_cmd);

        let light = OnOffLight::default();
        let global_dev = EndpointDevice::OnOffLight(Box::new(light));
        let wire_ep = global_dev.clone().into_endpoint(EndpointId::new(1));
        let dev_back = EndpointDevice::try_from(wire_ep).expect("try_from EndpointDevice");
        assert_eq!(dev_back, global_dev);
    }

    #[test]
    fn polymorphic_unknown_round_trips_for_all_seven_dispatch_enums() {
        use matter_spec::{
            dispatch::{AttributeMeta as _, ClusterMeta as _, CommandMeta as _, DeviceMeta as _, EventMeta as _},
            model::{
                Attribute as WireAttribute, AttributeId, Cluster, ClusterId, Command as WireCommand, CommandDirection,
                CommandId, DataVersion, DeviceTypeEntry, DeviceTypeId, Endpoint, Event as WireEvent, EventId,
                EventPriority, Optional,
            },
            v1_6::dispatch::{ClusterCommandResponse, ClusterEvent, ClusterPatch},
            wire::WireCluster,
        };

        let unknown_cid = ClusterId::new(0xFFF1_0042);

        // 1. ClusterSnapshot::Unknown (+ WireCluster<ClusterSnapshot>)
        let raw_cluster = Cluster::new(
            unknown_cid,
            vec![WireAttribute::new(AttributeId::new(0x0001), AttributeValue::U16(99))],
        )
        .with_data_version(DataVersion::new(7));
        let snap = ClusterSnapshot::try_from(raw_cluster.clone()).expect("unknown ClusterSnapshot");
        assert!(matches!(snap, ClusterSnapshot::Unknown(cid, _) if cid == unknown_cid));
        assert_eq!(snap.cluster_name(), "Unknown");
        assert_eq!(snap.cluster_id(), unknown_cid);
        let snap_desc_json = serde_json::to_value(Described::new(snap.clone())).expect("serialize Unknown snapshot");
        assert_eq!(snap_desc_json["cluster"], "Unknown");
        assert_eq!(snap_desc_json["cluster_id"], unknown_cid.raw());
        let snap_desc_back: Described<ClusterSnapshot> =
            serde_json::from_value(snap_desc_json).expect("deserialize Unknown snapshot");
        assert_eq!(snap_desc_back.into_inner(), snap);

        let env_snap = WireCluster::<ClusterSnapshot>::try_from(raw_cluster).expect("enveloped Unknown");
        assert_eq!(env_snap.data_version, Optional::Present(DataVersion::new(7)));
        let env_wire = Cluster::from(env_snap);
        assert_eq!(env_wire.id, unknown_cid);
        assert_eq!(env_wire.data_version, Optional::Present(DataVersion::new(7)));

        // 2. ClusterPatch::Unknown
        let patch = ClusterPatch::Unknown(
            unknown_cid,
            vec![WireAttribute::new(AttributeId::new(0x0002), AttributeValue::Bool(true))],
        );
        assert_eq!(patch.cluster_name(), "Unknown");
        assert_eq!(patch.cluster_id(), unknown_cid);
        let patch_json = serde_json::to_value(Described::new(patch.clone())).expect("serialize Unknown patch");
        let patch_back: Described<ClusterPatch> =
            serde_json::from_value(patch_json).expect("deserialize Unknown patch");
        assert_eq!(patch_back.into_inner(), patch);
        let patch_wire = Cluster::from(patch);
        assert_eq!(patch_wire.id, unknown_cid);

        // 3. ClusterAttribute::Unknown
        let attr_aid = AttributeId::new(0x0003);
        let attr = ClusterAttribute::from_wire(unknown_cid, attr_aid, AttributeValue::U8(5)).expect("unknown attr");
        assert!(matches!(attr, ClusterAttribute::Unknown(c, a, _) if c == unknown_cid && a == attr_aid));
        assert_eq!(attr.cluster_name(), "Unknown");
        assert_eq!(attr.attribute_name(), "Unknown");
        let attr_json = serde_json::to_value(Described::new(attr.clone())).expect("serialize Unknown attr");
        let attr_back: Described<ClusterAttribute> =
            serde_json::from_value(attr_json).expect("deserialize Unknown attr");
        assert_eq!(attr_back.into_inner(), attr);
        assert_eq!(
            WireAttribute::from(attr),
            WireAttribute::new(attr_aid, AttributeValue::U8(5))
        );

        // 4. ClusterCommand::Unknown
        let raw_cmd = WireCommand::new(
            unknown_cid,
            CommandId::new(0x01),
            CommandDirection::ClientToServer,
            AttributeValue::U32(123),
        );
        let cmd = ClusterCommand::from_wire(raw_cmd.clone()).expect("unknown ClusterCommand");
        assert!(matches!(cmd, ClusterCommand::Unknown(_)));
        assert_eq!(cmd.cluster_name(), "Unknown");
        assert_eq!(cmd.command_name(), "Unknown");
        let cmd_json = serde_json::to_value(Described::new(cmd.clone())).expect("serialize Unknown cmd");
        let cmd_back: Described<ClusterCommand> = serde_json::from_value(cmd_json).expect("deserialize Unknown cmd");
        assert_eq!(cmd_back.into_inner(), cmd);
        assert_eq!(WireCommand::from(cmd), raw_cmd);

        // 5. ClusterCommandResponse::Unknown
        let raw_resp = WireCommand::new(
            unknown_cid,
            CommandId::new(0x02),
            CommandDirection::ServerToClient,
            AttributeValue::String("ok".into()),
        );
        let resp = ClusterCommandResponse::from_wire(raw_resp.clone()).expect("unknown ClusterCommandResponse");
        assert!(matches!(resp, ClusterCommandResponse::Unknown(_)));
        assert_eq!(resp.cluster_name(), "Unknown");
        assert_eq!(resp.command_name(), "Unknown");
        let resp_json = serde_json::to_value(Described::new(resp.clone())).expect("serialize Unknown resp");
        let resp_back: Described<ClusterCommandResponse> =
            serde_json::from_value(resp_json).expect("deserialize Unknown resp");
        assert_eq!(resp_back.into_inner(), resp);
        assert_eq!(WireCommand::from(resp), raw_resp);

        // 6. ClusterEvent::Unknown
        let raw_ev = WireEvent::new(
            unknown_cid,
            EventId::new(0x04),
            EventPriority::Info,
            AttributeValue::Null,
        );
        let ev = ClusterEvent::from_wire(raw_ev.clone()).expect("unknown ClusterEvent");
        assert!(matches!(ev, ClusterEvent::Unknown(_)));
        assert_eq!(ev.cluster_name(), "Unknown");
        assert_eq!(ev.event_name(), "Unknown");
        let ev_json = serde_json::to_value(Described::new(ev.clone())).expect("serialize Unknown event");
        let ev_back: Described<ClusterEvent> = serde_json::from_value(ev_json).expect("deserialize Unknown event");
        assert_eq!(ev_back.into_inner(), ev);
        assert_eq!(WireEvent::from(ev), raw_ev);

        // 7. EndpointDevice::Unknown
        let unknown_dt = DeviceTypeId::new(0xFFF1_0099);
        let raw_ep = Endpoint::new(
            EndpointId::new(9),
            vec![DeviceTypeEntry::new(unknown_dt, 1)],
            vec![],
            vec![],
        );
        let dev = EndpointDevice::try_from(raw_ep.clone()).expect("unknown EndpointDevice");
        assert!(matches!(dev, EndpointDevice::Unknown(_)));
        assert_eq!(dev.device_type_name(), "Unknown");
        assert_eq!(dev.device_type_id(), unknown_dt);
        let dev_json = serde_json::to_value(Described::new(dev.clone())).expect("serialize Unknown device");
        let dev_back: Described<EndpointDevice> = serde_json::from_value(dev_json).expect("deserialize Unknown device");
        assert_eq!(dev_back.into_inner(), dev);
        assert_eq!(dev.into_endpoint(EndpointId::new(9)), raw_ep);
    }

    #[cfg(feature = "cluster-operational-state")]
    #[test]
    fn byte_size_boxing_boxes_operational_state_and_keeps_on_off_unboxed() {
        use matter_spec::{
            dispatch::ClusterMeta as _,
            v1_6::{
                clusters::operational_state::{OperationalStateCluster, OperationalStateClusterPatch},
                dispatch::ClusterPatch,
            },
        };

        let op_snap = ClusterSnapshot::OperationalState(Box::<OperationalStateCluster>::default());
        let op_patch = ClusterPatch::OperationalState(Box::<OperationalStateClusterPatch>::default());
        assert_eq!(op_snap.cluster_id(), OperationalStateCluster::CLUSTER_ID);
        assert_eq!(op_patch.cluster_id(), OperationalStateCluster::CLUSTER_ID);
        // Verify OnOff stays unboxed
        let _on_off_snap = ClusterSnapshot::OnOff(OnOffCluster::default());
        let _on_off_patch = ClusterPatch::OnOff(on_off::OnOffClusterPatch::default());
    }
}
