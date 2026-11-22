use std::collections::BTreeMap;

use crate::{
    build::{VersionSpec, is_valid_domain_device},
    category::Category,
    model::to_snake_case,
};

pub fn reconcile_categories_across_versions(versions: &mut [VersionSpec]) {
    // 1. Build canonical non-general cluster category map (newest wins)
    let mut cluster_canonical: BTreeMap<u32, Category> = BTreeMap::new();
    let mut cluster_name_canonical: BTreeMap<String, Category> = BTreeMap::new();

    for cluster in versions.iter().flat_map(|ver| &ver.clusters) {
        let cat = cluster.category();
        if *cat != Category::GENERAL {
            cluster_canonical.insert(cluster.cluster_def.code, cat.clone());
            cluster_name_canonical.insert(to_snake_case(&cluster.original_name), cat.clone());
        }
    }

    // 2. Build canonical non-general device type category map (newest wins)
    let mut dt_canonical: BTreeMap<u32, Category> = BTreeMap::new();
    let mut dt_name_canonical: BTreeMap<String, Category> = BTreeMap::new();

    for item in versions.iter().flat_map(|ver| &ver.device_types) {
        if *item.category() != Category::GENERAL && is_valid_domain_device(item.device_id) {
            dt_canonical.insert(item.device_id, item.category().clone());
            dt_name_canonical.insert(to_snake_case(&item.original_name), item.category().clone());
        }
    }

    // 3. Update each version's clusters and device types
    for ver in versions.iter_mut() {
        for cluster in &mut ver.clusters {
            if cluster.category == Category::testing() {
                continue;
            }
            let canonical_cat = cluster_canonical
                .get(&cluster.cluster_def.code)
                .or_else(|| cluster_name_canonical.get(&to_snake_case(&cluster.original_name)));
            if let Some(target_cat) = canonical_cat
                && cluster.category != *target_cat
            {
                cluster.category = target_cat.clone();
                cluster.cluster_def.domain = target_cat.as_str().to_owned();
            }
        }

        for item in &mut ver.device_types {
            let target_cat = if is_valid_domain_device(item.device_id) {
                dt_canonical
                    .get(&item.device_id)
                    .or_else(|| dt_name_canonical.get(&to_snake_case(&item.original_name)))
                    .cloned()
                    .unwrap_or_else(Category::general)
            } else {
                Category::general()
            };
            item.device_type.domain = target_cat.as_str().to_owned();
            item.category = target_cat;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build::resolve_device_type_categories,
        model::{
            ParsedDeviceType,
            parser::{ComposedEndpointDef, DeviceTypeClusterRequirement},
        },
    };

    #[test]
    fn pseudo_devices_remain_general_and_are_never_donors() {
        let mut cluster_to_category = BTreeMap::new();
        cluster_to_category.insert("Identify".to_owned(), Category::from_xml_domain("General").unwrap());
        cluster_to_category.insert("Descriptor".to_owned(), Category::from_xml_domain("General").unwrap());
        cluster_to_category.insert("OnOff".to_owned(), Category::from_xml_domain("Lighting").unwrap());

        let test_app = ParsedDeviceType {
            name: "AllClustersAppServerExample".to_owned(),
            type_name: "All-clusters-app Server Example".to_owned(),
            device_id: 0x0000,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Identify".to_owned(), "Descriptor".to_owned(), "OnOff".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: vec![
                DeviceTypeClusterRequirement {
                    name: "Identify".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "OnOff".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
            ],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let orphan = ParsedDeviceType {
            name: "OrphanClusters".to_owned(),
            type_name: "Orphan Clusters".to_owned(),
            device_id: 0xFFF1_0001,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Identify".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: Vec::new(),
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cats = resolve_device_type_categories(&[test_app, orphan], &cluster_to_category);
        assert_eq!(cats[0], Category::general());
        assert_eq!(cats[1], Category::general());
    }

    #[test]
    fn cook_surface_tie_breaking_prioritizes_earlier_cluster_declaration() {
        let mut cluster_to_category = BTreeMap::new();
        cluster_to_category.insert("OnOff".to_owned(), Category::general());
        cluster_to_category.insert("Descriptor".to_owned(), Category::general());
        cluster_to_category.insert(
            "TemperatureControl".to_owned(),
            Category::from_xml_domain("Appliances").unwrap(),
        );
        cluster_to_category.insert(
            "TemperatureMeasurement".to_owned(),
            Category::from_xml_domain("Measurement & Sensing").unwrap(),
        );

        let cook_surface = ParsedDeviceType {
            name: "CookSurface".to_owned(),
            type_name: "Cook Surface".to_owned(),
            device_id: 0x0077,
            revision: 2,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Descriptor".to_owned()],
            optional_server_clusters: vec![
                "OnOff".to_owned(),
                "TemperatureControl".to_owned(),
                "TemperatureMeasurement".to_owned(),
            ],
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: vec![
                DeviceTypeClusterRequirement {
                    name: "OnOff".to_owned(),
                    role_priority: 2,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "Descriptor".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "TemperatureControl".to_owned(),
                    role_priority: 2,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "TemperatureMeasurement".to_owned(),
                    role_priority: 2,
                    has_lighting_feature: false,
                },
            ],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cats = resolve_device_type_categories(&[cook_surface], &cluster_to_category);
        assert_eq!(cats[0], Category::from_xml_domain("Appliances").unwrap());
    }

    #[test]
    fn superset_inheritance_propagates_upward_and_downward() {
        let mut cluster_to_category = BTreeMap::new();
        cluster_to_category.insert(
            "ColorControl".to_owned(),
            Category::from_xml_domain("Lighting").unwrap(),
        );

        let switch = ParsedDeviceType {
            name: "OnOffLightSwitch".to_owned(),
            type_name: "On/Off Light Switch".to_owned(),
            device_id: 0x0103,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Identify".to_owned(), "Descriptor".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: Vec::new(),
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let dimmer = ParsedDeviceType {
            name: "DimmerSwitch".to_owned(),
            type_name: "Dimmer Switch".to_owned(),
            device_id: 0x0104,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: Some("OnOffLightSwitch".to_owned()),
            mandatory_server_clusters: vec!["Identify".to_owned(), "Descriptor".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: Vec::new(),
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let color_dimmer = ParsedDeviceType {
            name: "ColorDimmerSwitch".to_owned(),
            type_name: "Color Dimmer Switch".to_owned(),
            device_id: 0x0105,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: Some("DimmerSwitch".to_owned()),
            mandatory_server_clusters: vec!["Identify".to_owned(), "Descriptor".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: vec!["ColorControl".to_owned()],
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: vec![DeviceTypeClusterRequirement {
                name: "ColorControl".to_owned(),
                role_priority: 1,
                has_lighting_feature: false,
            }],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cats = resolve_device_type_categories(&[switch, dimmer, color_dimmer], &cluster_to_category);
        assert_eq!(cats[0], Category::from_xml_domain("Lighting").unwrap());
        assert_eq!(cats[1], Category::from_xml_domain("Lighting").unwrap());
        assert_eq!(cats[2], Category::from_xml_domain("Lighting").unwrap());
    }

    #[test]
    fn endpoint_composition_propagates_upward_with_weights() {
        let mut cluster_to_category = BTreeMap::new();
        cluster_to_category.insert(
            "CameraAvStreamManagement".to_owned(),
            Category::from_xml_domain("Cameras").unwrap(),
        );
        cluster_to_category.insert(
            "WebRtcTransportProvider".to_owned(),
            Category::from_xml_domain("Cameras").unwrap(),
        );

        let light = ParsedDeviceType {
            name: "OnOffLight".to_owned(),
            type_name: "On/Off Light".to_owned(),
            device_id: 0x0100,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Identify".to_owned(), "OnOff".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: true,
            cluster_requirements: vec![DeviceTypeClusterRequirement {
                name: "OnOff".to_owned(),
                role_priority: 3,
                has_lighting_feature: true,
            }],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let camera = ParsedDeviceType {
            name: "Camera".to_owned(),
            type_name: "Camera".to_owned(),
            device_id: 0x0142,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec![
                "CameraAvStreamManagement".to_owned(),
                "WebRtcTransportProvider".to_owned(),
            ],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: vec![
                DeviceTypeClusterRequirement {
                    name: "CameraAvStreamManagement".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "WebRtcTransportProvider".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
            ],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let floodlight_camera = ParsedDeviceType {
            name: "FloodlightCamera".to_owned(),
            type_name: "Floodlight Camera".to_owned(),
            device_id: 0x0144,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: Vec::new(),
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: Vec::new(),
            composed_endpoints: vec![
                ComposedEndpointDef {
                    device_type_id: Some(0x0100),
                    device_type_name: Some("OnOffLight".to_owned()),
                    clusters: Vec::new(),
                },
                ComposedEndpointDef {
                    device_type_id: Some(0x0142),
                    device_type_name: Some("Camera".to_owned()),
                    clusters: Vec::new(),
                },
            ],
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cats = resolve_device_type_categories(&[light, camera, floodlight_camera], &cluster_to_category);

        assert_eq!(cats[0], Category::from_xml_domain("Lighting").unwrap());
        assert_eq!(cats[1], Category::from_xml_domain("Cameras").unwrap());
        assert_eq!(cats[2], Category::from_xml_domain("Cameras").unwrap());
    }

    #[test]
    fn endpoint_composition_propagates_downward_to_unresolved_child() {
        let mut cluster_to_category = BTreeMap::new();
        cluster_to_category.insert(
            "TemperatureControl".to_owned(),
            Category::from_xml_domain("Appliances").unwrap(),
        );

        let tcc = ParsedDeviceType {
            name: "TemperatureControlledCabinet".to_owned(),
            type_name: "Temperature Controlled Cabinet".to_owned(),
            device_id: 0x0071,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["TemperatureControl".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: vec![DeviceTypeClusterRequirement {
                name: "TemperatureControl".to_owned(),
                role_priority: 3,
                has_lighting_feature: false,
            }],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cooktop = ParsedDeviceType {
            name: "Cooktop".to_owned(),
            type_name: "Cooktop".to_owned(),
            device_id: 0x0078,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Descriptor".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: Vec::new(),
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let oven = ParsedDeviceType {
            name: "Oven".to_owned(),
            type_name: "Oven".to_owned(),
            device_id: 0x007B,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec!["Descriptor".to_owned()],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: Vec::new(),
            composed_endpoints: vec![
                ComposedEndpointDef {
                    device_type_id: Some(0x0071),
                    device_type_name: None,
                    clusters: Vec::new(),
                },
                ComposedEndpointDef {
                    device_type_id: Some(0x0078),
                    device_type_name: None,
                    clusters: Vec::new(),
                },
            ],
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cats = resolve_device_type_categories(&[tcc, cooktop, oven], &cluster_to_category);

        assert_eq!(cats[0], Category::from_xml_domain("Appliances").unwrap());
        assert_eq!(cats[1], Category::from_xml_domain("Appliances").unwrap());
        assert_eq!(cats[2], Category::from_xml_domain("Appliances").unwrap());
    }

    #[test]
    fn speaker_remains_in_general() {
        let mut cluster_to_category = BTreeMap::new();
        cluster_to_category.insert("Identify".to_owned(), Category::general());
        cluster_to_category.insert("Descriptor".to_owned(), Category::general());
        cluster_to_category.insert("OnOff".to_owned(), Category::general());
        cluster_to_category.insert("LevelControl".to_owned(), Category::general());

        let speaker = ParsedDeviceType {
            name: "Speaker".to_owned(),
            type_name: "Speaker".to_owned(),
            device_id: 0x0022,
            revision: 1,
            domain: "CHIP".to_owned(),
            class: "Simple".to_owned(),
            scope: "Endpoint".to_owned(),
            superset: None,
            mandatory_server_clusters: vec![
                "Identify".to_owned(),
                "Descriptor".to_owned(),
                "OnOff".to_owned(),
                "LevelControl".to_owned(),
            ],
            optional_server_clusters: Vec::new(),
            mandatory_client_clusters: Vec::new(),
            optional_client_clusters: Vec::new(),
            disallowed_clusters: Vec::new(),
            has_lighting_feature: false,
            cluster_requirements: vec![
                DeviceTypeClusterRequirement {
                    name: "Identify".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "Descriptor".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "OnOff".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
                DeviceTypeClusterRequirement {
                    name: "LevelControl".to_owned(),
                    role_priority: 3,
                    has_lighting_feature: false,
                },
            ],
            composed_endpoints: Vec::new(),
            includes: Vec::new(),
            choices: Vec::new(),
        };

        let cats = resolve_device_type_categories(&[speaker], &cluster_to_category);
        assert_eq!(cats[0], Category::general());
    }
}
