use super::*;

#[test]
fn empty_registry_renders_deterministically() {
    let globals = GlobalDefs::default();
    let empty_e = BTreeMap::new();
    let empty_b = BTreeMap::new();
    let empty_s = BTreeMap::new();
    let a = render_registry_tables("1.6", &[], &[], &globals, &empty_e, &empty_b, &empty_s).expect("render a");
    let b = render_registry_tables("1.6", &[], &[], &globals, &empty_e, &empty_b, &empty_s).expect("render b");
    assert_eq!(a, b);
    assert!(a.contains("static CLUSTERS: &[ClusterDescriptor]"));
    assert!(a.contains("static DEVICE_TYPES: &[DeviceTypeDescriptor]"));
    assert!(!a.contains("pub static CLUSTERS"));
    assert!(!a.contains("pub static DEVICE_TYPES"));
    assert!(a.contains("pub static SPEC_REGISTRY: SpecRegistry"));
    assert!(a.contains("pub struct Validator;"));
    assert!(a.contains("pub const fn full() -> RegistryValidator<'static>"));
    assert!(a.contains("pub const fn partial() -> RegistryValidator<'static>"));
    assert!(!a.contains("pub fn validate_"));
    assert!(a.contains("expects_event_list: false"));

    use crate::model::AttributeDef;

    let mut globals_ev = GlobalDefs::default();
    globals_ev.attributes.push(AttributeDef {
        name: "EventList".to_owned(),
        code: 0xFFFA,
        ..Default::default()
    });
    let c = render_registry_tables("1.2", &[], &[], &globals_ev, &empty_e, &empty_b, &empty_s).expect("render c");
    assert!(c.contains("expects_event_list: true"));
}

#[test]
fn array_without_entry_type_uses_type_str_and_required_commands_dedup_client_only() {
    use crate::model::{CommandDef, FieldDef, ParsedDeviceTypeInclude};

    let globals = GlobalDefs::default();
    let empty_e = BTreeMap::new();
    let empty_b = BTreeMap::new();
    let empty_s = BTreeMap::new();

    let cluster = ClusterDef {
        name: "Groups".to_owned(),
        code: 0x0004,
        commands: vec![
            CommandDef {
                name: "AddGroup".to_owned(),
                code: 0x00,
                direction: CommandDirection::ClientToServer,
                args: vec![FieldDef {
                    field_id: 0,
                    name: "GroupList".to_owned(),
                    type_str: "group_id".to_owned(),
                    entry_type: None,
                    is_array: true,
                    ..Default::default()
                }],
                ..Default::default()
            },
            CommandDef {
                name: "AddGroupResponse".to_owned(),
                code: 0x00,
                direction: CommandDirection::ServerToClient,
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let dt = ParsedDeviceType {
        name: "TestLight".to_owned(),
        device_id: 0x0100,
        includes: vec![ParsedDeviceTypeInclude {
            cluster: "Groups".to_owned(),
            is_server: true,
            required_attributes: vec![],
            required_commands: vec![
                "AddGroup".to_owned(),
                "AddGroupResponse".to_owned(),
                "AddGroup".to_owned(),
            ],
            required_events: vec![],
        }],
        ..Default::default()
    };

    let rendered = render_registry_tables("1.6", &[&cluster], &[&dt], &globals, &empty_e, &empty_b, &empty_s)
        .expect("render registry");
    assert!(
        rendered.contains("item_constraint: &ValueConstraint { value_type: ValueType::U16"),
        "GroupList array without entryType must resolve group_id -> ValueType::U16:\n{rendered}"
    );
    assert!(
        rendered.contains("required_commands: &[CommandId::new(0x00)]"),
        "required_commands must filter ServerToClient and deduplicate CommandId:\n{rendered}"
    );
}

#[test]
fn attribute_descriptor_emits_pascal_name_and_snake_field_name_and_event_priority() {
    use crate::model::{AttributeDef, EventDef};

    let globals = GlobalDefs::default();
    let empty_e = BTreeMap::new();
    let empty_b = BTreeMap::new();
    let empty_s = BTreeMap::new();

    let cluster = ClusterDef {
        name: "Identify".to_owned(),
        code: 0x0003,
        attributes: vec![AttributeDef {
            name: "IdentifyTime".to_owned(),
            code: 0x0000,
            type_str: "int16u".to_owned(),
            ..Default::default()
        }],
        events: vec![EventDef {
            name: "StateChanged".to_owned(),
            code: 0x00,
            priority: "critical".to_owned(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let rendered = render_registry_tables("1.6", &[&cluster], &[], &globals, &empty_e, &empty_b, &empty_s)
        .expect("render registry");
    assert!(rendered.contains("name: \"IdentifyTime\""), "{rendered}");
    assert!(rendered.contains("field_name: \"identify_time\""), "{rendered}");
    assert!(rendered.contains("priority: EventPriority::Critical"), "{rendered}");
}

#[test]
fn registry_renders_privileges_qualities_fabric_scoping_response_id_and_revision() {
    use crate::model::parse_csa_clusters;

    let xml = r#"
<configurator>
  <struct name="FabricEntryStruct" isFabricScoped="true">
    <cluster code="0x00F0"/>
    <item fieldId="0" name="Label" type="char_string"/>
  </struct>
  <cluster>
    <name>SampleMeta</name>
    <code>0x00F0</code>
    <globalAttribute side="either" code="0xFFFD" value="5"/>
    <attribute side="server" code="0x0000" type="array" entryType="FabricEntryStruct" writable="true" reportable="true" mustUseAtomicWrite="true">
      FabricEntries
      <access op="read" privilege="administer"/>
      <access op="write" privilege="manage"/>
      <quality changeOmitted="true"/>
    </attribute>
    <attribute side="server" code="0x0001" type="int8u" readable="false" writable="true">
      WriteOnlySecret
    </attribute>
    <command source="client" code="0x00" name="DoWork" response="DoWorkResponse" mustUseTimedInvoke="true">
      <access op="invoke" privilege="administer"/>
      <quality largeMessage="true"/>
    </command>
    <command source="server" code="0x01" name="DoWorkResponse"/>
    <event side="server" code="0x00" name="AdminAudit" priority="info" isFabricSensitive="true">
      <access op="read" privilege="administer"/>
    </event>
  </cluster>
</configurator>
"#;
    let clusters = parse_csa_clusters(xml).expect("parse xml");
    let cluster_refs: Vec<&ClusterDef> = clusters.iter().collect();
    let globals = GlobalDefs::default();
    let empty_e = BTreeMap::new();
    let empty_b = BTreeMap::new();
    let empty_s = BTreeMap::new();

    let rendered = render_registry_tables("1.6", &cluster_refs, &[], &globals, &empty_e, &empty_b, &empty_s)
        .expect("render registry");

    // Cluster revision
    assert!(rendered.contains("revision: 5"), "{rendered}");

    // FabricEntries block
    let fabric_start = rendered
        .find("name: \"FabricEntries\"")
        .expect("FabricEntries descriptor");
    let write_only_start = rendered
        .find("name: \"WriteOnlySecret\"")
        .expect("WriteOnlySecret descriptor");
    let fabric_block = &rendered[fabric_start..write_only_start];
    assert!(fabric_block.contains("access: AccessFlags(7)"), "{fabric_block}");
    assert!(fabric_block.contains("quality: QualityFlags(13)"), "{fabric_block}");
    assert!(
        fabric_block.contains("read_privilege: Some(AccessPrivilege::Administer)"),
        "{fabric_block}"
    );
    assert!(
        fabric_block.contains("write_privilege: Some(AccessPrivilege::Manage)"),
        "{fabric_block}"
    );

    // WriteOnlySecret block
    let do_work_start = rendered.find("name: \"DoWork\"").expect("DoWork descriptor");
    let write_only_block = &rendered[write_only_start..do_work_start];
    assert!(
        write_only_block.contains("access: AccessFlags(2)"),
        "{write_only_block}"
    );
    assert!(write_only_block.contains("read_privilege: None"), "{write_only_block}");
    assert!(
        write_only_block.contains("write_privilege: Some(AccessPrivilege::Operate)"),
        "{write_only_block}"
    );

    // DoWork block
    let do_work_resp_start = rendered
        .find("name: \"DoWorkResponse\"")
        .expect("DoWorkResponse descriptor");
    let do_work_block = &rendered[do_work_start..do_work_resp_start];
    assert!(do_work_block.contains("access: AccessFlags(80)"), "{do_work_block}");
    assert!(do_work_block.contains("quality: QualityFlags(2)"), "{do_work_block}");
    assert!(
        do_work_block.contains("invoke_privilege: Some(AccessPrivilege::Administer)"),
        "{do_work_block}"
    );
    assert!(
        do_work_block.contains("response_id: Some(CommandId::new(0x01))"),
        "{do_work_block}"
    );

    // DoWorkResponse block
    let admin_audit_start = rendered.find("name: \"AdminAudit\"").expect("AdminAudit descriptor");
    let resp_block = &rendered[do_work_resp_start..admin_audit_start];
    assert!(resp_block.contains("invoke_privilege: None"), "{resp_block}");
    assert!(resp_block.contains("response_id: None"), "{resp_block}");

    // AdminAudit event block
    let event_block = &rendered[admin_audit_start..];
    assert!(event_block.contains("access: AccessFlags(9)"), "{event_block}");
    assert!(
        event_block.contains("read_privilege: AccessPrivilege::Administer"),
        "{event_block}"
    );
}
