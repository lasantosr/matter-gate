use std::collections::BTreeSet;

use super::*;
use crate::model::{AttributeDef, FieldDef};

#[test]
fn byte_size_threshold_boxes_large_structs_and_keeps_compact_unboxed() {
    let shared = Arc::new(SharedClusterContext {
        global_defs: GlobalDefs::default(),
        all_enums: BTreeMap::new(),
        all_bitmaps: BTreeMap::new(),
        all_structs: BTreeMap::new(),
    });

    let compact = ClusterItem {
        spec_version: "1.6".to_owned(),
        exact_tag: "v1.6".to_owned(),
        category: Category::general(),
        module_name: "on_off".to_owned(),
        original_name: "OnOff".to_owned(),
        cluster_cfg: "cluster_on_off".to_owned(),
        cluster_def: ClusterDef {
            name: "OnOff".to_owned(),
            attributes: vec![
                AttributeDef {
                    name: "OnOff".to_owned(),
                    type_str: "boolean".to_owned(),
                    ..Default::default()
                },
                AttributeDef {
                    name: "OnTime".to_owned(),
                    type_str: "int16u".to_owned(),
                    is_optional: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        shared: Arc::clone(&shared),
    };
    assert!(!compact.is_large_state());
    assert!(!compact.is_large_patch());

    // Fewer than 8 attributes (4 attrs), but > 64 bytes due to Vec + String + nested struct.
    let large_op_state = ClusterItem {
        spec_version: "1.6".to_owned(),
        exact_tag: "v1.6".to_owned(),
        category: Category::general(),
        module_name: "operational_state".to_owned(),
        original_name: "OperationalState".to_owned(),
        cluster_cfg: "cluster_operational_state".to_owned(),
        cluster_def: ClusterDef {
            name: "OperationalState".to_owned(),
            structs: vec![StructDef {
                name: "ErrorStateStruct".to_owned(),
                fields: vec![
                    FieldDef {
                        name: "ErrorStateID".to_owned(),
                        type_str: "int8u".to_owned(),
                        ..Default::default()
                    },
                    FieldDef {
                        name: "ErrorStateLabel".to_owned(),
                        type_str: "char_string".to_owned(),
                        is_optional: true,
                        ..Default::default()
                    },
                    FieldDef {
                        name: "ErrorStateDetails".to_owned(),
                        type_str: "char_string".to_owned(),
                        is_optional: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            attributes: vec![
                AttributeDef {
                    name: "PhaseList".to_owned(),
                    type_str: "array".to_owned(),
                    entry_type: Some("char_string".to_owned()),
                    is_array: true,
                    is_nullable: true,
                    ..Default::default()
                },
                AttributeDef {
                    name: "OperationalStateList".to_owned(),
                    type_str: "array".to_owned(),
                    entry_type: Some("int8u".to_owned()),
                    is_array: true,
                    ..Default::default()
                },
                AttributeDef {
                    name: "OperationalError".to_owned(),
                    type_str: "ErrorStateStruct".to_owned(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        shared,
    };
    assert!(large_op_state.is_large_state(), "expected > 64B state");
    assert!(large_op_state.is_large_patch(), "expected > 64B patch");

    // DeviceTypeItem layout threshold: 1 large cluster (> 64B) makes the endpoint large even with 1 dependency.
    let mut cluster_layouts = BTreeMap::new();
    cluster_layouts.insert("OnOff".to_owned(), compact.estimated_state_layout());
    cluster_layouts.insert("OperationalState".to_owned(), large_op_state.estimated_state_layout());

    let dt_shared = Arc::new(SharedDeviceTypeContext {
        available_clusters: vec!["OnOff".to_owned(), "OperationalState".to_owned()],
        cluster_defs: vec![],
        eq_clusters: BTreeSet::new(),
        cluster_layouts,
    });

    let small_dt = DeviceTypeItem {
        spec_version: "1.6".to_owned(),
        category: Category::general(),
        module_name: "small_dev".to_owned(),
        original_name: "SmallDev".to_owned(),
        feature_name: "endpoint-small-dev".to_owned(),
        device_id: 0x0100,
        cluster_dependencies: BTreeSet::from(["cluster_on_off".to_owned()]),
        device_type: ParsedDeviceType {
            mandatory_server_clusters: vec!["OnOff".to_owned()],
            ..Default::default()
        },
        shared: Arc::clone(&dt_shared),
    };
    assert!(!small_dt.is_large());

    let large_dt = DeviceTypeItem {
        spec_version: "1.6".to_owned(),
        category: Category::general(),
        module_name: "large_dev".to_owned(),
        original_name: "LargeDev".to_owned(),
        feature_name: "endpoint-large-dev".to_owned(),
        device_id: 0x0101,
        cluster_dependencies: BTreeSet::from(["cluster_operational_state".to_owned()]),
        device_type: ParsedDeviceType {
            mandatory_server_clusters: vec!["OperationalState".to_owned()],
            ..Default::default()
        },
        shared: dt_shared,
    };
    assert!(large_dt.is_large());
}
