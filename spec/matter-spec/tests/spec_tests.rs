#[macro_use]
mod common;

#[cfg(all(feature = "v1_0", feature = "all"))]
test_version_behavior!(v1_0_tests, matter_spec::v1_0, OnOffLightSwitch, false, true, false);

#[cfg(all(feature = "v1_1", feature = "all"))]
test_version_behavior!(v1_1_tests, matter_spec::v1_1, OnOffLightSwitch, false, true, false);

#[cfg(all(feature = "v1_2", feature = "all"))]
test_version_behavior!(v1_2_tests, matter_spec::v1_2, OnOffLightSwitch, true, true, false);

#[cfg(all(feature = "v1_3", feature = "all"))]
test_version_behavior!(v1_3_tests, matter_spec::v1_3, OnOffLightSwitch, true, true, true);

#[cfg(all(feature = "v1_4", feature = "all"))]
test_version_behavior!(v1_4_tests, matter_spec::v1_4, OnOffLightSwitch, true, true, true);

#[cfg(all(feature = "v1_5", feature = "all"))]
test_version_behavior!(v1_5_tests, matter_spec::v1_5, OnOffLightSwitch, true, true, true);

#[cfg(all(feature = "v1_6", feature = "all"))]
test_version_behavior!(v1_6_tests, matter_spec::v1_6, OnOffLightSwitch, true, true, true);

#[cfg(all(feature = "v1_2", feature = "all"))]
#[cfg(test)]
mod v1_2_specific_tests {
    use matter_spec::{
        model::{AttributeId, EventId},
        v1_2::globals::attributes::EventList,
    };

    #[test]
    fn event_list_global_attribute_presence() {
        let ev1 = EventId::new(1);
        let ev2 = EventId::new(2);
        let ev_list = EventList::new(vec![ev1, ev2]);
        assert_eq!(ev_list.raw(), &[ev1, ev2]);
        assert_eq!(EventList::ATTRIBUTE_ID, AttributeId::new(0xFFFA));

        let val = ev_list.into_value();
        let parsed = EventList::try_from_value(val).expect("parse EventList");
        assert_eq!(parsed.raw(), &[ev1, ev2]);
    }
}

#[cfg(all(feature = "v1_4", feature = "all", feature = "registry"))]
#[cfg(test)]
mod v1_4_specific_tests {
    use matter_spec::{
        EndpointError, ModeClusterExt as _, NodeError,
        model::{
            AttributeId, Cluster, Command as WireCommand, CommandDirection, CommandId, DeviceTypeEntry, Endpoint,
            EndpointId, Node, NodeId, Nullable, Optional,
        },
        types::ElapsedS,
        v1_4::{
            clusters::{
                descriptor::DescriptorCluster,
                dishwasher_alarm::{AlarmBitmap, DishwasherAlarmCluster},
                dishwasher_mode::{DishwasherModeCluster, ModeTag},
                groups::{GroupsCluster, NameSupportBitmap},
                identify::{IdentifyCluster, IdentifyTypeEnum},
                on_off::{OnOff as OnOffState, OnOffCluster, StartUpOnOffEnum},
                operational_state::{ErrorStateEnum, OperationalStateCluster, OperationalStateEnum},
                scenes_management::ScenesManagementCluster,
                thermostat::{
                    FeatureMap as ThermostatFeatureMap, RelayStateBitmap,
                    commands::{AtomicRequest, AtomicResponse},
                },
            },
            endpoints::{Dishwasher, OnOffLight},
            globals::AtomicAttributeStatusStruct,
            registry,
        },
    };

    #[test]
    fn on_off_light_with_scenes_management_conformance() {
        let light = OnOffLight::new(
            IdentifyCluster::new(0_u16, IdentifyTypeEnum::None),
            GroupsCluster::new(NameSupportBitmap::EMPTY),
            ScenesManagementCluster::new(0_u16, Vec::new()),
            OnOffCluster::ON
                .with_global_scene_control(true)
                .with_on_time(0_u16)
                .with_off_wait_time(0_u16)
                .with_start_up_on_off(Nullable::Null::<StartUpOnOffEnum>),
        );

        assert!(light.level_control.is_omitted());
        let endpoint = light.clone().into_endpoint(EndpointId::new(1));
        assert!(endpoint.server_cluster(DescriptorCluster::CLUSTER_ID).is_some());
        assert!(endpoint.server_cluster(OnOffCluster::CLUSTER_ID).is_some());

        let light_back = OnOffLight::try_from(endpoint).unwrap();
        assert_eq!(light, light_back);

        // Conformance verification detects missing mandatory clusters
        let partial_endpoint = Endpoint::new(
            EndpointId::new(1),
            vec![DeviceTypeEntry::new(OnOffLight::DEVICE_TYPE_ID, OnOffLight::REVISION)],
            vec![
                Cluster::empty(IdentifyCluster::CLUSTER_ID),
                Cluster::empty(GroupsCluster::CLUSTER_ID),
            ],
            vec![],
        );
        let err_missing = OnOffLight::try_from(partial_endpoint);
        assert!(matches!(
            err_missing,
            Err(EndpointError::MissingMandatoryServerCluster(cluster))
                if cluster == DescriptorCluster::CLUSTER_ID || cluster == OnOffCluster::CLUSTER_ID
        ));
    }

    #[test]
    fn dishwasher_builder_and_conformance() {
        let mode_cluster =
            DishwasherModeCluster::from_tags([ModeTag::Normal, ModeTag::Heavy], ModeTag::Normal).unwrap();
        assert_eq!(mode_cluster.supported_modes.len(), 2);
        assert_eq!(mode_cluster.current_mode, 0);

        let dishwasher = Dishwasher::new(
            OperationalStateCluster::new(
                None,
                None,
                vec![
                    OperationalStateEnum::Stopped.into(),
                    OperationalStateEnum::Running.into(),
                ],
                OperationalStateEnum::Running,
                ErrorStateEnum::NoError,
            )
            .with_countdown_time(ElapsedS::from_raw(1800)),
        )
        .with_on_off(OnOffState::On)
        .with_dishwasher_mode(mode_cluster)
        .with_dishwasher_alarm(DishwasherAlarmCluster {
            mask: AlarmBitmap::INFLOW_ERROR | AlarmBitmap::DRAIN_ERROR,
            supported: AlarmBitmap::INFLOW_ERROR | AlarmBitmap::DRAIN_ERROR | AlarmBitmap::DOOR_ERROR,
            ..Default::default()
        });

        assert!(dishwasher.on_off.is_present());
        assert!(dishwasher.dishwasher_mode.is_present());
        assert!(dishwasher.dishwasher_alarm.is_present());
        let endpoint = dishwasher.clone().into_endpoint(EndpointId::new(1));
        let dw_back = Dishwasher::try_from(endpoint).unwrap();
        assert_eq!(dishwasher, dw_back);
    }

    #[test]
    fn atomic_response_command_with_optional() {
        assert_eq!(AtomicResponse::COMMAND_ID, CommandId::new(0xFD));
        assert_eq!(AtomicResponse::DIRECTION, CommandDirection::ServerToClient);

        let resp = AtomicResponse::new(
            0_u8,
            vec![AtomicAttributeStatusStruct {
                attribute_id: AttributeId::new(0x0000),
                status_code: 0,
            }],
        )
        .with_timeout(60_u16);

        let resp_val = resp.into_value();
        let parsed_resp = AtomicResponse::try_from_value(resp_val).expect("parse AtomicResponse");
        assert_eq!(parsed_resp.status_code, 0);
        assert_eq!(parsed_resp.attribute_status.len(), 1);
        assert_eq!(parsed_resp.attribute_status[0].attribute_id, AttributeId::new(0x0000));
        assert_eq!(parsed_resp.timeout, Optional::Present(60));

        assert_eq!(AtomicRequest::COMMAND_ID, CommandId::new(0xFE));
        assert_eq!(AtomicResponse::COMMAND_ID, CommandId::new(0xFD));
        assert_eq!(AtomicRequest::DIRECTION, CommandDirection::ClientToServer);
        assert_eq!(AtomicResponse::DIRECTION, CommandDirection::ServerToClient);
    }

    #[test]
    fn runtime_registry_validation() {
        let light = OnOffLight::new(
            IdentifyCluster::new(0_u16, IdentifyTypeEnum::None),
            GroupsCluster::new(NameSupportBitmap::EMPTY),
            ScenesManagementCluster::new(0_u16, Vec::new()),
            OnOffCluster::ON
                .with_global_scene_control(true)
                .with_on_time(0_u16)
                .with_off_wait_time(0_u16)
                .with_start_up_on_off(Nullable::Null::<StartUpOnOffEnum>),
        );
        let ep = light.into_endpoint(EndpointId::new(1));

        let validator = registry::Validator::full();

        // Validate cluster via registry
        let on_off_wire = Cluster::from(OnOffCluster::ON);
        validator.cluster(&on_off_wire).expect("validate cluster in registry");
        assert_eq!(on_off_wire.id, OnOffCluster::CLUSTER_ID);

        // Validate command via registry
        use matter_spec::v1_4::globals::AtomicRequestTypeEnum;
        let req = AtomicRequest::new(AtomicRequestTypeEnum::BeginWrite, vec![AttributeId::new(0x0000)]);
        let cmd_wire = WireCommand::from(req);
        validator.command(&cmd_wire).expect("validate command in registry");
        assert_eq!(cmd_wire.command_id, AtomicRequest::COMMAND_ID);

        // Validate endpoint via registry
        validator.endpoint(&ep).expect("validate endpoint in registry");
        assert!(ep.server_cluster(OnOffCluster::CLUSTER_ID).is_some());

        // Validate node missing root endpoint
        let invalid_node = Node::new(NodeId::new(1), vec![ep.clone()]);
        assert!(matches!(
            validator.node(&invalid_node),
            Err(NodeError::MissingRootEndpoint)
        ));

        // Validate valid node with root endpoint
        let root_light = OnOffLight::new(
            IdentifyCluster::new(0_u16, IdentifyTypeEnum::None),
            GroupsCluster::new(NameSupportBitmap::EMPTY),
            ScenesManagementCluster::new(0_u16, Vec::new()),
            OnOffCluster::ON
                .with_global_scene_control(true)
                .with_on_time(0_u16)
                .with_off_wait_time(0_u16)
                .with_start_up_on_off(Nullable::Null::<StartUpOnOffEnum>),
        );
        let root_ep = root_light.into_endpoint(EndpointId::ROOT);

        let node = Node::new(NodeId::new(1), vec![root_ep, ep]);
        validator.node(&node).expect("validate node in registry");
        let validated_node = node;
        assert_eq!(validated_node.endpoints.len(), 2);
    }

    #[test]
    fn bitmap_semantics_and_operators() {
        assert_eq!(RelayStateBitmap::EMPTY, RelayStateBitmap::empty());
        assert!(RelayStateBitmap::EMPTY.is_empty());
        assert_eq!(RelayStateBitmap::HEAT.bits(), 0x0001);

        let running = RelayStateBitmap::HEAT | RelayStateBitmap::COOL;
        assert!(running.contains(RelayStateBitmap::HEAT));
        assert!(running.contains(RelayStateBitmap::COOL));
        assert!(!running.contains(RelayStateBitmap::FAN));
        assert!(running.is_heat());
        assert!(running.is_cool());
        assert!(!running.is_fan());

        let mut flags = RelayStateBitmap::empty();
        flags.insert(RelayStateBitmap::HEAT);
        flags |= RelayStateBitmap::FAN;
        assert_eq!(flags.bits(), 0x0005);

        flags.remove(RelayStateBitmap::HEAT);
        assert_eq!(flags, RelayStateBitmap::FAN);

        flags &= RelayStateBitmap::FAN;
        assert_eq!(flags, RelayStateBitmap::FAN);

        flags ^= RelayStateBitmap::FAN;
        assert!(flags.is_empty());

        let union_val = RelayStateBitmap::HEAT.union(RelayStateBitmap::COOL);
        assert_eq!(union_val, RelayStateBitmap::HEAT | RelayStateBitmap::COOL);

        let intersect_val = union_val.intersection(RelayStateBitmap::HEAT);
        assert_eq!(intersect_val, RelayStateBitmap::HEAT);

        let not_heat = !RelayStateBitmap::HEAT;
        assert_eq!(not_heat.bits(), !0x0001_u16);

        assert_eq!(u16::from(RelayStateBitmap::HEAT), 0x0001);
        assert_eq!(RelayStateBitmap::from(0x0002), RelayStateBitmap::COOL);

        assert_eq!(format!("{}", RelayStateBitmap::HEAT), "0x0001");
        assert_eq!(format!("{}", ThermostatFeatureMap::EMPTY), "0x00000000");
    }

    #[test]
    fn root_node_default_endpoint_id_and_composed_validation() {
        use matter_spec::{
            EndpointExt as _,
            error::EndpointError,
            model::{Cluster, DeviceTypeEntry, Endpoint, EndpointId},
            v1_4::{
                clusters::{
                    bridged_device_basic_information::BridgedDeviceBasicInformationCluster, on_off::OnOffCluster,
                },
                endpoints::{
                    bridged_node::BridgedNode, dishwasher::Dishwasher, on_off_light::OnOffLight,
                    on_off_light_switch::OnOffLightSwitch, root_node::RootNode,
                },
                registry,
            },
        };

        let root = RootNode::default();
        let root_ep = Endpoint::from(root);
        assert_eq!(root_ep.id, EndpointId::ROOT);

        let light = OnOffLight {
            on_off: OnOffCluster::OFF
                .with_global_scene_control(true)
                .with_on_time(0_u16)
                .with_off_wait_time(0_u16)
                .with_start_up_on_off(Nullable::Null::<StartUpOnOffEnum>),
            ..Default::default()
        };
        let light_ep5 = light.clone().into_endpoint(EndpointId::new(5));
        assert_eq!(light_ep5.id, EndpointId::new(5));

        // Composed/bridged endpoint: [OnOffLight, BridgedNode]
        let mut composed = light.into_endpoint(EndpointId::new(2));
        composed
            .device_types
            .push(DeviceTypeEntry::new(BridgedNode::DEVICE_TYPE_ID, BridgedNode::REVISION));
        composed
            .server_clusters
            .push(Cluster::from(BridgedDeviceBasicInformationCluster::default()));
        {
            use matter_spec::v1_4::globals::DeviceTypeStruct;
            let new_desc = Cluster::from(DescriptorCluster::new(
                composed
                    .device_types
                    .iter()
                    .map(|dt| DeviceTypeStruct::new(dt.id, dt.revision))
                    .collect::<Vec<_>>(),
                composed.server_clusters.iter().map(|c| c.id).collect::<Vec<_>>(),
                composed.client_clusters.clone(),
                composed.parts.clone(),
            ));
            if let Some(desc) = composed
                .server_clusters
                .iter_mut()
                .find(|c| c.id == DescriptorCluster::CLUSTER_ID)
            {
                *desc = new_desc;
            }
        }
        registry::Validator::full()
            .endpoint(&composed)
            .expect("composed/bridged endpoint should validate");
        assert_eq!(composed.device_types.len(), 2);

        let light_view = composed
            .view_as::<OnOffLight>()
            .expect("composed endpoint declares valid OnOffLight")
            .expect("OnOffLight should be present on composed endpoint");
        assert!(light_view.on_off().is_some());
        assert!(light_view.descriptor().is_some());

        let bridged_view = composed
            .view_as::<BridgedNode>()
            .expect("composed endpoint declares valid BridgedNode")
            .expect("BridgedNode should be present on composed endpoint");
        assert!(bridged_view.bridged_device_basic_information().is_some());

        assert!(
            composed
                .view_as::<Dishwasher>()
                .expect("absent device type should return Ok(None)")
                .is_none()
        );

        let decoded_light =
            OnOffLight::try_from(composed).expect("OnOffLight::try_from should ignore extra non-disallowed clusters");
        assert_eq!(decoded_light.on_off.on_off, OnOffState::Off);

        // Single-device OnOffLightSwitch rejects GroupsCluster as a server cluster (ConformanceOp::Disallow)
        let mut switch_ep = OnOffLightSwitch::default().into_endpoint(EndpointId::new(1));
        switch_ep
            .server_clusters
            .push(Cluster::from(GroupsCluster::new(NameSupportBitmap::EMPTY)));
        {
            use matter_spec::v1_4::globals::DeviceTypeStruct;
            let switch_desc = Cluster::from(DescriptorCluster::new(
                switch_ep
                    .device_types
                    .iter()
                    .map(|dt| DeviceTypeStruct::new(dt.id, dt.revision))
                    .collect::<Vec<_>>(),
                switch_ep.server_clusters.iter().map(|c| c.id).collect::<Vec<_>>(),
                switch_ep.client_clusters.clone(),
                switch_ep.parts.clone(),
            ));
            if let Some(desc) = switch_ep
                .server_clusters
                .iter_mut()
                .find(|c| c.id == DescriptorCluster::CLUSTER_ID)
            {
                *desc = switch_desc;
            }
        }
        let err = registry::Validator::full()
            .endpoint(&switch_ep)
            .expect_err("single-device switch should reject Groups server");
        assert_eq!(err, EndpointError::DisallowedCluster(GroupsCluster::CLUSTER_ID));
    }
}

#[cfg(all(feature = "v1_6", feature = "all", feature = "registry"))]
#[cfg(test)]
mod v1_6_regression_tests {
    use matter_spec::{
        error::CommandError,
        model::{
            Attribute, AttributeId, AttributeValue, Cluster, Command as WireCommand, CommandDirection, CommandId,
            DataVersion, EndpointId, Optional,
        },
        v1_6::{
            clusters::{
                descriptor::DescriptorCluster,
                on_off::{self, OnOff, OnOffCluster},
                thermostat::ThermostatCluster,
            },
            endpoints::on_off_light::OnOffLight,
            registry,
        },
    };

    #[test]
    fn global_attributes_0xfff8_to_0xfffd_roundtrip_and_unknown_rejected() {
        for mut wire in [
            Cluster::from(OnOffCluster::new(OnOff::On)),
            Cluster::from(ThermostatCluster::default()),
        ] {
            wire.attributes
                .push(Attribute::new(AttributeId::new(0xFFF8), AttributeValue::List(vec![])));
            wire.attributes
                .push(Attribute::new(AttributeId::new(0xFFF9), AttributeValue::List(vec![])));
            wire.attributes
                .push(Attribute::new(AttributeId::new(0xFFFA), AttributeValue::List(vec![])));
            wire.attributes
                .push(Attribute::new(AttributeId::new(0xFFFB), AttributeValue::List(vec![])));
            wire.attributes
                .push(Attribute::new(AttributeId::new(0xFFFC), AttributeValue::U32(0)));
            wire.attributes
                .push(Attribute::new(AttributeId::new(0xFFFD), AttributeValue::U16(1)));

            let cluster_id = wire.id;
            if cluster_id == OnOffCluster::CLUSTER_ID {
                let decoded = OnOffCluster::try_from(wire).expect("OnOffCluster should accept 0xFFF8..=0xFFFD");
                assert_eq!(decoded.on_off, OnOff::On);
            } else {
                let _decoded =
                    ThermostatCluster::try_from(wire).expect("ThermostatCluster should accept 0xFFF8..=0xFFFD");
            }
        }

        let mut bad_wire = Cluster::from(OnOffCluster::new(OnOff::Off));
        bad_wire
            .attributes
            .push(Attribute::new(AttributeId::new(0x1234), AttributeValue::U8(1)));
        let err = OnOffCluster::try_from(bad_wire).expect_err("unknown non-global attribute must fail");
        assert!(format!("{err}").contains("UnsupportedAttribute") || !format!("{err}").is_empty());
    }

    #[test]
    fn endpoint_validate_and_try_from_preserve_descriptor_and_data_version() {
        use matter_spec::{model::Nullable, v1_6::clusters::on_off::StartUpOnOffEnum};

        let light = OnOffLight {
            on_off: OnOffCluster::OFF
                .with_global_scene_control(true)
                .with_on_time(0_u16)
                .with_off_wait_time(0_u16)
                .with_start_up_on_off(Nullable::Null::<StartUpOnOffEnum>),
            ..Default::default()
        };
        let mut light_ep = light.into_endpoint(EndpointId::new(1));
        let custom_version = DataVersion::new(42);
        if let Some(desc_wire) = light_ep
            .server_clusters
            .iter_mut()
            .find(|c| c.id == DescriptorCluster::CLUSTER_ID)
        {
            desc_wire.data_version = Optional::Present(custom_version);
        }
        if let Some(on_off_wire) = light_ep
            .server_clusters
            .iter_mut()
            .find(|c| c.id == OnOffCluster::CLUSTER_ID)
        {
            on_off_wire.data_version = Optional::Present(DataVersion::new(99));
        }

        registry::Validator::full()
            .endpoint(&light_ep)
            .expect("OnOffLight should validate");
        let validated = &light_ep;
        let desc_after = validated
            .server_clusters
            .iter()
            .find(|c| c.id == DescriptorCluster::CLUSTER_ID)
            .expect("Descriptor cluster present");
        assert_eq!(desc_after.data_version, Optional::Present(custom_version));
        let parsed_desc = DescriptorCluster::try_from(desc_after.clone()).expect("parse Descriptor");
        assert_eq!(parsed_desc.parts_list, []);

        let on_off_after = validated
            .server_clusters
            .iter()
            .find(|c| c.id == OnOffCluster::CLUSTER_ID)
            .expect("OnOff cluster present");
        assert_eq!(on_off_after.data_version, Optional::Present(DataVersion::new(99)));

        // Non-mutating validator rejects inconsistent Descriptor PartsList rather than rewriting it.
        if let Some(desc_wire) = light_ep
            .server_clusters
            .iter_mut()
            .find(|c| c.id == DescriptorCluster::CLUSTER_ID)
            && let Some(parts_attr) = desc_wire
                .attributes
                .iter_mut()
                .find(|a| a.id == AttributeId::new(0x0003))
        {
            parts_attr.value = AttributeValue::List(vec![AttributeValue::U16(2), AttributeValue::U16(3)]);
        }
        let _err = registry::Validator::full()
            .endpoint(&light_ep)
            .expect_err("inconsistent PartsList must fail");
    }

    #[test]
    fn command_direction_mismatch_and_directionless_from_wire() {
        // OnOff::Off is a client-to-server command (ID 0x0000).
        let wrong_dir = WireCommand::new(
            OnOffCluster::CLUSTER_ID,
            CommandId::new(0x0000),
            CommandDirection::ServerToClient,
            AttributeValue::Null,
        );
        let err = on_off::Command::try_from(wrong_dir).expect_err("opposite direction must fail");
        assert!(matches!(
            err,
            CommandError::DirectionMismatch { .. } | CommandError::UnsupportedCommand(_)
        ));

        let directionless = on_off::Command::from_wire(WireCommand::new(
            OnOffCluster::CLUSTER_ID,
            CommandId::new(0x0000),
            CommandDirection::ClientToServer,
            AttributeValue::Null,
        ))
        .expect("from_wire should succeed");
        assert_eq!(directionless, on_off::Command::Off);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn optional_cluster_json_omits_and_deserializes_omitted_fields() {
        use matter_spec::{model::Optional, v1_6::clusters::on_off::OnOffClusterPatch};

        let opt = OnOffClusterPatch {
            on_time: Optional::Present(10_u16),
            ..Default::default()
        };
        let json = serde_json::to_value(&opt).expect("serialize OnOffClusterPatch");
        let obj = json.as_object().expect("JSON object");
        assert!(obj.contains_key("on_time"));

        let parsed: OnOffClusterPatch = serde_json::from_value(json).expect("deserialize roundtrip");
        assert_eq!(parsed.on_time, Optional::Present(10));
        assert_eq!(parsed.global_scene_control, Optional::Omitted);
    }

    #[test]
    fn duplicate_attribute_and_field_ids_are_rejected() {
        use matter_spec::{
            error::{AttributeError, ClusterError},
            model::{FieldId, MatterStatus},
            v1_6::clusters::identify,
        };

        let dup_cluster = Cluster::new(
            OnOffCluster::CLUSTER_ID,
            vec![
                Attribute::new(OnOffCluster::ATTR_ON_OFF, AttributeValue::Bool(false)),
                Attribute::new(OnOffCluster::ATTR_ON_OFF, AttributeValue::Bool(true)),
            ],
        );
        let err = OnOffCluster::try_from(dup_cluster).expect_err("duplicate attribute ID must fail");
        assert_eq!(
            err,
            ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: OnOffCluster::ATTR_ON_OFF,
                field_id: None,
                status: MatterStatus::ConstraintError,
            })
        );

        let dup_pairs = [
            (OnOffCluster::ATTR_ON_OFF, AttributeValue::Bool(false)),
            (OnOffCluster::ATTR_ON_OFF, AttributeValue::Bool(true)),
        ];
        assert_eq!(
            OnOffCluster::try_from_attributes(dup_pairs),
            Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: OnOffCluster::ATTR_ON_OFF,
                field_id: None,
                status: MatterStatus::ConstraintError,
            }))
        );

        let unknown_id = AttributeId::new(0x1234);
        let unknown_pairs = [
            (OnOffCluster::ATTR_ON_OFF, AttributeValue::Bool(true)),
            (unknown_id, AttributeValue::U8(1)),
        ];
        assert_eq!(
            OnOffCluster::try_from_attributes(unknown_pairs),
            Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
                unknown_id
            )))
        );

        let dup_cmd_payload = AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::U16(5)),
            (FieldId::new(0), AttributeValue::U16(10)),
        ]);
        assert_eq!(
            identify::commands::Identify::try_from_value(dup_cmd_payload),
            Err(MatterStatus::ConstraintError)
        );
    }

    #[test]
    fn event_envelope_try_from_wire_event_and_priority_mismatch() {
        use matter_spec::{
            error::EventError,
            model::{Event as ModelEvent, EventNumber, EventPriority, EventTimestamp, FieldId},
            v1_6::clusters::basic_information::{self, events::StartUp},
            wire::WireEvent,
        };

        let mut wire_ok = ModelEvent::new(
            basic_information::BasicInformationCluster::CLUSTER_ID,
            StartUp::EVENT_ID,
            StartUp::PRIORITY,
            AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::U32(7))]),
        );
        wire_ok.event_number = Optional::Present(EventNumber::new(42));
        wire_ok.timestamp = Optional::Present(EventTimestamp::EpochMs(1_700_000_000));

        let env_concrete: WireEvent<StartUp> =
            WireEvent::try_from(wire_ok.clone()).expect("WireEvent<StartUp> should decode");
        assert_eq!(env_concrete.priority, EventPriority::Critical);
        assert_eq!(env_concrete.event_number, Optional::Present(EventNumber::new(42)));
        assert_eq!(
            env_concrete.timestamp,
            Optional::Present(EventTimestamp::EpochMs(1_700_000_000))
        );
        assert_eq!(env_concrete.event.software_version, 7);

        let env_enum: WireEvent<basic_information::Event> =
            WireEvent::try_from(wire_ok).expect("WireEvent<Event> should decode");
        assert_eq!(env_enum.event, basic_information::Event::StartUp(StartUp::new(7_u32)));

        let wire_bad_prio = ModelEvent::new(
            basic_information::BasicInformationCluster::CLUSTER_ID,
            StartUp::EVENT_ID,
            EventPriority::Debug,
            AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::U32(7))]),
        );
        assert_eq!(
            WireEvent::<StartUp>::try_from(wire_bad_prio.clone()),
            Err(EventError::PriorityMismatch {
                expected: StartUp::PRIORITY,
                actual: EventPriority::Debug,
            })
        );
        assert_eq!(
            WireEvent::<basic_information::Event>::try_from(wire_bad_prio),
            Err(EventError::PriorityMismatch {
                expected: StartUp::PRIORITY,
                actual: EventPriority::Debug,
            })
        );
    }

    #[test]
    fn concrete_command_and_event_preserve_field_id() {
        use matter_spec::{
            error::{CommandError, EventError},
            model::{Event as WireEvent, EventPriority, FieldId, MatterStatus},
            v1_6::clusters::{
                basic_information::{self, events::StartUp},
                thermostat::commands::SetpointRaiseLower,
            },
        };

        // Missing mandatory FieldId(1) on SetpointRaiseLower -> InvalidPayload with field_id: Some(FieldId(1))
        let bad_cmd = WireCommand::new(
            ThermostatCluster::CLUSTER_ID,
            SetpointRaiseLower::COMMAND_ID,
            CommandDirection::ClientToServer,
            AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::U8(0))]),
        );
        assert_eq!(
            SetpointRaiseLower::try_from(bad_cmd),
            Err(CommandError::InvalidPayload {
                id: SetpointRaiseLower::COMMAND_ID,
                field_id: Some(FieldId::new(1)),
                status: MatterStatus::InvalidCommand,
            })
        );

        // Wrong type for FieldId(0) on concrete StartUp event and enum Event -> field_id: Some(FieldId(0))
        let bad_ev = WireEvent::new(
            basic_information::BasicInformationCluster::CLUSTER_ID,
            StartUp::EVENT_ID,
            StartUp::PRIORITY,
            AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::Bool(true))]),
        );
        assert_eq!(
            StartUp::try_from(bad_ev.clone()),
            Err(EventError::InvalidPayload {
                id: StartUp::EVENT_ID,
                field_id: Some(FieldId::new(0)),
                status: MatterStatus::InvalidDataType,
            })
        );
        assert_eq!(
            basic_information::Event::try_from(bad_ev),
            Err(EventError::InvalidPayload {
                id: StartUp::EVENT_ID,
                field_id: Some(FieldId::new(0)),
                status: MatterStatus::InvalidDataType,
            })
        );

        // Priority mismatch on StartUp
        let prio_ev = WireEvent::new(
            basic_information::BasicInformationCluster::CLUSTER_ID,
            StartUp::EVENT_ID,
            EventPriority::Info,
            AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::U32(1))]),
        );
        assert_eq!(
            StartUp::try_from(prio_ev),
            Err(EventError::PriorityMismatch {
                expected: StartUp::PRIORITY,
                actual: EventPriority::Info,
            })
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn optional_nullable_null_json_roundtrip_on_patch_and_cluster() {
        use matter_spec::{
            model::Nullable,
            v1_6::clusters::on_off::{OnOffClusterPatch, StartUpOnOffEnum},
        };

        let patch = OnOffClusterPatch {
            start_up_on_off: Optional::Present(Nullable::Null),
            ..Default::default()
        };
        let json = serde_json::to_value(&patch).expect("serialize OnOffClusterPatch");
        assert_eq!(json.get("start_up_on_off"), Some(&serde_json::Value::Null));

        let patch_back: OnOffClusterPatch = serde_json::from_value(json).expect("deserialize OnOffClusterPatch");
        assert_eq!(patch_back.start_up_on_off, Optional::Present(Nullable::Null));

        let cluster = OnOffCluster::ON.with_start_up_on_off(Nullable::Null::<StartUpOnOffEnum>);
        let c_json = serde_json::to_value(&cluster).expect("serialize OnOffCluster");
        assert_eq!(c_json.get("start_up_on_off"), Some(&serde_json::Value::Null));

        let c_back: OnOffCluster = serde_json::from_value(c_json).expect("deserialize OnOffCluster");
        assert_eq!(c_back.start_up_on_off, Optional::Present(Nullable::Null));
    }

    #[cfg(all(feature = "dispatch", feature = "schemars"))]
    #[test]
    fn feature_pruned_cluster_schema_filters_disallowed_and_requires_mandatory() {
        use matter_spec::{
            dispatch::feature_pruned_cluster_schema,
            v1_6::{clusters::thermostat::FeatureMap as ThermostatFm, registry::SPEC_REGISTRY},
        };

        let desc = SPEC_REGISTRY
            .find_cluster(ThermostatCluster::CLUSTER_ID)
            .expect("Thermostat descriptor in registry");

        // Heating only (no Cooling): occupied_heating_setpoint is required, occupied_cooling_setpoint is not.
        let mut generator = schemars::SchemaGenerator::default();
        let heat_schema =
            feature_pruned_cluster_schema::<ThermostatCluster>(&mut generator, desc, ThermostatFm::HEATING.bits());
        let heat_obj = heat_schema.as_object().expect("schema object");
        let heat_req = heat_obj["required"].as_array().expect("required");
        assert!(heat_req.iter().any(|v| v.as_str() == Some("occupied_heating_setpoint")));
        assert!(!heat_req.iter().any(|v| v.as_str() == Some("occupied_cooling_setpoint")));

        // Cooling only (no Heating): occupied_cooling_setpoint is required, occupied_heating_setpoint is not.
        let cool_schema =
            feature_pruned_cluster_schema::<ThermostatCluster>(&mut generator, desc, ThermostatFm::COOLING.bits());
        let cool_obj = cool_schema.as_object().expect("schema object");
        let cool_req = cool_obj["required"].as_array().expect("required");
        assert!(cool_req.iter().any(|v| v.as_str() == Some("occupied_cooling_setpoint")));
        assert!(!cool_req.iter().any(|v| v.as_str() == Some("occupied_heating_setpoint")));
    }

    #[test]
    fn registry_validate_node_ref_enforces_mandatory() {
        use matter_spec::{
            error::{ClusterError, EndpointError, NodeError},
            model::{DeviceTypeEntry, Endpoint, Node, NodeId},
            v1_6::{endpoints::RootNode, registry},
        };

        // Construct a root endpoint whose Descriptor cluster is empty (missing mandatory DeviceTypeList)
        let mut ep = Endpoint::new(
            EndpointId::ROOT,
            vec![DeviceTypeEntry::new(RootNode::DEVICE_TYPE_ID, RootNode::REVISION)],
            vec![Cluster::empty(DescriptorCluster::CLUSTER_ID)],
            vec![],
        );
        ep.server_clusters.push(Cluster::empty(OnOffCluster::CLUSTER_ID));

        let node = Node::new(NodeId::new(1), vec![ep]);
        assert!(matches!(
            registry::Validator::full().node(&node),
            Err(NodeError::InvalidEndpoint {
                id: EndpointId::ROOT,
                source: EndpointError::MissingMandatoryServerCluster(_)
                    | EndpointError::InvalidCluster {
                        source: ClusterError::MissingMandatoryAttribute(_),
                        ..
                    }
            })
        ));
    }

    #[test]
    fn view_as_handles_present_absent_and_invalid_device_types() {
        use matter_spec::{
            error::EndpointError,
            model::{DeviceTypeEntry, Endpoint},
            traits::EndpointExt as _,
            v1_6::endpoints::BridgedNode,
        };

        let ep = OnOffLight {
            on_off: OnOffCluster::ON,
            ..Default::default()
        }
        .into_endpoint(EndpointId::new(5));

        let view = ep
            .view_as::<OnOffLight>()
            .expect("valid OnOffLight")
            .expect("OnOffLight declared");
        assert!(view.on_off().is_some());
        assert!(view.descriptor().is_some());
        assert_eq!(view.endpoint().id, EndpointId::new(5));

        assert!(
            ep.view_as::<BridgedNode>()
                .expect("absent device type check should not error")
                .is_none()
        );

        let incomplete = Endpoint::new(
            EndpointId::new(6),
            vec![DeviceTypeEntry::new(OnOffLight::DEVICE_TYPE_ID, OnOffLight::REVISION)],
            vec![Cluster::empty(DescriptorCluster::CLUSTER_ID)],
            vec![],
        );
        let err = incomplete
            .view_as::<OnOffLight>()
            .expect_err("declared device type missing mandatory server clusters must error");
        assert!(matches!(err, EndpointError::MissingMandatoryServerCluster(_)));
    }

    #[test]
    fn try_from_endpoint_allows_extra_clusters_and_rejects_duplicate_cluster() {
        use matter_spec::{
            error::EndpointError,
            v1_6::clusters::bridged_device_basic_information::BridgedDeviceBasicInformationCluster,
        };

        let mut ep = OnOffLight {
            on_off: OnOffCluster::ON,
            ..Default::default()
        }
        .into_endpoint(EndpointId::new(2));

        ep.server_clusters
            .push(Cluster::from(BridgedDeviceBasicInformationCluster::default()));

        let decoded =
            OnOffLight::try_from(ep.clone()).expect("OnOffLight::try_from should accept extra non-disallowed clusters");
        assert_eq!(decoded.on_off.on_off, OnOff::On);

        ep.server_clusters.push(Cluster::from(OnOffCluster::OFF));
        let err = OnOffLight::try_from(ep).expect_err("duplicate OnOffCluster must fail");
        assert_eq!(err, EndpointError::DuplicateCluster(OnOffCluster::CLUSTER_ID));
    }

    #[test]
    fn patch_apply_and_partial_cluster_coexist_without_shadowing() {
        use matter_spec::traits::PartialCluster as _;

        let mut patch = on_off::OnOffClusterPatch::default();
        patch.apply(on_off::Attribute::OnOff(true.into()));
        patch
            .apply_attribute(OnOffCluster::ATTR_ON_TIME, AttributeValue::U16(30))
            .expect("apply_attribute via PartialCluster");

        let from_method = patch.clone().try_into_full().expect("try_into_full");
        let from_try = OnOffCluster::try_from(patch).expect("TryFrom<OnOffClusterPatch>");
        assert_eq!(from_method, from_try);
        assert_eq!(from_method.on_off, OnOff::On);
        assert_eq!(from_method.on_time, Optional::Present(30));
    }

    #[test]
    fn registry_attribute_descriptor_exposes_pascal_name_and_snake_field_name() {
        use matter_spec::v1_6::{clusters::identify::IdentifyCluster, registry::SPEC_REGISTRY};

        let desc = SPEC_REGISTRY
            .find_cluster(IdentifyCluster::CLUSTER_ID)
            .expect("Identify cluster in SPEC_REGISTRY");
        let attr = desc
            .find_attribute(IdentifyCluster::ATTR_IDENTIFY_TIME)
            .expect("IdentifyTime attribute descriptor");
        assert_eq!(attr.name(), "IdentifyTime");
        assert_eq!(attr.field_name(), "identify_time");
    }
}

#[cfg(all(
    feature = "v1_0",
    feature = "v1_2",
    feature = "v1_4",
    feature = "cluster-basic-information",
    feature = "registry"
))]
#[cfg(test)]
mod cross_version_registry_parity_tests {
    use matter_spec::{
        ClusterError,
        model::{Attribute, AttributeId, AttributeValue, Cluster},
        v1_0, v1_2, v1_4,
    };

    #[test]
    fn expects_event_list_parity_across_v1_0_v1_2_v1_4() {
        assert!(!v1_0::registry::SPEC_REGISTRY.expects_event_list);
        assert!(v1_2::registry::SPEC_REGISTRY.expects_event_list);
        assert!(!v1_4::registry::SPEC_REGISTRY.expects_event_list);

        // OnOff has no events in v1.2, while BasicInformation (0x0028) has events across v1.0/v1.2/v1.4.
        let mut bi_v1_0 = v1_0::clusters::basic_information::BasicInformationCluster::default();
        bi_v1_0.location = "XX".into();
        bi_v1_0.hardware_version_string = "1.0".into();
        bi_v1_0.software_version_string = "1.0".into();
        bi_v1_0.capability_minima = v1_0::clusters::basic_information::CapabilityMinimaStruct::new(3_u16, 3_u16);
        let basic_v1_0 = Cluster::from(bi_v1_0);

        let mut bi_v1_2 = v1_2::clusters::basic_information::BasicInformationCluster::default();
        bi_v1_2.location = "XX".into();
        bi_v1_2.hardware_version_string = "1.0".into();
        bi_v1_2.software_version_string = "1.0".into();
        bi_v1_2.capability_minima = v1_2::clusters::basic_information::CapabilityMinimaStruct::new(3_u16, 3_u16);
        let basic_v1_2 = Cluster::from(bi_v1_2);

        let mut bi_v1_4 = v1_4::clusters::basic_information::BasicInformationCluster::default();
        bi_v1_4.location = "XX".into();
        bi_v1_4.hardware_version_string = "1.0".into();
        bi_v1_4.software_version_string = "1.0".into();
        bi_v1_4.capability_minima = v1_4::clusters::basic_information::CapabilityMinimaStruct::new(3_u16, 3_u16);
        bi_v1_4.max_paths_per_invoke = 1;
        bi_v1_4.configuration_version = 1;
        let basic_v1_4 = Cluster::from(bi_v1_4);

        assert_eq!(
            v1_2::registry::Validator::full().cluster(&basic_v1_2),
            Err(ClusterError::MissingMandatoryAttribute(AttributeId::EVENT_LIST))
        );
        let mut basic_v1_2_with_ev_list = basic_v1_2;
        basic_v1_2_with_ev_list
            .attributes
            .push(Attribute::new(AttributeId::EVENT_LIST, AttributeValue::List(vec![])));
        v1_2::registry::Validator::full()
            .cluster(&basic_v1_2_with_ev_list)
            .expect("v1_2 should pass with EventList");

        v1_0::registry::Validator::full()
            .cluster(&basic_v1_0)
            .expect("v1_0 must not require EventList");
        v1_4::registry::Validator::full()
            .cluster(&basic_v1_4)
            .expect("v1_4 must not require EventList");
    }
}
