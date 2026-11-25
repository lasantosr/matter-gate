//! Shared test helpers and behavioral test macro generator for multi-version Matter specifications.

#[macro_export]
macro_rules! test_version_behavior {
    // 4-argument dispatch: automatically maps to standard switch type
    ($mod_name:ident, $ver:path, false, $has_closures:tt) => {
        $crate::test_version_behavior!($mod_name, $ver, OnOffLightSwitch, false, $has_closures);
    };
    ($mod_name:ident, $ver:path, true, $has_closures:tt) => {
        $crate::test_version_behavior!($mod_name, $ver, OnOffLightSwitch, true, $has_closures);
    };

    // 6-argument core implementation
    ($mod_name:ident, $ver:path, $switch_ty:ident, $has_appliances:tt, $has_closures:tt, $has_temperature:tt) => {
        #[cfg(test)]
        mod $mod_name {
            use std::collections::HashSet;

            use $ver as ver;

            use matter_spec::{
                error::EndpointError,
                model::{
                    AttributeId, AttributeValue, Cluster, DeviceTypeEntry, Endpoint, EndpointId,
                    MatterStatus, Nullable, Optional,
                },
            };
            use matter_spec::types::Percent100ths;

            #[test]
            fn semantic_booleans_and_wire_roundtrip() {
                use ver::clusters::on_off::{OnOff, OnOffCluster, attributes::OnOff as OnOffAttr};

                // From<bool> and Into<bool>
                assert_eq!(OnOff::from(true), OnOff::On);
                assert_eq!(OnOff::from(false), OnOff::Off);
                assert!(bool::from(OnOff::On));
                assert!(!bool::from(OnOff::Off));

                // AttributeValue conversions
                let val_on: AttributeValue = OnOff::On.into();
                assert_eq!(val_on, AttributeValue::Bool(true));
                let val_off: AttributeValue = OnOff::Off.into();
                assert_eq!(val_off, AttributeValue::Bool(false));

                let parsed_on = OnOff::try_from(val_on).expect("parse OnOff::On");
                assert_eq!(parsed_on, OnOff::On);
                let parsed_off = OnOff::try_from(val_off).expect("parse OnOff::Off");
                assert_eq!(parsed_off, OnOff::Off);

                // Cluster ON / OFF constants
                assert_eq!(OnOffCluster::ON.on_off, OnOff::On);
                assert_eq!(OnOffCluster::OFF.on_off, OnOff::Off);

                // TypedAttribute wire roundtrip
                let on_attr = OnOffAttr::new(true);
                let wire_val = on_attr.into_value();
                assert_eq!(wire_val, AttributeValue::Bool(true));
                let parsed_attr = OnOffAttr::try_from_value(wire_val).expect("parse OnOff attribute");
                assert_eq!(parsed_attr.raw(), OnOff::On);

                assert_eq!(OnOffAttr::ON, OnOffAttr::new(true));
                assert_eq!(OnOffAttr::OFF, OnOffAttr::new(false));
                assert_eq!(OnOffAttr::ON.raw(), OnOff::On);
                assert_eq!(OnOffAttr::OFF.raw(), OnOff::Off);
            }

            #[test]
            fn cluster_attributes_roundtrip_and_dynamic_enum() {
                use ver::clusters::on_off::{Attribute, OnOff, OnOffCluster};

                let cluster = OnOffCluster::ON.with_on_time(120_u16);
                let attrs = cluster.to_attributes();
                assert_eq!(attrs.len(), 2);

                let decoded = OnOffCluster::try_from_attributes(attrs).expect("decode on_off cluster");
                assert_eq!(decoded.on_off, OnOff::On);
                assert_eq!(decoded.on_time, Optional::Present(120));
                assert_eq!(decoded.off_wait_time, Optional::Omitted);

                // Dynamic Attribute routing
                let attr = Attribute::OnOff(OnOff::On.into());
                let (id, val) = attr.into_pair();
                assert_eq!(id, OnOffCluster::ATTR_ON_OFF);
                assert_eq!(val, AttributeValue::Bool(true));

                let parsed = Attribute::try_from_pair(id, val.clone()).expect("parse dynamic pair");
                assert_eq!(parsed.id(), OnOffCluster::ATTR_ON_OFF);

                let unknown_id = AttributeId::new(0xDEAD);
                assert_eq!(
                    Attribute::try_from_pair(unknown_id, val),
                    Err(MatterStatus::UnsupportedAttribute)
                );
            }

            #[test]
            fn bounds_and_enum_discriminant_constraint_errors() {
                use ver::clusters::thermostat::attributes::{ControlSequenceOfOperation, OccupiedHeatingSetpoint};

                // Unknown enum discriminant -> ConstraintError
                let invalid_enum = AttributeValue::U8(99);
                assert_eq!(
                    ControlSequenceOfOperation::try_from_value(invalid_enum),
                    Err(MatterStatus::ConstraintError)
                );

                // Min bound violation -> ConstraintError
                let out_of_bounds = AttributeValue::I16(-30000);
                assert_eq!(
                    OccupiedHeatingSetpoint::try_from_value(out_of_bounds),
                    Err(MatterStatus::ConstraintError)
                );

                #[cfg(cluster_unit_testing)]
                {
                    use ver::clusters::unit_testing::TestListStructOctet;

                    // Byte length violation -> ConstraintError
                    let invalid_bytes = matter_spec::Bytes::from(vec![0_u8; 33]);
                    let val_invalid = AttributeValue::Struct(vec![
                        (TestListStructOctet::FIELD_MEMBER1, AttributeValue::U64(1)),
                        (TestListStructOctet::FIELD_MEMBER2, AttributeValue::Bytes(invalid_bytes)),
                    ]);
                    assert_eq!(
                        TestListStructOctet::try_from(val_invalid),
                        Err(MatterStatus::ConstraintError)
                    );
                }
            }

            #[test]
            fn type_safety_and_variant_rejection() {
                use ver::clusters::basic_information::events::StartUp;
                use ver::clusters::on_off::attributes::OnOff;
                use ver::clusters::thermostat::commands::SetpointRaiseLower;

                // Non-nullable attribute rejects Null
                assert_eq!(
                    OnOff::try_from_value(AttributeValue::Null),
                    Err(MatterStatus::InvalidDataType)
                );

                // Variant mismatch rejects unexpected types
                assert_eq!(
                    OnOff::try_from_value(AttributeValue::U32(1)),
                    Err(MatterStatus::InvalidDataType)
                );
                assert_eq!(
                    OnOff::try_from_value(AttributeValue::String("on".into())),
                    Err(MatterStatus::InvalidDataType)
                );

                // Non-struct decode failures on production command & event
                assert_eq!(
                    SetpointRaiseLower::try_from_value(AttributeValue::U32(42)),
                    Err(MatterStatus::InvalidDataType)
                );
                assert_eq!(
                    StartUp::try_from_value(AttributeValue::String("not struct".into())),
                    Err(MatterStatus::InvalidDataType)
                );

                #[cfg(cluster_unit_testing)]
                {
                    use ver::clusters::unit_testing::TestListStructOctet;

                    assert_eq!(
                        TestListStructOctet::try_from(AttributeValue::Bool(true)),
                        Err(MatterStatus::InvalidDataType)
                    );
                }
            }

            #[test]
            fn nullable_and_optional_transitions() {
                use ver::clusters::thermostat::attributes::LocalTemperature;

                // Null transitions
                let null_temp = LocalTemperature::null();
                assert_eq!(null_temp.raw(), Nullable::Null);
                assert_eq!(null_temp.into_value(), AttributeValue::Null);
                assert!(null_temp.is_null());

                let parsed_null = LocalTemperature::try_from_value(AttributeValue::Null).expect("parse null temp");
                assert_eq!(parsed_null.raw(), Nullable::Null);

                // Value transitions
                let val = AttributeValue::I16(2150);
                let parsed_val = LocalTemperature::try_from_value(val).expect("parse 21.5C");
                assert_eq!(parsed_val.raw(), Nullable::NonNull(2150));

                // Optional chaining & wire roundtrip
                #[cfg(cluster_unit_testing)]
                {
                    use ver::clusters::unit_testing::NullablesAndOptionalsStruct;

                    let st = NullablesAndOptionalsStruct::new(42_u16, Nullable::Null, Nullable::Null, Nullable::Null)
                        .with_optional_int(100_u16)
                        .with_nullable_optional_int(200_u16)
                        .with_nullable_optional_string(Nullable::Null);

                    let wire_st = AttributeValue::from(st.clone());
                    let parsed_st = NullablesAndOptionalsStruct::try_from(wire_st).expect("parse struct");
                    assert_eq!(parsed_st, st);
                }
            }

            #[test]
            fn percent100ths_extension_on_cluster_attribute() {
                use ver::clusters::relative_humidity_measurement::attributes::MeasuredValue as HumidityMeasuredValue;

                // Percent100ths semantic type
                let hum = HumidityMeasuredValue::from(5000_u16);
                assert_eq!(hum.raw(), Nullable::NonNull(5000));
                assert_eq!(hum.map(|v| Percent100ths::from_raw(v).to_percent()), Nullable::NonNull(50.0));

                let null_hum = HumidityMeasuredValue::null();
                assert_eq!(null_hum.map(|v| Percent100ths::from_raw(v).to_percent()), Nullable::Null);
            }

            #[test]
            fn typed_commands_and_events_roundtrip() {
                use ver::clusters::boolean_state::events::StateChange;
                use ver::clusters::identify::commands::Identify;
                use ver::clusters::on_off::commands::Off;

                // Unit command without payload
                assert_eq!(Off.into_fields(), []);
                let _parsed_off_fields = Off::try_from_fields(Vec::new()).expect("parse Off command fields");
                let off_val = Off.into_value();
                assert_eq!(off_val, AttributeValue::Struct(Vec::new()));
                let _parsed_off = Off::try_from_value(off_val).expect("parse Off command");

                // Command with structured arguments
                let cmd = Identify::new(10_u16);
                let cmd_fields = cmd.clone().into_fields();
                assert_eq!(Identify::try_from_fields(cmd_fields).expect("parse Identify fields"), cmd);
                let cmd_val = cmd.into_value();
                let parsed_cmd = Identify::try_from_value(cmd_val).expect("parse Identify");
                assert_eq!(parsed_cmd.identify_time, 10);

                // Event roundtrip
                let ev = StateChange { state_value: true };
                let ev_fields = ev.clone().into_fields();
                assert_eq!(StateChange::try_from_fields(ev_fields).expect("parse StateChange fields"), ev);
                let ev_val = ev.into_value();
                let parsed_ev = StateChange::try_from_value(ev_val).expect("parse StateChange");
                assert!(parsed_ev.state_value);
            }

            #[test]
            fn parts_list_endpoint_id_roundtrip() {
                use ver::clusters::descriptor::attributes::PartsList;

                let ep1 = EndpointId::new(1);
                let ep2 = EndpointId::new(2);
                let list = PartsList::new(vec![ep1, ep2]);
                assert_eq!(list.raw(), &vec![ep1, ep2]);
                let val = list.into_value();
                let parsed = PartsList::try_from_value(val).expect("parse PartsList");
                assert_eq!(parsed.raw(), &vec![ep1, ep2]);
            }

            #[test]
            fn attribute_deref_display_and_hash() {
                use ver::clusters::on_off::attributes::OnOff;

                let on = OnOff::from(true);
                assert_eq!(*on, ver::clusters::on_off::OnOff::On);
                assert_eq!(format!("{on}"), "On");

                let mut set = HashSet::new();
                set.insert(OnOff::ON);
                set.insert(OnOff::OFF);
                set.insert(OnOff::ON);
                assert_eq!(set.len(), 2);
                assert!(set.contains(&OnOff::ON));
                assert!(set.contains(&OnOff::OFF));

                let mut cmd_set = HashSet::new();
                cmd_set.insert(ver::clusters::on_off::Command::On);
                cmd_set.insert(ver::clusters::on_off::Command::Off);
                assert_eq!(cmd_set.len(), 2);

                let mut ep_set = HashSet::new();
                ep_set.insert(ver::endpoints::on_off_light::OnOffLight::default());
                assert_eq!(ep_set.len(), 1);
            }

            #[test]
            fn prelude_reexports() {
                use ver::prelude::*;

                let n: Nullable<u32> = Null;
                let v: Nullable<u32> = NonNull(42);
                let o: Optional<u32> = Omitted;
                let p: Optional<u32> = Present(42);

                assert!(n.is_null());
                assert_eq!(v, NonNull(42));
                assert!(o.is_omitted());
                assert_eq!(p, Present(42));
            }

            #[test]
            fn endpoint_conformance() {
                use ver::clusters::{
                    binding::BindingCluster,
                    descriptor::DescriptorCluster,
                    identify::IdentifyCluster,
                    on_off::OnOffCluster,
                };
                use ver::endpoints::$switch_ty;

                let sw = $switch_ty::new(
                    IdentifyCluster::default(),
                    BindingCluster::default(),
                );
                let endpoint = sw.clone().into_endpoint(EndpointId::new(1));
                let sw_back = $switch_ty::try_from(endpoint).expect("switch conformance");
                assert_eq!(sw, sw_back);

                // Missing mandatory cluster rejection
                let partial_endpoint = Endpoint::new(
                    EndpointId::new(1),
                    vec![Cluster::empty(IdentifyCluster::CLUSTER_ID)],
                )
                .with_device_type_list([DeviceTypeEntry::new($switch_ty::DEVICE_TYPE_ID, $switch_ty::REVISION)]);
                let err_missing = $switch_ty::try_from(partial_endpoint);
                assert!(matches!(
                    err_missing,
                    Err(EndpointError::MissingMandatoryServerCluster(c))
                        if c == DescriptorCluster::CLUSTER_ID || c == BindingCluster::CLUSTER_ID
                ));

                // Disallowed cluster rejection (OnOffCluster is disallowed as a server on OnOffLightSwitch)
                let invalid_endpoint = Endpoint::new(
                    EndpointId::new(1),
                    vec![
                        Cluster::empty(DescriptorCluster::CLUSTER_ID),
                        Cluster::empty(IdentifyCluster::CLUSTER_ID),
                        Cluster::empty(BindingCluster::CLUSTER_ID),
                        Cluster::empty(OnOffCluster::CLUSTER_ID),
                    ],
                )
                .with_device_type_list([DeviceTypeEntry::new($switch_ty::DEVICE_TYPE_ID, $switch_ty::REVISION)])
                .with_client_list([
                    IdentifyCluster::CLUSTER_ID,
                    OnOffCluster::CLUSTER_ID,
                ]);
                let err_disallowed = $switch_ty::try_from(invalid_endpoint);
                assert!(matches!(
                    err_disallowed,
                    Err(EndpointError::DisallowedCluster(c)) if c == OnOffCluster::CLUSTER_ID
                ));
            }

            $crate::test_version_behavior!(@closures $has_closures);
            $crate::test_version_behavior!(@appliances $has_appliances);
            $crate::test_version_behavior!(@temperature $has_temperature);

            #[cfg(feature = "serde")]
            #[test]
            fn serde_roundtrip() {
                use ver::clusters::binding::BindingCluster;
                use ver::clusters::identify::IdentifyCluster;
                use ver::clusters::on_off::{OnOffCluster, attributes::OnOff};
                use ver::endpoints::$switch_ty;

                let cluster = OnOffCluster::ON.with_on_time(60_u16);
                let json_c = serde_json::to_string(&cluster).expect("serialize cluster");
                let des_c: OnOffCluster = serde_json::from_str(&json_c).expect("deserialize cluster");
                assert_eq!(cluster, des_c);

                let sw = $switch_ty::new(
                    IdentifyCluster::default(),
                    BindingCluster::default(),
                );
                let json_sw = serde_json::to_string(&sw).expect("serialize switch");
                let des_sw: $switch_ty = serde_json::from_str(&json_sw).expect("deserialize switch");
                assert_eq!(sw, des_sw);

                let on = OnOff::ON;
                let json_on = serde_json::to_string(&on).expect("serialize on");
                let des_on: OnOff = serde_json::from_str(&json_on).expect("deserialize on");
                assert_eq!(on, des_on);
            }

            #[cfg(feature = "schemars")]
            #[test]
            fn schemars_generation() {
                use ver::clusters::on_off::OnOffCluster;
                use ver::endpoints::{OnOffLight, $switch_ty};

                let schema_c = schemars::schema_for!(OnOffCluster);
                let json_c = serde_json::to_string(&schema_c).expect("serialize cluster schema");
                assert!(json_c.contains("OnOffCluster"));
                let required_c = schema_c
                    .as_object()
                    .and_then(|o| o.get("required"))
                    .and_then(|v| v.as_array());
                assert!(
                    required_c.is_some_and(|req| {
                        req.iter().any(|v| v.as_str() == Some("on_off"))
                            && !req.iter().any(|v| v.as_str() == Some("global_scene_control"))
                            && !req.iter().any(|v| v.as_str() == Some("optional"))
                    }),
                    "OnOffCluster JSON schema must flatten optional fields and omit them from required"
                );

                let schema_sw = schemars::schema_for!($switch_ty);
                let json_sw = serde_json::to_string(&schema_sw).expect("serialize switch schema");
                assert!(json_sw.contains(stringify!($switch_ty)));

                let schema_lt = schemars::schema_for!(OnOffLight);
                let required_lt = schema_lt
                    .as_object()
                    .and_then(|o| o.get("required"))
                    .and_then(|v| v.as_array());
                assert!(
                    required_lt.is_some_and(|req| {
                        req.iter().any(|v| v.as_str() == Some("on_off"))
                            && !req.iter().any(|v| v.as_str() == Some("level_control"))
                            && !req.iter().any(|v| v.as_str() == Some("optional"))
                    }),
                    "OnOffLight JSON schema must flatten optional clusters and omit them from required"
                );
            }
        }
    };

    // Sub-macro branches for optional modules
    (@closures true) => {
        #[test]
        fn door_lock_feature_map_has_pin_and_usr() {
            use ver::clusters::door_lock::FeatureMap as DoorLockFeatureMap;

            let features = DoorLockFeatureMap::PIN | DoorLockFeatureMap::USR;
            assert!(features.contains(DoorLockFeatureMap::PIN));
            assert!(features.contains(DoorLockFeatureMap::USR));
        }
    };
    (@closures false) => {};

    (@appliances true) => {
        #[test]
        fn appliances_mode_cluster_attributes_roundtrip() {
            use matter_spec::extensions::ModeClusterExt as _;
            use ver::clusters::dishwasher_mode::{DishwasherModeCluster, ModeTag};

            let mode = DishwasherModeCluster::from_tags([ModeTag::Normal, ModeTag::Heavy], ModeTag::Normal).unwrap();
            let attrs = mode.to_attributes();
            let decoded = DishwasherModeCluster::try_from_attributes(attrs).expect("decode mode cluster");
            assert_eq!(decoded, mode);
        }
    };
    (@appliances false) => {};

    (@temperature true) => {
        #[test]
        fn thermostat_temperature_semantic_deref_and_display() {
            use matter_spec::types::Temperature;
            use ver::clusters::thermostat::attributes::LocalTemperature;

            let temp = LocalTemperature::from(Temperature::from_raw(2100));
            assert_eq!(*temp, Nullable::NonNull(Temperature::from_raw(2100)));
            assert_eq!(format!("{temp}"), "21.00°C");
        }
    };
    (@temperature false) => {};
}
