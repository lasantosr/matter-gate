use matter_spec_gen::{Render as _, SpecInput, XmlDocument, build};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_from_minimal_in_memory_fixtures() {
        let chip_types = include_str!("fixtures/chip-types.xml");
        let global_attributes = include_str!("fixtures/global-attributes.xml");
        let on_off = include_str!("fixtures/on-off.xml");
        let matter_devices = include_str!("fixtures/matter-devices.xml");

        let docs = vec![
            XmlDocument::new("chip-types.xml", chip_types),
            XmlDocument::new("global-attributes.xml", global_attributes),
            XmlDocument::new("on-off.xml", on_off),
            XmlDocument::new("matter-devices.xml", matter_devices),
        ];

        let inputs = vec![SpecInput::new("v1_4", "v1.4", docs)];

        let spec = build(inputs).expect("build should succeed");
        assert_eq!(spec.versions.len(), 1);

        let ver = &spec.versions[0];
        assert_eq!(ver.version, "v1_4");
        assert_eq!(ver.tag, "v1.4");

        // Clusters
        assert_eq!(ver.clusters.len(), 1);
        let cl = &ver.clusters[0];
        assert_eq!(cl.name(), "OnOff");
        assert_eq!(cl.category().as_str(), "general");
        assert_eq!(cl.module_name(), "on_off");
        assert_eq!(cl.cluster_cfg(), "cluster_on_off");
        assert_eq!(cl.exact_tag(), "v1.4");

        let cl_code = cl.render().expect("render cluster");
        assert!(cl_code.contains("OnOffCluster"));
        assert!(cl_code.contains("0x0006"));
        assert!(cl_code.contains("pub struct OnOff(super::OnOff);"));
        assert!(!cl_code.contains("pub struct OnOff(pub super::OnOff);"));
        assert!(cl_code.contains("pub struct FeatureMap(u32);"));
        assert!(!cl_code.contains("pub struct FeatureMap(pub u32);"));
        assert!(cl_code.contains("crate::define_matter_bitmap!"));
        assert!(cl_code.contains("impl crate::traits::ClusterSpec for OnOffCluster"));
        assert!(cl_code.contains("crate::impl_matter_command!(Off, super::OnOffCluster, true);"));
        assert!(cl_code.contains("crate::impl_matter_attribute!(OnOff, super::OnOff, super::OnOffCluster, copy);"));
        assert!(cl_code.contains("Off,"));
        assert!(!cl_code.contains("Off(commands::Off)"));
        assert!(cl_code.contains("impl crate::traits::ClusterSpec for OnOffCluster"));
        assert!(cl_code.contains("impl crate::traits::PartialCluster for OnOffClusterPatch"));
        assert!(cl_code.contains("impl crate::traits::CommandSpec for Off"));
        assert!(cl_code.contains("pub fn apply(&mut self, attr: Attribute)"));
        assert!(cl_code.contains("fn try_into_full(self) -> Result<Self::Full, ClusterError>"));
        assert!(cl_code.contains("crate::impl_cluster_wire_conversions!(OnOffCluster);"));
        assert!(cl_code.contains("crate::impl_matter_describe!(cluster, OnOffCluster, false);"));
        assert!(cl_code.contains("pub(crate) fn decode_attributes_inner"));
        assert!(cl_code.contains("pub(crate) fn encode_attributes_inner"));
        assert!(!cl_code.contains("pub fn validate("));
        assert!(!cl_code.contains("fn diff("));
        assert!(!cl_code.contains("super::super::dispatch::"));

        // Device types
        assert_eq!(ver.device_types.len(), 1);
        let dt_item = &ver.device_types[0];
        assert_eq!(dt_item.name(), "OnOffLight");
        assert_eq!(dt_item.category().as_str(), "lighting");
        assert_eq!(dt_item.module_name(), "on_off_light");
        assert_eq!(dt_item.feature_name(), "endpoint-on-off-light");
        assert_eq!(dt_item.device_id(), 0x0100);
        assert!(dt_item.cluster_dependencies().contains("cluster_on_off"));

        let dt_single_code = dt_item.render().expect("render single device type");
        assert!(dt_single_code.contains("pub struct OnOffLight"));
        assert!(dt_single_code.contains("impl DeviceTypeSpec for OnOffLight"));
        assert!(dt_single_code.contains("validate_device_view("));
        assert!(dt_single_code.contains("pub fn on_off(&self) -> Option<&'a Cluster>"));
        assert!(dt_single_code.contains("crate::impl_matter_describe!(device, OnOffLight, false);"));
        assert!(!dt_single_code.contains("impl From<OnOffLight> for Endpoint"));
        assert!(!dt_single_code.contains("pub fn validate("));
        assert!(!dt_single_code.contains("OnOffLightOptional"));
        assert!(!dt_single_code.contains("super::super::dispatch::"));

        let reg_code = ver.render_registry().expect("render registry");
        assert!(reg_code.contains("pub static SPEC_REGISTRY: SpecRegistry"));
        assert!(reg_code.contains("expects_event_list:"));
        assert!(reg_code.contains("field_name: \"on_off\""));

        // Globals
        let globals_code = ver.globals.render().expect("render globals");
        assert!(globals_code.contains("pub struct ClusterRevision(u16);"));
        assert!(!globals_code.contains("pub struct ClusterRevision(pub u16);"));
        assert!(globals_code.contains("pub struct FeatureMap(u32);"));
        assert!(!globals_code.contains("pub struct FeatureMap(pub u32);"));
        assert!(globals_code.contains("pub struct AttributeList(Vec<AttributeId>);"));
        assert!(!globals_code.contains("pub struct AttributeList(pub Vec<AttributeId>);"));

        // Deterministic output across repeated runs
        let docs2 = vec![
            XmlDocument::new("chip-types.xml", chip_types),
            XmlDocument::new("global-attributes.xml", global_attributes),
            XmlDocument::new("on-off.xml", on_off),
            XmlDocument::new("matter-devices.xml", matter_devices),
        ];
        let spec2 = build(vec![SpecInput::new("v1_4", "v1.4", docs2)]).expect("second build should succeed");
        let cl_code2 = spec2.versions[0].clusters[0].render().expect("render cluster 2");
        let dt_code2 = spec2.versions[0].device_types[0]
            .render()
            .expect("render device type 2");
        assert_eq!(cl_code, cl_code2);
        assert_eq!(dt_single_code, dt_code2);
    }

    #[test]
    fn render_optional_schemars_and_serde_attrs() {
        let chip_types = include_str!("fixtures/chip-types.xml");
        let global_attributes = include_str!("fixtures/global-attributes.xml");
        let cluster_xml = r#"<configurator>
  <cluster>
    <name>OnOff</name>
    <domain>General</domain>
    <code>0x0006</code>
    <define>ON_OFF_CLUSTER</define>
    <client init="false" tick="false">true</client>
    <server init="false" tick="false">true</server>
    <globalAttribute side="either" code="0xFFFD" value="6"/>
    <attribute side="server" code="0x0000" define="ON_OFF" type="boolean" writable="false" optional="false">OnOff</attribute>
    <attribute side="server" code="0x4001" define="ON_TIME" type="int16u" writable="true" optional="true">OnTime</attribute>
  </cluster>
</configurator>"#;
        let devices_xml = r#"<configurator>
  <deviceType>
    <name>MA-onofflight</name>
    <typeName>Matter On/Off Light</typeName>
    <deviceId>0x0100</deviceId>
    <clusters>
      <include cluster="OnOff" server="true" client="false" serverLocked="false" clientLocked="false">
        <features>
          <feature code="LT" name="Lighting"/>
        </features>
      </include>
    </clusters>
  </deviceType>
</configurator>"#;

        let spec = build(vec![SpecInput::new(
            "v1_4",
            "v1.4",
            vec![
                XmlDocument::new("chip-types.xml", chip_types),
                XmlDocument::new("global-attributes.xml", global_attributes),
                XmlDocument::new("on-off.xml", cluster_xml),
                XmlDocument::new("matter-devices.xml", devices_xml),
            ],
        )])
        .expect("build with optional attr");

        let cl_code = spec.versions[0].clusters[0].render().expect("render cluster");
        assert!(cl_code.contains("schemars(default)"));
        assert!(!cl_code.contains("schemars(flatten)"));
        assert!(!cl_code.contains("serde(flatten)"));
        assert!(!cl_code.contains("ClusterOptional"));

        let dt_code = spec.versions[0].device_types[0].render().expect("render device type");
        assert!(dt_code.contains("Optional::is_omitted"));
        assert!(dt_code.contains("schemars(default)"));
        assert!(!dt_code.contains("schemars(flatten)"));
        assert!(!dt_code.contains("serde(flatten)"));
    }

    #[test]
    fn shared_dedup_union_gates_and_determinism() {
        let chip_types = r#"<configurator><atomic><type id="0x20" name="int8u" size="1" /></atomic></configurator>"#;
        let cluster_xml = r#"<configurator>
            <domain name="General" />
            <cluster>
                <name>OnOff</name>
                <code>0x0006</code>
                <globalAttribute code="0xFFFD" side="either" value="6" />
                <attribute side="server" code="0x0000" define="ON_OFF" type="boolean" writable="false" optional="false">OnOff</attribute>
                <command source="client" code="0x00" name="Off" optional="false" />
            </cluster>
        </configurator>"#;

        let inputs = vec![
            SpecInput::new(
                "v1_0",
                "1.0",
                vec![
                    XmlDocument::new("chip-types.xml", chip_types),
                    XmlDocument::new("onoff.xml", cluster_xml),
                ],
            ),
            SpecInput::new(
                "v1_1",
                "1.1",
                vec![
                    XmlDocument::new("chip-types.xml", chip_types),
                    XmlDocument::new("onoff.xml", cluster_xml),
                ],
            ),
        ];

        let spec1 = build(inputs.clone()).expect("build 1");
        let spec2 = build(inputs).expect("build 2");
        let plan1 = spec1.shared_plan();
        let plan2 = spec2.shared_plan();
        assert_eq!(plan1, plan2, "shared_plan must be byte-identical across runs");

        let cluster_buckets: Vec<_> = plan1
            .buckets
            .values()
            .filter(|b| b.kind == matter_spec_gen::SharedKind::Cluster)
            .collect();
        assert_eq!(
            cluster_buckets.len(),
            1,
            "identical clusters across v1_0 and v1_1 must collapse into 1 shared bucket"
        );
        assert_eq!(
            cluster_buckets[0].cfg_gate,
            r#"all(cluster_on_off, any(feature = "v1_0", feature = "v1_1"))"#
        );
    }

    #[test]
    fn shared_hash_sensitivity_to_metadata() {
        let chip_types = r#"<configurator><atomic><type id="0x20" name="int8u" size="1" /></atomic></configurator>"#;
        let base_xml = r#"<configurator>
            <domain name="General" />
            <cluster>
                <name>OnOff</name>
                <code>0x0006</code>
                <globalAttribute code="0xFFFD" side="either" value="6" />
                <attribute side="server" code="0x0000" define="ON_OFF" type="int8u" min="0" max="1" writable="false" optional="false">OnOff</attribute>
            </cluster>
        </configurator>"#;
        let rev_changed_xml = r#"<configurator>
            <domain name="General" />
            <cluster>
                <name>OnOff</name>
                <code>0x0006</code>
                <globalAttribute code="0xFFFD" side="either" value="7" />
                <attribute side="server" code="0x0000" define="ON_OFF" type="int8u" min="0" max="1" writable="false" optional="false">OnOff</attribute>
            </cluster>
        </configurator>"#;

        let spec = build(vec![
            SpecInput::new(
                "v1_0",
                "1.0",
                vec![
                    XmlDocument::new("chip-types.xml", chip_types),
                    XmlDocument::new("onoff.xml", base_xml),
                ],
            ),
            SpecInput::new(
                "v1_1",
                "1.1",
                vec![
                    XmlDocument::new("chip-types.xml", chip_types),
                    XmlDocument::new("onoff.xml", rev_changed_xml),
                ],
            ),
        ])
        .expect("build");

        let plan = spec.shared_plan();
        let cluster_buckets: Vec<_> = plan
            .buckets
            .values()
            .filter(|b| b.kind == matter_spec_gen::SharedKind::Cluster)
            .collect();
        assert!(
            cluster_buckets.is_empty(),
            "changing CLUSTER_REVISION must produce distinct hashes so single-version clusters are not placed in \
             __shared"
        );
    }
}
