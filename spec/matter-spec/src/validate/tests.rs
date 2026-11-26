use alloc::{string::String, vec};

use super::{descriptor_checks::descriptor_attr_error, value::validate_value_with_ctx, *};
use crate::{
    error::{AttributeError, ClusterError, CommandError, EndpointError, EventError, NodeError},
    model::{
        Attribute, AttributeId, AttributeValue, Cluster, ClusterId, Command, CommandDirection, CommandId, DataVersion,
        DeviceTypeEntry, DeviceTypeId, Endpoint, EndpointId, Event, EventId, EventPriority, FieldId, MatterStatus,
        Node, NodeId, Optional,
    },
};

static NESTED_STRUCT: StructDescriptor = StructDescriptor {
    name: "NestedStruct",
    fields: &[FieldDescriptor {
        id: FieldId::new(1),
        name: "Count",
        constraint: ValueConstraint {
            value_type: ValueType::U8,
            bounds: ValueBounds::Unsigned { min: 1, max: 10 },
            nullable: false,
        },
        conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
    }],
};

static OUTER_STRUCT: StructDescriptor = StructDescriptor {
    name: "OuterStruct",
    fields: &[
        FieldDescriptor {
            id: FieldId::new(0),
            name: "Label",
            constraint: ValueConstraint {
                value_type: ValueType::String,
                bounds: ValueBounds::StringLen { min: 2, max: 8 },
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
        },
        FieldDescriptor {
            id: FieldId::new(1),
            name: "Nested",
            constraint: ValueConstraint {
                value_type: ValueType::Struct(&NESTED_STRUCT),
                bounds: ValueBounds::None,
                nullable: true,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
        },
    ],
};

static ITEM_U8_CONSTRAINT: ValueConstraint = ValueConstraint {
    value_type: ValueType::U8,
    bounds: ValueBounds::Unsigned { min: 1, max: 5 },
    nullable: false,
};

static LIST_CONSTRAINT: ListLength = ListLength {
    min: 1,
    max: 3,
    item_constraint: &ITEM_U8_CONSTRAINT,
};

static MOCK_CLUSTERS: &[ClusterDescriptor] = &[ClusterDescriptor {
    id: ClusterId::new(0x0006),
    name: "OnOff",
    attributes: &[
        AttributeDescriptor {
            id: AttributeId::new(0x0000),
            name: "OnOff",
            field_name: "on_off",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
        },
        AttributeDescriptor {
            id: AttributeId::new(0x0001),
            name: "FeatureAttr",
            field_name: "feature_attr",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::U8,
                bounds: ValueBounds::Unsigned { min: 0, max: 100 },
                nullable: false,
            },
            conformance: &[
                ConformanceOp::PushFeature(0x01),
                ConformanceOp::Mandatory,
                ConformanceOp::PushBool(true),
                ConformanceOp::Disallow,
                ConformanceOp::Otherwise,
            ],
        },
        AttributeDescriptor {
            id: AttributeId::new(0x0002),
            name: "ProvisionalAttr",
            field_name: "provisional_attr",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Provisional],
        },
        AttributeDescriptor {
            id: AttributeId::new(0x0003),
            name: "GreaterAttr",
            field_name: "greater_attr",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false,
            },
            conformance: &[
                ConformanceOp::PushAttrValue(0x0001),
                ConformanceOp::PushConst(10),
                ConformanceOp::Greater,
                ConformanceOp::Mandatory,
            ],
        },
        AttributeDescriptor {
            id: AttributeId::new(0x0004),
            name: "ChoiceA",
            field_name: "choice_a",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
        },
        AttributeDescriptor {
            id: AttributeId::new(0x0005),
            name: "ChoiceB",
            field_name: "choice_b",
            access: AccessFlags::READ,
            constraint: ValueConstraint {
                value_type: ValueType::Any,
                bounds: ValueBounds::None,
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
        },
    ],
    commands: &[CommandDescriptor {
        id: CommandId::new(0x00),
        name: "Off",
        direction: CommandDirection::ClientToServer,
        access: AccessFlags::READ,
        fields: &[FieldDescriptor {
            id: FieldId::new(0),
            name: "Arg",
            constraint: ValueConstraint {
                value_type: ValueType::U8,
                bounds: ValueBounds::Unsigned { min: 0, max: 10 },
                nullable: false,
            },
            conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
        }],
        conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
    }],
    events: &[EventDescriptor {
        id: EventId::new(0x00),
        name: "StateChanged",
        priority: EventPriority::Info,
        access: AccessFlags::READ,
        fields: &[],
        conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Optional],
    }],
    choice_groups: &[ChoiceGroup {
        min: 1,
        max: 1,
        more: false,
        members: &[0x0004, 0x0005],
    }],
}];

static MOCK_DEVICE_TYPES: &[DeviceTypeDescriptor] = &[DeviceTypeDescriptor {
    id: DeviceTypeId::new(0x0100),
    name: "OnOffLight",
    server_clusters: &[DeviceClusterReq {
        cluster_id: ClusterId::new(0x0006),
        conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
    }],
    client_clusters: &[],
    choice_groups: &[],
    includes: &[DeviceTypeIncludeItem {
        cluster_id: ClusterId::new(0x0006),
        is_server: true,
        required_attributes: &[AttributeId::new(0x0000)],
        required_commands: &[CommandId::new(0x00)],
        required_events: &[],
    }],
}];

fn mock_registry(expects_event_list: bool) -> SpecRegistry {
    SpecRegistry {
        expects_event_list,
        clusters: MOCK_CLUSTERS,
        device_types: MOCK_DEVICE_TYPES,
    }
}

#[test]
fn rpn_conformance_feature_disallow_provisional_and_greater() {
    let reg = mock_registry(false);
    let mut cluster = Cluster::empty(ClusterId::new(0x0006));
    cluster.data_version = Optional::Present(DataVersion::new(42));
    cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
    cluster.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x00));

    // FeatureAttr (0x0001) is disallowed when feature bit 0x01 is 0.
    cluster.set_attribute(AttributeId::new(0x0001), AttributeValue::U8(5));
    assert_eq!(
        reg.validator().cluster(&cluster),
        Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
            AttributeId::new(0x0001)
        )))
    );

    // Enable feature bit 0x01 -> FeatureAttr becomes mandatory and valid.
    cluster.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x01));
    reg.validator().cluster(&cluster).unwrap();

    // Set FeatureAttr = 15 (> 10) -> GreaterAttr (0x0003) becomes mandatory.
    cluster.set_attribute(AttributeId::new(0x0001), AttributeValue::U8(15));
    assert_eq!(
        reg.validator().cluster(&cluster),
        Err(ClusterError::MissingMandatoryAttribute(AttributeId::new(0x0003)))
    );
    cluster.set_attribute(AttributeId::new(0x0003), AttributeValue::U8(1));
    reg.validator().cluster(&cluster).unwrap();

    // ProvisionalAttr: accepted when enforce_provisional = false, rejected when true.
    cluster.set_attribute(AttributeId::new(0x0002), AttributeValue::U8(9));
    reg.validator().cluster(&cluster).unwrap();
    assert_eq!(
        reg.validate_cluster_ref_with_mode(
            &cluster,
            ValidationMode::FullSnapshot {
                enforce_provisional: true
            }
        ),
        Err(ClusterError::InvalidAttribute(AttributeError::UnsupportedAttribute(
            AttributeId::new(0x0002)
        )))
    );
}

#[test]
fn choice_group_underflow_and_overflow() {
    let reg = mock_registry(false);
    let mut cluster = Cluster::empty(ClusterId::new(0x0006));
    cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));

    // Neither ChoiceA nor ChoiceB -> underflow.
    assert!(reg.validator().cluster(&cluster).is_err());

    // Both ChoiceA and ChoiceB -> overflow (max = 1).
    cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
    cluster.set_attribute(AttributeId::new(0x0005), AttributeValue::U8(1));
    assert!(reg.validator().cluster(&cluster).is_err());
}

#[test]
fn validate_value_bounds_innermost_field_id_and_status_codes() {
    let constraint = ValueConstraint {
        value_type: ValueType::Struct(&OUTER_STRUCT),
        bounds: ValueBounds::None,
        nullable: false,
    };

    // Wrong wire tag -> InvalidDataType (0x8D)
    assert_eq!(
        validate_value(&AttributeValue::U8(1), &constraint),
        Err((None, MatterStatus::InvalidDataType))
    );

    // Below minLength string -> ConstraintError (0x87) with FieldId(0)
    let bad_str = AttributeValue::Struct(vec![(FieldId::new(0), AttributeValue::String(String::from("a")))]);
    assert_eq!(
        validate_value(&bad_str, &constraint),
        Err((Some(FieldId::new(0)), MatterStatus::ConstraintError))
    );

    // Nested struct out-of-range integer -> innermost FieldId(1)
    let bad_nested = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::String(String::from("ok"))),
        (
            FieldId::new(1),
            AttributeValue::Struct(vec![(FieldId::new(1), AttributeValue::U8(99))]),
        ),
    ]);
    assert_eq!(
        validate_value(&bad_nested, &constraint),
        Err((Some(FieldId::new(1)), MatterStatus::ConstraintError))
    );

    // Unknown struct FieldId -> ConstraintError
    let unknown_field = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::String(String::from("ok"))),
        (FieldId::new(99), AttributeValue::U8(1)),
    ]);
    assert_eq!(
        validate_value(&unknown_field, &constraint),
        Err((Some(FieldId::new(99)), MatterStatus::ConstraintError))
    );

    // ListLength outer & item constraints
    let list_c = ValueConstraint {
        value_type: ValueType::List(&LIST_CONSTRAINT),
        bounds: ValueBounds::None,
        nullable: false,
    };
    assert_eq!(
        validate_value(&AttributeValue::List(vec![]), &list_c),
        Err((None, MatterStatus::ConstraintError))
    );
    assert_eq!(
        validate_value(&AttributeValue::List(vec![AttributeValue::U8(9)]), &list_c),
        Err((None, MatterStatus::ConstraintError))
    );
    validate_value(&AttributeValue::List(vec![AttributeValue::U8(3)]), &list_c).unwrap();
}

#[test]
fn command_missing_mandatory_field_returns_invalid_command() {
    let reg = mock_registry(false);
    let cmd = Command::unit(
        ClusterId::new(0x0006),
        CommandId::new(0x00),
        CommandDirection::ClientToServer,
    );
    assert_eq!(
        reg.validator().command(&cmd),
        Err(CommandError::InvalidPayload {
            id: CommandId::new(0x00),
            field_id: Some(FieldId::new(0)),
            status: MatterStatus::InvalidCommand,
        })
    );
}

#[test]
fn event_list_version_gating_and_non_mutating_endpoint() {
    let mut cluster = Cluster::empty(ClusterId::new(0x0006));
    cluster.data_version = Optional::Present(DataVersion::new(7));
    cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
    cluster.set_attribute(
        AttributeId::ACCEPTED_COMMAND_LIST,
        AttributeValue::List(vec![AttributeValue::U32(0x00)]),
    );

    // Registry expecting EventList (0xFFFA) requires it when cluster has events.
    let reg_with_ev_list = mock_registry(true);
    assert_eq!(
        reg_with_ev_list.validator().cluster(&cluster),
        Err(ClusterError::MissingMandatoryAttribute(AttributeId::EVENT_LIST))
    );

    // Registry not expecting EventList does not require it.
    let reg_no_ev_list = mock_registry(false);
    let ep = Endpoint::empty(EndpointId::new(1))
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(cluster)
        // Extra/MEI cluster not in registry is permitted.
        .with_cluster(Cluster::empty(ClusterId::new(0xFFF1_0001)));

    reg_no_ev_list.validator().endpoint(&ep).unwrap();

    // Event validation preserves metadata.
    let ev = Event::unit(ClusterId::new(0x0006), EventId::new(0x00), EventPriority::Info);
    reg_no_ev_list.validator().event(&ev).unwrap();
}

#[test]
fn validate_struct_payload_rejects_disallowed_fields() {
    static DISALLOW_STRUCT: StructDescriptor = StructDescriptor {
        name: "DisallowStruct",
        fields: &[
            FieldDescriptor {
                id: FieldId::new(0),
                name: "AlwaysAllowed",
                constraint: ValueConstraint {
                    value_type: ValueType::U8,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Mandatory],
            },
            FieldDescriptor {
                id: FieldId::new(1),
                name: "NeverAllowed",
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[ConformanceOp::PushBool(true), ConformanceOp::Disallow],
            },
            FieldDescriptor {
                id: FieldId::new(2),
                name: "FeatureOnly",
                constraint: ValueConstraint {
                    value_type: ValueType::Any,
                    bounds: ValueBounds::None,
                    nullable: false,
                },
                conformance: &[
                    ConformanceOp::PushFeature(0x01),
                    ConformanceOp::Optional,
                    ConformanceOp::PushBool(true),
                    ConformanceOp::Disallow,
                    ConformanceOp::Otherwise,
                ],
            },
        ],
    };

    let constraint = ValueConstraint {
        value_type: ValueType::Struct(&DISALLOW_STRUCT),
        bounds: ValueBounds::None,
        nullable: false,
    };

    // Static Disallow field must be rejected with ConstraintError and its FieldId.
    let with_disallowed = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::U8(1)),
        (FieldId::new(1), AttributeValue::U8(2)),
    ]);
    assert_eq!(
        validate_value(&with_disallowed, &constraint),
        Err((Some(FieldId::new(1)), MatterStatus::ConstraintError))
    );

    // Feature-conditional Disallow field must be rejected when feature bit is unset,
    // and accepted when feature bit is set.
    let with_feature_field = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::U8(1)),
        (FieldId::new(2), AttributeValue::U8(3)),
    ]);
    assert_eq!(
        validate_value_with_ctx(&with_feature_field, &constraint, 0x00, None, IntegerMode::Widening),
        Err((Some(FieldId::new(2)), MatterStatus::ConstraintError))
    );
    validate_value_with_ctx(&with_feature_field, &constraint, 0x01, None, IntegerMode::Widening).unwrap();
}

#[test]
fn validate_node_ref_enforces_mandatory_under_full_snapshot() {
    let reg = mock_registry(false);

    let root_ep = Endpoint::empty(EndpointId::ROOT)
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        // OnOff cluster is missing mandatory OnOff attribute (0x0000) and ChoiceA/ChoiceB.
        .with_cluster(Cluster::empty(ClusterId::new(0x0006)));

    let node = Node::new(NodeId::new(1), vec![root_ep]);
    assert_eq!(
        reg.validator().node(&node),
        Err(NodeError::InvalidEndpoint {
            id: EndpointId::ROOT,
            source: EndpointError::InvalidCluster {
                id: ClusterId::new(0x0006),
                source: ClusterError::MissingMandatoryAttribute(AttributeId::new(0x0000)),
            },
        })
    );
}

#[test]
fn validator_builder_and_descriptor_accessors() {
    static ATTRS: [AttributeDescriptor; 1] = [AttributeDescriptor {
        id: AttributeId::new(0x0000),
        name: "OnOff",
        field_name: "on_off",
        access: AccessFlags::READ,
        constraint: ValueConstraint {
            value_type: ValueType::Any,
            bounds: ValueBounds::None,
            nullable: false,
        },
        conformance: &[ConformanceOp::Mandatory],
    }];
    static CMDS: [CommandDescriptor; 1] = [CommandDescriptor {
        id: CommandId::new(0x0000),
        name: "Off",
        direction: CommandDirection::ClientToServer,
        access: AccessFlags::empty(),
        fields: &[],
        conformance: &[ConformanceOp::Mandatory],
    }];
    static EVENTS: [EventDescriptor; 1] = [EventDescriptor {
        id: EventId::new(0x0000),
        name: "StateChanged",
        priority: EventPriority::Info,
        access: AccessFlags::READ,
        fields: &[],
        conformance: &[ConformanceOp::Optional],
    }];
    static CLUSTERS: [ClusterDescriptor; 1] = [ClusterDescriptor {
        id: ClusterId::new(0x0006),
        name: "OnOff",
        attributes: &ATTRS,
        commands: &CMDS,
        events: &EVENTS,
        choice_groups: &[],
    }];
    static DEVICE_TYPES: [DeviceTypeDescriptor; 1] = [DeviceTypeDescriptor {
        id: DeviceTypeId::new(0x0100),
        name: "OnOffLight",
        server_clusters: &[],
        client_clusters: &[],
        choice_groups: &[],
        includes: &[],
    }];
    static REG: SpecRegistry = SpecRegistry {
        expects_event_list: false,
        clusters: &CLUSTERS,
        device_types: &DEVICE_TYPES,
    };

    assert_eq!(
        ValidationMode::default(),
        ValidationMode::FullSnapshot {
            enforce_provisional: false
        }
    );

    let v = REG.validator();
    let v_copy = v;
    assert_eq!(
        v_copy.mode(),
        ValidationMode::FullSnapshot {
            enforce_provisional: false
        }
    );
    assert_eq!(
        v.enforce_provisional(true).mode(),
        ValidationMode::FullSnapshot {
            enforce_provisional: true
        }
    );
    assert_eq!(v.partial_mode().mode(), ValidationMode::Partial);
    assert_eq!(
        REG.partial_validator().enforce_provisional(true).mode(),
        ValidationMode::Partial
    );
    assert_eq!(
        REG.partial_validator().full_mode().mode(),
        ValidationMode::FullSnapshot {
            enforce_provisional: false
        }
    );
    assert_eq!(
        RegistryValidator::partial(&REG)
            .with_mode(ValidationMode::FullSnapshot {
                enforce_provisional: true
            })
            .mode(),
        ValidationMode::FullSnapshot {
            enforce_provisional: true
        }
    );

    assert_eq!(REG.clusters().len(), 1);
    assert_eq!(REG.device_types().len(), 1);
    let cluster = REG.find_cluster(ClusterId::new(0x0006)).unwrap();
    assert_eq!(cluster.id(), ClusterId::new(0x0006));
    assert_eq!(cluster.name(), "OnOff");
    assert_eq!(cluster.attributes().len(), 1);
    assert_eq!(cluster.commands().len(), 1);
    assert_eq!(cluster.events().len(), 1);

    let attr = cluster.find_attribute(AttributeId::new(0x0000)).unwrap();
    assert_eq!(attr.id(), AttributeId::new(0x0000));
    assert_eq!(attr.name(), "OnOff");
    assert_eq!(attr.field_name(), "on_off");
    assert_eq!(attr.access(), AccessFlags::READ);

    let cmd = cluster
        .find_command(CommandId::new(0x0000), CommandDirection::ClientToServer)
        .unwrap();
    assert_eq!(cmd.id(), CommandId::new(0x0000));
    assert_eq!(cmd.name(), "Off");
    assert_eq!(cmd.direction(), CommandDirection::ClientToServer);
    assert_eq!(cmd.access(), AccessFlags::empty());

    let ev = cluster.find_event(EventId::new(0x0000)).unwrap();
    assert_eq!(ev.id(), EventId::new(0x0000));
    assert_eq!(ev.name(), "StateChanged");
    assert_eq!(ev.priority(), EventPriority::Info);
    assert_eq!(ev.access(), AccessFlags::READ);

    assert_eq!(
        REG.validator().attribute(
            ClusterId::new(0x9999),
            &Attribute::new(AttributeId::new(0x0000), AttributeValue::Bool(true)),
        ),
        Err(AttributeError::UnsupportedCluster(ClusterId::new(0x9999)))
    );
    assert_eq!(
        REG.validator().event(&Event::unit(
            ClusterId::new(0x0006),
            EventId::new(0x0000),
            EventPriority::Critical,
        )),
        Err(EventError::PriorityMismatch {
            expected: EventPriority::Info,
            actual: EventPriority::Critical,
        })
    );

    let dt = REG.find_device_type(DeviceTypeId::new(0x0100)).unwrap();
    assert_eq!(dt.id(), DeviceTypeId::new(0x0100));
    assert_eq!(dt.name(), "OnOffLight");

    let flags = AccessFlags::from_bits_truncate(0xFF);
    assert!(flags.contains(AccessFlags::READ));
    assert_eq!(AccessFlags::READ.bits(), 1 << 0);
}

#[test]
fn access_flags_bitmap_ops() {
    assert!(AccessFlags::EMPTY.is_empty());
    assert_eq!(AccessFlags::EMPTY, AccessFlags::empty());

    let mut flags = AccessFlags::READ | AccessFlags::WRITE;
    assert!(!flags.is_empty());
    assert!(flags.contains(AccessFlags::READ));
    assert!(flags.contains(AccessFlags::WRITE));
    assert_eq!(
        flags.union(AccessFlags::TIMED),
        AccessFlags::READ | AccessFlags::WRITE | AccessFlags::TIMED
    );
    assert_eq!(flags.intersection(AccessFlags::READ), AccessFlags::READ);
    assert_eq!(u8::from(flags), 0x03);
    assert_eq!(alloc::format!("{flags}"), "0x03");

    flags.insert(AccessFlags::SCENE);
    assert!(flags.contains(AccessFlags::SCENE));
    flags.remove(AccessFlags::WRITE);
    assert!(!flags.contains(AccessFlags::WRITE));
    assert_eq!(flags ^ AccessFlags::READ, AccessFlags::SCENE);
    assert!((!AccessFlags::READ).contains(AccessFlags::WRITE));
    assert!(!(!AccessFlags::READ).contains(AccessFlags::READ));
}

#[test]
fn global_attribute_wire_types_validated_on_attribute_and_cluster() {
    let reg = mock_registry(false);
    let on_off = ClusterId::new(0x0006);

    // Valid §7.13 globals pass single-attribute validation.
    for (id, val) in [
        (
            AttributeId::GENERATED_COMMAND_LIST,
            AttributeValue::List(vec![AttributeValue::U32(0)]),
        ),
        (
            AttributeId::ACCEPTED_COMMAND_LIST,
            AttributeValue::List(vec![AttributeValue::U32(0)]),
        ),
        (
            AttributeId::EVENT_LIST,
            AttributeValue::List(vec![AttributeValue::U32(0)]),
        ),
        (
            AttributeId::ATTRIBUTE_LIST,
            AttributeValue::List(vec![AttributeValue::U32(0)]),
        ),
        (AttributeId::FEATURE_MAP, AttributeValue::U32(0)),
        (AttributeId::CLUSTER_REVISION, AttributeValue::U16(1)),
    ] {
        reg.validator()
            .attribute(on_off, &Attribute::new(id, val))
            .expect("valid global attribute should pass");
    }

    // Wrong wire tags and narrowing under default Widening return InvalidDataType.
    for (id, bad_val) in [
        (AttributeId::FEATURE_MAP, AttributeValue::Bool(true)),
        (AttributeId::CLUSTER_REVISION, AttributeValue::U32(1)),
        (AttributeId::ACCEPTED_COMMAND_LIST, AttributeValue::U8(0)),
        (
            AttributeId::ATTRIBUTE_LIST,
            AttributeValue::List(vec![AttributeValue::U64(0)]),
        ),
        (
            AttributeId::EVENT_LIST,
            AttributeValue::List(vec![AttributeValue::String(String::from("x"))]),
        ),
    ] {
        assert_eq!(
            reg.validator().attribute(on_off, &Attribute::new(id, bad_val.clone())),
            Err(AttributeError::InvalidValue {
                id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            })
        );

        let mut cluster = Cluster::empty(on_off);
        cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
        cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
        cluster.set_attribute(id, bad_val);
        assert_eq!(
            reg.validator().cluster(&cluster),
            Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id,
                field_id: None,
                status: MatterStatus::InvalidDataType,
            }))
        );
    }
}

#[test]
fn endpoint_wildcard_parts_list_tag_list_and_server_list_checks() {
    let reg = mock_registry(false);

    let mut on_off = Cluster::empty(ClusterId::new(0x0006));
    on_off.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    on_off.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));

    // Wildcard EndpointId (0xFFFF) rejected.
    let wildcard_ep = Endpoint::empty(EndpointId::WILDCARD)
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(on_off.clone());
    assert_eq!(
        reg.validator().endpoint(&wildcard_ep),
        Err(EndpointError::InvalidEndpointId(EndpointId::WILDCARD))
    );

    let base_ep = Endpoint::empty(EndpointId::new(1))
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(on_off.clone());

    // PartsList: non-list -> InvalidDataType, non-U16 -> InvalidDataType.
    let mut bad_parts_type = base_ep.clone();
    bad_parts_type
        .descriptor_mut()
        .unwrap()
        .set_attribute(AttributeId::new(0x0003), AttributeValue::U16(2));
    assert_eq!(
        reg.validator().endpoint(&bad_parts_type),
        Err(EndpointError::descriptor_attr_status(
            0x0003,
            MatterStatus::InvalidDataType
        ))
    );

    let mut bad_parts_elem = base_ep.clone();
    bad_parts_elem.descriptor_mut().unwrap().set_attribute(
        AttributeId::new(0x0003),
        AttributeValue::List(vec![AttributeValue::U32(2)]),
    );
    assert_eq!(
        reg.validator().endpoint(&bad_parts_elem),
        Err(EndpointError::descriptor_attr_status(
            0x0003,
            MatterStatus::InvalidDataType
        ))
    );

    // PartsList: wildcard, root endpoint (0), self-reference, duplicate -> ConstraintError.
    for invalid_parts in [
        vec![AttributeValue::U16(0xFFFF)],
        vec![AttributeValue::U16(0)],
        vec![AttributeValue::U16(1)],
        vec![AttributeValue::U16(2), AttributeValue::U16(2)],
    ] {
        let mut ep = base_ep.clone();
        ep.descriptor_mut()
            .unwrap()
            .set_attribute(AttributeId::new(0x0003), AttributeValue::List(invalid_parts));
        assert_eq!(reg.validator().endpoint(&ep), Err(descriptor_attr_error(0x0003)));
    }

    // ServerList: non-U32 element -> InvalidDataType under Strict; widening U16 passes under default Widening.
    let mut bad_server_list = base_ep.clone();
    bad_server_list.descriptor_mut().unwrap().set_attribute(
        AttributeId::new(0x0001),
        AttributeValue::List(vec![AttributeValue::U16(0x001D), AttributeValue::U16(0x0006)]),
    );
    reg.validator().endpoint(&bad_server_list).unwrap();
    assert_eq!(
        reg.validator()
            .with_integer_mode(IntegerMode::Strict)
            .endpoint(&bad_server_list),
        Err(EndpointError::descriptor_attr_status(
            0x0001,
            MatterStatus::InvalidDataType
        ))
    );

    // TagList: duplicate semantic tags (even with reordered fields) -> ConstraintError.
    let tag_a = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::Null),
        (FieldId::new(1), AttributeValue::U8(1)),
        (FieldId::new(2), AttributeValue::U8(2)),
    ]);
    let tag_a_reordered = AttributeValue::Struct(vec![
        (FieldId::new(2), AttributeValue::U8(2)),
        (FieldId::new(1), AttributeValue::U8(1)),
        (FieldId::new(0), AttributeValue::Null),
    ]);
    let mut dup_tags = base_ep.clone();
    dup_tags.descriptor_mut().unwrap().set_attribute(
        AttributeId::new(0x0004),
        AttributeValue::List(vec![tag_a.clone(), tag_a_reordered]),
    );
    assert_eq!(reg.validator().endpoint(&dup_tags), Err(descriptor_attr_error(0x0004)));

    // TagList: same (mfg_code, namespace_id, tag) with distinct labels passes.
    let tag_label_1 = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::Null),
        (FieldId::new(1), AttributeValue::U8(1)),
        (FieldId::new(2), AttributeValue::U8(2)),
        (FieldId::new(3), AttributeValue::String(String::from("Up"))),
    ]);
    let tag_label_2 = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::Null),
        (FieldId::new(1), AttributeValue::U8(1)),
        (FieldId::new(2), AttributeValue::U8(2)),
        (FieldId::new(3), AttributeValue::String(String::from("Down"))),
    ]);
    let mut distinct_labels = base_ep;
    distinct_labels.descriptor_mut().unwrap().set_attribute(
        AttributeId::new(0x0004),
        AttributeValue::List(vec![tag_label_1, tag_label_2]),
    );
    reg.validator().endpoint(&distinct_labels).unwrap();
}

#[test]
fn node_parts_list_referential_integrity() {
    let reg = mock_registry(false);

    let mut on_off = Cluster::empty(ClusterId::new(0x0006));
    on_off.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    on_off.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));

    let root_dangling = Endpoint::empty(EndpointId::ROOT)
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_parts_list([EndpointId::new(2)])
        .with_cluster(on_off.clone());

    let dangling_node = Node::new(NodeId::new(1), vec![root_dangling.clone()]);
    assert_eq!(
        reg.validator().node(&dangling_node),
        Err(NodeError::InvalidEndpoint {
            id: EndpointId::ROOT,
            source: descriptor_attr_error(0x0003),
        })
    );
    // Partial mode allows PartsList entries outside the partial node snapshot.
    reg.partial_validator().node(&dangling_node).unwrap();

    let child = Endpoint::empty(EndpointId::new(2))
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(on_off);
    let valid_node = Node::new(NodeId::new(1), vec![root_dangling, child]);
    reg.validator().node(&valid_node).unwrap();
}

#[test]
fn include_item_partial_mode_and_malformed_command_list_status() {
    let reg = mock_registry(false);

    // Partial mode must not fail with MissingMandatoryAttribute for include required_attributes.
    let partial_ep = Endpoint::empty(EndpointId::new(1))
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(Cluster::empty(ClusterId::new(0x0006)));
    reg.partial_validator().endpoint(&partial_ep).unwrap();

    // Malformed AcceptedCommandList (non-List and narrowing U64 under Widening) returns InvalidDataType.
    for bad_list in [
        AttributeValue::U32(0),
        AttributeValue::List(vec![AttributeValue::U64(0)]),
    ] {
        let mut on_off = Cluster::empty(ClusterId::new(0x0006));
        on_off.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
        on_off.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
        on_off.set_attribute(AttributeId::ACCEPTED_COMMAND_LIST, bad_list);
        let ep = Endpoint::empty(EndpointId::new(1))
            .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
            .with_cluster(on_off);
        assert_eq!(
            reg.validator().endpoint(&ep),
            Err(EndpointError::InvalidCluster {
                id: ClusterId::new(0x0006),
                source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                    id: AttributeId::ACCEPTED_COMMAND_LIST,
                    field_id: None,
                    status: MatterStatus::InvalidDataType,
                }),
            })
        );
    }

    // Well-typed AcceptedCommandList missing required command 0x00 returns ConstraintError.
    let mut on_off = Cluster::empty(ClusterId::new(0x0006));
    on_off.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    on_off.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
    on_off.set_attribute(
        AttributeId::ACCEPTED_COMMAND_LIST,
        AttributeValue::List(vec![AttributeValue::U32(0x01)]),
    );
    let ep = Endpoint::empty(EndpointId::new(1))
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(on_off);
    assert_eq!(
        reg.validator().endpoint(&ep),
        Err(EndpointError::InvalidCluster {
            id: ClusterId::new(0x0006),
            source: ClusterError::InvalidAttribute(AttributeError::InvalidValue {
                id: AttributeId::ACCEPTED_COMMAND_LIST,
                field_id: None,
                status: MatterStatus::ConstraintError,
            }),
        })
    );
}

#[test]
fn integer_mode_strict_widening_and_fit() {
    let reg = mock_registry(false);
    let on_off = ClusterId::new(0x0006);
    let attr_id = AttributeId::new(0x0001); // U8, bounds 0..=100
    let rev_id = AttributeId::CLUSTER_REVISION; // U16

    let v_default = reg.validator();
    assert_eq!(v_default.integer_mode(), IntegerMode::Widening);
    let v_strict = v_default.with_integer_mode(IntegerMode::Strict);
    let v_fit = v_default.with_integer_mode(IntegerMode::Fit);

    // Strict: exact width only.
    v_strict
        .attribute(on_off, &Attribute::new(rev_id, AttributeValue::U16(1)))
        .unwrap();
    assert_eq!(
        v_strict.attribute(on_off, &Attribute::new(rev_id, AttributeValue::U8(1))),
        Err(AttributeError::InvalidValue {
            id: rev_id,
            field_id: None,
            status: MatterStatus::InvalidDataType,
        })
    );

    // Widening (default): U8 -> U16 ok, U32 -> U16 rejected as InvalidDataType.
    v_default
        .attribute(on_off, &Attribute::new(rev_id, AttributeValue::U8(1)))
        .unwrap();
    assert_eq!(
        v_default.attribute(on_off, &Attribute::new(rev_id, AttributeValue::U32(1))),
        Err(AttributeError::InvalidValue {
            id: rev_id,
            field_id: None,
            status: MatterStatus::InvalidDataType,
        })
    );

    // Fit: U64 fitting in U8 (0..=100) passes; U64(300) overflows U8 -> ConstraintError; signed -> InvalidDataType.
    v_fit
        .attribute(on_off, &Attribute::new(attr_id, AttributeValue::U64(42)))
        .unwrap();
    assert_eq!(
        v_fit.attribute(on_off, &Attribute::new(attr_id, AttributeValue::U64(300))),
        Err(AttributeError::InvalidValue {
            id: attr_id,
            field_id: None,
            status: MatterStatus::ConstraintError,
        })
    );
    assert_eq!(
        v_fit.attribute(on_off, &Attribute::new(attr_id, AttributeValue::I8(42))),
        Err(AttributeError::InvalidValue {
            id: attr_id,
            field_id: None,
            status: MatterStatus::InvalidDataType,
        })
    );
}

#[test]
fn feature_map_integer_mode_controls_conformance() {
    let reg = mock_registry(false);
    let mut cluster = Cluster::empty(ClusterId::new(0x0006));
    cluster.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    cluster.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));
    // Minimal-width U8 FeatureMap enabling bit 0x01 (inserted before FeatureAttr so global type errors surface first).
    cluster.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U8(0x01));
    cluster.set_attribute(AttributeId::new(0x0001), AttributeValue::U8(5));

    // Widening (default) and Fit both extract FeatureMap=0x01 -> FeatureAttr (0x0001) is allowed & mandatory.
    reg.validator().cluster(&cluster).unwrap();
    reg.validator()
        .with_integer_mode(IntegerMode::Fit)
        .cluster(&cluster)
        .unwrap();

    // Strict rejects U8 for FeatureMap (U32) with InvalidDataType.
    assert_eq!(
        reg.validator().with_integer_mode(IntegerMode::Strict).cluster(&cluster),
        Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
            id: AttributeId::FEATURE_MAP,
            field_id: None,
            status: MatterStatus::InvalidDataType,
        }))
    );

    // Narrowing U64 FeatureMap fails Widening as InvalidDataType, but succeeds under Fit.
    let mut cluster_u64 = cluster;
    cluster_u64.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U64(0x01));
    assert_eq!(
        reg.validator().cluster(&cluster_u64),
        Err(ClusterError::InvalidAttribute(AttributeError::InvalidValue {
            id: AttributeId::FEATURE_MAP,
            field_id: None,
            status: MatterStatus::InvalidDataType,
        }))
    );
    reg.validator()
        .with_integer_mode(IntegerMode::Fit)
        .cluster(&cluster_u64)
        .unwrap();
}

#[test]
fn enum_and_bitmap_respect_integer_mode_bit_widths() {
    let enum8 = ValueConstraint {
        value_type: ValueType::Enum {
            bits: 8,
            valid_values: &[0, 1],
        },
        bounds: ValueBounds::None,
        nullable: false,
    };
    let enum16 = ValueConstraint {
        value_type: ValueType::Enum {
            bits: 16,
            valid_values: &[0, 1],
        },
        bounds: ValueBounds::None,
        nullable: false,
    };
    let bitmap16 = ValueConstraint {
        value_type: ValueType::Bitmap {
            bits: 16,
            valid_mask: 0x03,
        },
        bounds: ValueBounds::None,
        nullable: false,
    };

    // Enum8: U16(1) rejected under Strict/Widening, accepted under Fit; U16(300) overflows u8 -> ConstraintError.
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U16(1), &enum8, 0, None, IntegerMode::Strict),
        Err((None, MatterStatus::InvalidDataType))
    );
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U16(1), &enum8, 0, None, IntegerMode::Widening),
        Err((None, MatterStatus::InvalidDataType))
    );
    validate_value_with_ctx(&AttributeValue::U16(1), &enum8, 0, None, IntegerMode::Fit).unwrap();
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U16(300), &enum8, 0, None, IntegerMode::Fit),
        Err((None, MatterStatus::ConstraintError))
    );

    // Enum16 and Bitmap16: U8 allowed under Widening, rejected under Strict; U64 allowed only under Fit.
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U8(1), &enum16, 0, None, IntegerMode::Strict),
        Err((None, MatterStatus::InvalidDataType))
    );
    validate_value_with_ctx(&AttributeValue::U8(1), &enum16, 0, None, IntegerMode::Widening).unwrap();
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U64(1), &enum16, 0, None, IntegerMode::Widening),
        Err((None, MatterStatus::InvalidDataType))
    );
    validate_value_with_ctx(&AttributeValue::U64(1), &enum16, 0, None, IntegerMode::Fit).unwrap();

    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U8(1), &bitmap16, 0, None, IntegerMode::Strict),
        Err((None, MatterStatus::InvalidDataType))
    );
    validate_value_with_ctx(&AttributeValue::U8(1), &bitmap16, 0, None, IntegerMode::Widening).unwrap();
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U64(1), &bitmap16, 0, None, IntegerMode::Widening),
        Err((None, MatterStatus::InvalidDataType))
    );
    validate_value_with_ctx(&AttributeValue::U64(1), &bitmap16, 0, None, IntegerMode::Fit).unwrap();
    assert_eq!(
        validate_value_with_ctx(&AttributeValue::U64(0x04), &bitmap16, 0, None, IntegerMode::Fit),
        Err((None, MatterStatus::ConstraintError))
    );
}

#[test]
fn device_type_list_and_tag_list_respect_integer_modes() {
    let reg = mock_registry(false);

    let mut on_off = Cluster::empty(ClusterId::new(0x0006));
    on_off.set_attribute(AttributeId::new(0x0000), AttributeValue::Bool(true));
    on_off.set_attribute(AttributeId::new(0x0004), AttributeValue::U8(1));

    // DeviceTypeList with U8 id/revision (valid under Widening/Fit, rejected under Strict).
    let mut desc_u8 = Cluster::empty(ClusterId::DESCRIPTOR);
    desc_u8.set_attribute(
        AttributeId::new(0x0000),
        AttributeValue::List(vec![AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::U16(0x0100)),
            (FieldId::new(1), AttributeValue::U8(1)),
        ])]),
    );
    let ep_u8_dt = Endpoint::empty(EndpointId::new(1))
        .with_cluster(desc_u8)
        .with_cluster(on_off.clone());
    reg.validator().endpoint(&ep_u8_dt).unwrap();
    assert_eq!(
        reg.validator()
            .with_integer_mode(IntegerMode::Strict)
            .endpoint(&ep_u8_dt),
        Err(EndpointError::descriptor_attr_status(
            0x0000,
            MatterStatus::InvalidDataType
        ))
    );

    // DeviceTypeList with U64 id/revision (rejected under Widening, accepted under Fit).
    let mut desc_u64 = Cluster::empty(ClusterId::DESCRIPTOR);
    desc_u64.set_attribute(
        AttributeId::new(0x0000),
        AttributeValue::List(vec![AttributeValue::Struct(vec![
            (FieldId::new(0), AttributeValue::U64(0x0100)),
            (FieldId::new(1), AttributeValue::U64(1)),
        ])]),
    );
    let ep_u64_dt = Endpoint::empty(EndpointId::new(1))
        .with_cluster(desc_u64)
        .with_cluster(on_off.clone());
    assert_eq!(
        reg.validator().endpoint(&ep_u64_dt),
        Err(EndpointError::descriptor_attr_status(
            0x0000,
            MatterStatus::InvalidDataType
        ))
    );
    reg.validator()
        .with_integer_mode(IntegerMode::Fit)
        .endpoint(&ep_u64_dt)
        .unwrap();

    // TagList with U8 mfg_code must still detect duplicates under Widening and Fit.
    let mut ep_u8_dup_tag = Endpoint::empty(EndpointId::new(1))
        .with_device_type_list([DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 1)])
        .with_cluster(on_off);
    let tag_u8 = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::U8(1)),
        (FieldId::new(1), AttributeValue::U8(2)),
        (FieldId::new(2), AttributeValue::U8(3)),
    ]);
    let tag_u16 = AttributeValue::Struct(vec![
        (FieldId::new(0), AttributeValue::U16(1)),
        (FieldId::new(1), AttributeValue::U8(2)),
        (FieldId::new(2), AttributeValue::U8(3)),
    ]);
    ep_u8_dup_tag
        .descriptor_mut()
        .unwrap()
        .set_attribute(AttributeId::new(0x0004), AttributeValue::List(vec![tag_u8, tag_u16]));
    assert_eq!(
        reg.validator().endpoint(&ep_u8_dup_tag),
        Err(descriptor_attr_error(0x0004))
    );
    assert_eq!(
        reg.validator()
            .with_integer_mode(IntegerMode::Fit)
            .endpoint(&ep_u8_dup_tag),
        Err(descriptor_attr_error(0x0004))
    );
}
