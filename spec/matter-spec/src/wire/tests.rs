use alloc::vec::Vec;

use super::*;
use crate::model::{
    Attribute as WireAttribute, AttributeId, AttributeValue, CommandId, EventId, EventNumber, EventPriority,
    EventTimestamp, MatterStatus, Optional,
};

#[test]
fn global_attributes_roundtrip_and_duplicate_rejection() {
    let mut globals = GlobalAttributes::<u32>::default();
    assert!(
        globals
            .apply_global_attribute(
                AttributeId::GENERATED_COMMAND_LIST,
                AttributeValue::List(alloc::vec![AttributeValue::U32(0x01)]),
            )
            .expect("generated_command_list")
    );
    assert!(
        globals
            .apply_global_attribute(
                AttributeId::ACCEPTED_COMMAND_LIST,
                AttributeValue::List(alloc::vec![AttributeValue::U32(0x00)]),
            )
            .expect("accepted_command_list")
    );
    assert!(
        globals
            .apply_global_attribute(
                AttributeId::EVENT_LIST,
                AttributeValue::List(alloc::vec![AttributeValue::U32(0x02)]),
            )
            .expect("event_list")
    );
    assert!(
        globals
            .apply_global_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x03))
            .expect("feature_map")
    );
    assert!(
        globals
            .apply_global_attribute(AttributeId::CLUSTER_REVISION, AttributeValue::U16(4))
            .expect("cluster_revision")
    );
    assert!(
        globals
            .apply_global_attribute(
                AttributeId::ATTRIBUTE_LIST,
                AttributeValue::List(alloc::vec![
                    AttributeValue::U32(0x0000),
                    AttributeValue::U32(AttributeId::FEATURE_MAP.raw()),
                ]),
            )
            .expect("attribute_list")
    );

    let mut encoded = Vec::new();
    globals.clone().into_attributes(&mut encoded);
    assert_eq!(encoded.len(), 6);

    let mut decoded = GlobalAttributes::<u32>::default();
    for (id, val) in encoded {
        assert!(decoded.apply_global_attribute(id, val).expect("decode"));
    }
    assert_eq!(decoded, globals);

    // Duplicate global attributes must fail with ConstraintError across all 6 globals
    for (id, val) in [
        (AttributeId::GENERATED_COMMAND_LIST, AttributeValue::List(Vec::new())),
        (AttributeId::ACCEPTED_COMMAND_LIST, AttributeValue::List(Vec::new())),
        (AttributeId::EVENT_LIST, AttributeValue::List(Vec::new())),
        (AttributeId::ATTRIBUTE_LIST, AttributeValue::List(Vec::new())),
        (AttributeId::FEATURE_MAP, AttributeValue::U32(0)),
        (AttributeId::CLUSTER_REVISION, AttributeValue::U16(5)),
    ] {
        assert_eq!(
            decoded.apply_global_attribute(id, val),
            Err(MatterStatus::ConstraintError)
        );
    }
}

#[cfg(feature = "schemars")]
#[test]
fn envelopes_schemars_and_deny_unknown_fields_compatible() {
    let g_schema = schemars::schema_for!(GlobalAttributes<u32>);
    let c_schema = schemars::schema_for!(WireCluster<(), u32>);
    let e_schema = schemars::schema_for!(WireEvent<()>);

    for schema in [g_schema, c_schema, e_schema] {
        let json = serde_json::to_value(&schema).expect("serialize envelope schema");
        let obj = json.as_object().expect("schema object");
        if cfg!(feature = "deny-unknown-fields") {
            assert_eq!(
                obj.get("additionalProperties"),
                Some(&serde_json::Value::Bool(false)),
                "deny-unknown-fields must emit additionalProperties: false: {json}"
            );
        } else {
            assert_ne!(obj.get("additionalProperties"), Some(&serde_json::Value::Bool(false)));
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn envelopes_serde_deny_unknown_fields_respects_feature() {
    let g_json = r#"{"extra_unknown": 1}"#;
    let g_res: Result<GlobalAttributes<u32>, _> = serde_json::from_str(g_json);
    let c_json = r#"{"domain": null, "extra_unknown": 1}"#;
    let c_res: Result<WireCluster<(), u32>, _> = serde_json::from_str(c_json);
    let e_json = r#"{"priority": "Info", "event": null, "extra_unknown": 1}"#;
    let e_res: Result<WireEvent<()>, _> = serde_json::from_str(e_json);

    #[cfg(feature = "deny-unknown-fields")]
    {
        g_res.expect_err("GlobalAttributes must reject unknown fields");
        c_res.expect_err("WireCluster must reject unknown fields");
        e_res.expect_err("WireEvent must reject unknown fields");
    }
    #[cfg(not(feature = "deny-unknown-fields"))]
    {
        g_res.expect("GlobalAttributes should ignore unknown fields");
        c_res.expect("WireCluster should ignore unknown fields");
        e_res.expect("WireEvent should ignore unknown fields");
    }
}

#[test]
fn cluster_envelope_routes_mei_and_rejects_unknown_standard_and_reserved() {
    let mut env = WireCluster::<(), u32>::new(());

    // Valid MEI: vendor 0xFFF1, sub-ID 0x0001
    let valid_mei = AttributeId::new(0xFFF1_0001);
    env.route_unknown_attribute(valid_mei, AttributeValue::U8(42))
        .expect("valid MEI should route to extra_attributes");
    assert_eq!(env.extra_attributes.len(), 1);
    assert_eq!(env.extra_attributes[0].id, valid_mei);

    // Unknown standard attribute 0x0000..=0x4FFF -> UnsupportedAttribute
    assert_eq!(
        env.route_unknown_attribute(AttributeId::new(0x0005), AttributeValue::Bool(true)),
        Err(MatterStatus::UnsupportedAttribute)
    );

    // Reserved vendor 0xFFFF -> UnsupportedAttribute
    assert_eq!(
        env.route_unknown_attribute(AttributeId::new(0xFFFF_0001), AttributeValue::U8(1)),
        Err(MatterStatus::UnsupportedAttribute)
    );

    // Reserved vendor 0xFFF5 -> UnsupportedAttribute
    assert_eq!(
        env.route_unknown_attribute(AttributeId::new(0xFFF5_0001), AttributeValue::U8(1)),
        Err(MatterStatus::UnsupportedAttribute)
    );

    // Unallocated standard ranges -> UnsupportedAttribute
    for raw in [0x5000_u32, 0xEFFF, 0xF000, 0xFFF7, 0xFFFE] {
        assert_eq!(
            env.route_unknown_attribute(AttributeId::new(raw), AttributeValue::U8(1)),
            Err(MatterStatus::UnsupportedAttribute)
        );
    }
}

#[test]
fn event_envelope_verifies_priority_and_preserves_metadata() {
    assert_eq!(
        WireEvent::try_new(EventPriority::Debug, EventPriority::Critical, ()),
        Err(MatterStatus::InvalidDataType)
    );

    let env = WireEvent::try_new(EventPriority::Info, EventPriority::Info, ())
        .expect("matching priority")
        .with_event_number(EventNumber::new(7))
        .with_timestamp(EventTimestamp::EpochMs(1_000_000));

    assert_eq!(env.priority, EventPriority::Info);
    assert_eq!(env.event_number, Optional::Present(EventNumber::new(7)));
    assert_eq!(env.timestamp, Optional::Present(EventTimestamp::EpochMs(1_000_000)));
}

#[test]
fn fluent_builders_and_inner_accessors() {
    let globals = GlobalAttributes::<u32>::empty()
        .with_generated_command_list(alloc::vec![CommandId::new(1)])
        .with_accepted_command_list(alloc::vec![CommandId::new(2)])
        .with_event_list(alloc::vec![EventId::new(3)])
        .with_attribute_list(alloc::vec![AttributeId::new(4)])
        .with_feature_map(0x05)
        .with_cluster_revision(6);

    let mut cluster = WireCluster::new(10_u8)
        .with_globals(globals.clone())
        .with_feature_map(0x0A)
        .with_extra_attribute(WireAttribute::new(AttributeId::new(0xFFF1_0001), AttributeValue::U8(9)));
    assert_eq!(*cluster.inner(), 10);
    *cluster.inner_mut() = 20;
    assert_eq!(*cluster.inner(), 20);
    assert_eq!(cluster.globals.feature_map, Optional::Present(0x0A));
    assert_eq!(cluster.globals.cluster_revision, Optional::Present(6));
    assert_eq!(cluster.extra_attributes.len(), 1);
    assert_eq!(cluster.into_inner(), 20);

    let mut ev = WireEvent::new(EventPriority::Info, 30_u8);
    assert_eq!(*ev.inner(), 30);
    *ev.inner_mut() = 40;
    assert_eq!(ev.into_inner(), 40);
}
