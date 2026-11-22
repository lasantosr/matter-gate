use super::*;

#[test]
fn deserialize_sample_cluster_snippet() {
    let snippet = r#"<configurator>
            <cluster>
                <name>OnOff</name>
                <domain>General</domain>
                <description>On/Off cluster</description>
                <code>0x0006</code>
                <define>ON_OFF_CLUSTER</define>
                <client init="false" tick="false">true</client>
                <server init="false" tick="false">true</server>
                <attribute code="0x0000" side="server" type="boolean" writable="false" optional="false">OnOff</attribute>
                <command code="0x00" source="client" name="Off" optional="false">
                    <description>Turn off</description>
                </command>
            </cluster>
        </configurator>"#;

    let parsed = ConfiguratorXml::from_xml(snippet).expect("valid cluster snippet");
    assert_eq!(parsed.items.len(), 1);
    match &parsed.items[0] {
        ConfiguratorItem::Cluster(cl) => {
            let non_text: Vec<_> = cl
                .items
                .iter()
                .filter(|item| !matches!(item, ClusterItem::Text(text) if text.trim().is_empty()))
                .collect();
            assert_eq!(non_text.len(), 9);
        }
        other => panic!("expected cluster, got {other:?}"),
    }
}

#[test]
fn deserialize_sample_enum_and_bitmap_snippet() {
    let snippet = r#"<configurator>
            <enum name="StatusEnum" type="enum8">
                <item name="Success" value="0"/>
                <item name="Failure" value="1"/>
            </enum>
            <bitmap name="FeatureMap" type="bitmap32">
                <field name="Lighting" mask="0x01"/>
            </bitmap>
        </configurator>"#;

    let parsed = ConfiguratorXml::from_xml(snippet).expect("valid enum and bitmap snippet");
    assert_eq!(parsed.items.len(), 2);
}
