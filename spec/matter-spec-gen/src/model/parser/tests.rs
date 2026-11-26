use super::*;
use crate::model::parser::cluster::parse_csa_xml;

// Unit tests for model::parser

#[test]
fn parse_sample_onoff_xml() {
    let xml = r#"
        <configurator>
          <enum name="StartUpOnOffEnum" type="enum8">
            <cluster code="0x0006"/>
            <item name="Off" value="0x00"/>
            <item name="On" value="0x01"/>
          </enum>
          <cluster>
            <name>On/Off</name>
            <domain>General</domain>
            <code>0x0006</code>
            <attribute side="server" code="0x0000" define="ON_OFF" type="boolean">OnOff</attribute>
            <command source="client" code="0x00" name="Off">
              <description>Turn off</description>
            </command>
          </cluster>
        </configurator>
        "#;
    let clusters = parse_csa_clusters(xml).expect("parse cluster");
    assert_eq!(clusters.len(), 1);
    let cluster = &clusters[0];
    assert_eq!(cluster.name, "OnOff");
    assert_eq!(cluster.code, 0x0006);
    assert_eq!(cluster.domain, "General");
    assert_eq!(cluster.enums.len(), 1);
    assert_eq!(cluster.attributes.len(), 1);
    assert_eq!(cluster.attributes[0].name, "OnOff");
    assert_eq!(cluster.commands.len(), 1);
    assert_eq!(cluster.commands[0].name, "Off");
}

#[test]
fn heck_casing_acronym_splitting() {
    assert_eq!(to_snake_case("HEPAFilterMonitoring"), "hepa_filter_monitoring");
    assert_eq!(
        to_snake_case("OTASoftwareUpdateProvider"),
        "ota_software_update_provider"
    );
    assert_eq!(to_snake_case("RVCRunMode"), "rvc_run_mode");
    assert_eq!(to_snake_case("SmokeCOAlarm"), "smoke_co_alarm");
    assert_eq!(to_snake_case("EnergyEVSEMode"), "energy_evse_mode");
    assert_eq!(to_snake_case("CameraAVStreamManagement"), "camera_av_stream_management");
    assert_eq!(to_shouty_snake_case("DelayedAllOff"), "DELAYED_ALL_OFF");
}

#[test]
fn sample_concentration_measurement_parsing() {
    let xml = r#"
        <configurator>
          <cluster>
            <name>CarbonDioxideConcentrationMeasurement</name>
            <domain name="Measurement &amp; Sensing"/>
            <code>0x040D</code>
            <attribute side="server" code="0x0000" define="MEASURED_VALUE" type="single">MeasuredValue</attribute>
          </cluster>
        </configurator>
        "#;
    let clusters = parse_csa_clusters(xml).expect("parse clusters");
    assert_eq!(clusters.len(), 1);
    let cluster = &clusters[0];
    assert_eq!(cluster.name, "CarbonDioxideConcentrationMeasurement");
    assert_eq!(cluster.code, 0x040D);
    assert_eq!(cluster.domain, "Measurement & Sensing");
    assert_eq!(cluster.attributes.len(), 1);
}

#[test]
fn signed_hex_bounds_parse() {
    assert_eq!(parse_bound_for_type("0x80", "i8"), Some(ParsedBound::I8(-128)));
    assert_eq!(parse_bound_for_type("0x8000", "i16"), Some(ParsedBound::I16(i16::MIN)));
    assert_eq!(parse_bound_for_type("-0x80", "i16"), Some(ParsedBound::I16(-128)));
    assert_eq!(parse_bound_for_type("-0x80", "i8"), Some(ParsedBound::I8(-128)));
    assert_eq!(parse_bound_for_type("-0x81", "i8"), None);
    assert_eq!(parse_bound_for_type("-0x1", "u8"), None);
}

#[test]
fn punctuation_only_identifier_is_unknown() {
    assert_eq!(sanitize_identifier("---"), "Unknown");
}

#[test]
fn duplicate_enum_name_or_value_fails_fast() {
    let dup_name_xml = r#"
        <configurator>
          <enum name="DupNameEnum" type="enum8">
            <item name="Same" value="0x00"/>
            <item name="Same" value="0x01"/>
          </enum>
        </configurator>
    "#;
    let err_name = parse_csa_xml(dup_name_xml).expect_err("duplicate variant name must fail");
    assert!(err_name.to_string().contains("Duplicate variant name"), "{err_name}");

    let dup_val_xml = r#"
        <configurator>
          <enum name="DupValEnum" type="enum8">
            <item name="First" value="0x00"/>
            <item name="Second" value="0x00"/>
          </enum>
        </configurator>
    "#;
    let err_val = parse_csa_xml(dup_val_xml).expect_err("duplicate discriminant value must fail");
    assert!(
        err_val.to_string().contains("Duplicate discriminant value"),
        "{err_val}"
    );
}

#[test]
fn out_of_order_and_colliding_field_ids_stay_unique() {
    let xml = r#"
        <configurator>
          <struct name="TestStruct">
            <item name="First" fieldId="1" type="int8u"/>
            <item name="Second" fieldId="0" type="int8u"/>
            <item name="Third" fieldId="1" type="int8u"/>
          </struct>
          <cluster>
            <name>Test</name>
            <domain>General</domain>
            <code>0x1234</code>
            <event code="0x00" name="TestEvent" priority="info">
              <field id="1" name="A" type="int8u"/>
              <field id="0" name="B" type="int8u"/>
              <field id="1" name="C" type="int8u"/>
            </event>
          </cluster>
        </configurator>
    "#;
    let parsed = parse_csa_xml(xml).expect("colliding field IDs should resolve uniquely");
    let st = &parsed.standalone_structs[0].0;
    let st_ids: Vec<u32> = st.fields.iter().map(|f| f.field_id).collect();
    assert_eq!(st_ids, vec![1, 0, 2]);

    let ev = &parsed.clusters[0].events[0];
    let ev_ids: Vec<u32> = ev.fields.iter().map(|f| f.field_id).collect();
    assert_eq!(ev_ids, vec![1, 0, 2]);
}

#[test]
fn conformance_rpn_and_access_flags_parse() {
    let xml = r#"<configurator>
        <cluster>
            <name>TestCluster</name>
            <code>0x0006</code>
            <attribute side="server" code="0x0000" define="DEVICE_LIST" type="array" entryType="uint16u" minLength="1" length="10" writable="true">
                <description>List</description>
                <access op="read" privilege="view" />
                <access op="write" privilege="manage" modifier="timed" />
                <mandatoryConform>
                    <feature name="LT" />
                </mandatoryConform>
            </attribute>
            <command source="client" code="0x00" name="Off" optional="false" />
            <command source="server" code="0x01" name="OffResponse" optional="false" />
        </cluster>
    </configurator>"#;
    use quick_xml::de::from_str;
    let doc: ConfiguratorXml = from_str(xml).expect("valid xml");
    let parsed = parse_csa_configurator(doc).expect("valid configurator");
    let cluster = &parsed.clusters[0];
    let attr = &cluster.attributes[0];
    assert_eq!(attr.define.as_deref(), Some("DEVICE_LIST"));
    assert_eq!(attr.min_length, Some(1));
    assert_eq!(attr.length, Some(10));
    assert_eq!(
        attr.list_length,
        Some(ListLength {
            min: Some(1),
            max: Some(10),
            item_constraint: Some("uint16u".to_owned()),
        })
    );
    assert!(attr.access.read);
    assert!(attr.access.write);
    assert!(attr.access.timed);
    assert_eq!(attr.access.read_privilege, Some(AccessPrivilege::View));
    assert_eq!(attr.access.write_privilege, Some(AccessPrivilege::Manage));
    assert_eq!(
        attr.conformance,
        vec![ConformanceOp::Feature("LT".to_owned()), ConformanceOp::Mandatory,]
    );
    assert_eq!(cluster.commands[0].direction, CommandDirection::ClientToServer);
    assert_eq!(cluster.commands[1].direction, CommandDirection::ServerToClient);
    assert!(
        !cluster.commands[0].access.read,
        "command without <access op=\"read\"/> must not force read=true"
    );
    assert!(
        !cluster.commands[1].access.read,
        "server response command must not force read=true"
    );
}

#[test]
fn device_type_include_resolution_and_legacy_ignores() {
    let candidates = vec![
        (Some("DEVICE_LIST"), "DeviceList"),
        (None, "EffectiveOperationMode"),
        (None, "DeviceTypeList"),
    ];
    assert_eq!(
        resolve_include_target("DEVICE_LIST", &candidates, to_shouty_snake_case),
        Some("DeviceList")
    );
    assert_eq!(
        resolve_include_target("EffectiveOperationMode", &candidates, to_shouty_snake_case),
        Some("EffectiveOperationMode")
    );
    assert_eq!(
        resolve_include_target("DEVICE_TYPE_LIST", &candidates, to_shouty_snake_case),
        Some("DeviceTypeList")
    );
    assert!(is_legacy_device_include_ignored("1.0", "Scenes"));
    assert!(is_legacy_device_include_ignored(
        "1.6",
        "COLOR_CONTROL_COLOR_TEMP_PHYSICAL_MIN"
    ));
    assert!(!is_legacy_device_include_ignored(
        "1.5",
        "COLOR_CONTROL_COLOR_TEMP_PHYSICAL_MIN"
    ));
    assert!(!is_legacy_device_include_ignored("1.6", "UnknownAttr"));
}
