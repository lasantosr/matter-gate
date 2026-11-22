use super::{
    ComposedEndpointDef, DeviceTypeClusterRequirement, ParsedDeviceType, ParsedDeviceTypeInclude, Result,
    parse_hex_or_dec, sanitize_identifier,
};
use crate::xml::*;

pub(crate) fn parse_device_types(xml: &str) -> Result<Vec<ParsedDeviceType>> {
    let config = ConfiguratorXml::from_xml(xml)?;
    Ok(parse_device_types_configurator(config))
}

pub(crate) fn parse_device_types_configurator(config: ConfiguratorXml) -> Vec<ParsedDeviceType> {
    config
        .items
        .into_iter()
        .filter_map(|item| {
            if let ConfiguratorItem::DeviceType(dt) = item {
                parse_single_device_type(dt)
            } else {
                None
            }
        })
        .collect()
}

struct ParsedDeviceTypeClusters {
    mandatory_server: Vec<String>,
    optional_server: Vec<String>,
    mandatory_client: Vec<String>,
    optional_client: Vec<String>,
    disallowed: Vec<String>,
    cluster_requirements: Vec<DeviceTypeClusterRequirement>,
    includes: Vec<ParsedDeviceTypeInclude>,
}

fn classify_device_type_includes(includes: Vec<DeviceTypeIncludeXml>) -> ParsedDeviceTypeClusters {
    let mut mandatory_server = Vec::new();
    let mut optional_server = Vec::new();
    let mut mandatory_client = Vec::new();
    let mut optional_client = Vec::new();
    let mut disallowed = Vec::new();
    let mut cluster_requirements = Vec::new();
    let mut parsed_includes = Vec::new();

    for inc in includes {
        let cluster_raw = inc.cluster.trim();
        if cluster_raw.is_empty() {
            continue;
        }
        let is_server = inc.server.as_deref() == Some("true");
        let is_client = inc.client.as_deref() == Some("true");
        let server_is_locked = inc.server_locked.as_deref() == Some("true");
        let client_is_locked = inc.client_locked.as_deref() == Some("true");
        let has_lighting_feature = include_has_lighting_feature(&inc.items);

        let mut required_attributes = Vec::new();
        let mut required_commands = Vec::new();
        let mut required_events = Vec::new();
        for item in &inc.items {
            match item {
                DeviceTypeIncludeItem::RequireAttribute(t) => {
                    let trimmed = t.text.trim();
                    if !trimmed.is_empty() {
                        required_attributes.push(trimmed.to_owned());
                    }
                }
                DeviceTypeIncludeItem::RequireCommand(t) => {
                    let trimmed = t.text.trim();
                    if !trimmed.is_empty() {
                        required_commands.push(trimmed.to_owned());
                    }
                }
                DeviceTypeIncludeItem::RequireEvent(t) => {
                    let trimmed = t.text.trim();
                    if !trimmed.is_empty() {
                        required_events.push(trimmed.to_owned());
                    }
                }
                DeviceTypeIncludeItem::Features(_) | DeviceTypeIncludeItem::Text(_) => {}
            }
        }
        if !required_attributes.is_empty() || !required_commands.is_empty() || !required_events.is_empty() {
            parsed_includes.push(ParsedDeviceTypeInclude {
                cluster: sanitize_identifier(cluster_raw),
                is_server: is_server || !is_client,
                required_attributes,
                required_commands,
                required_events,
            });
        }

        let cluster_string = cluster_raw.to_owned();

        let role_priority = if is_server && server_is_locked {
            3
        } else if !server_is_locked && (is_server || !is_client) {
            2
        } else {
            u8::from(is_client && client_is_locked)
        };

        cluster_requirements.push(DeviceTypeClusterRequirement {
            name: sanitize_identifier(&cluster_string),
            role_priority,
            has_lighting_feature,
        });

        if is_server && server_is_locked && !mandatory_server.contains(&cluster_string) {
            mandatory_server.push(cluster_string.clone());
        }
        if !server_is_locked && (is_server || !is_client) && !optional_server.contains(&cluster_string) {
            optional_server.push(cluster_string.clone());
        }

        if is_client && client_is_locked && !mandatory_client.contains(&cluster_string) {
            mandatory_client.push(cluster_string.clone());
        }
        if is_client && !client_is_locked && !optional_client.contains(&cluster_string) {
            optional_client.push(cluster_string.clone());
        }

        if !is_server && server_is_locked && !disallowed.contains(&cluster_string) {
            disallowed.push(cluster_string);
        }
    }

    ParsedDeviceTypeClusters {
        mandatory_server,
        optional_server,
        mandatory_client,
        optional_client,
        disallowed,
        cluster_requirements,
        includes: parsed_includes,
    }
}

fn include_has_lighting_feature(items: &[DeviceTypeIncludeItem]) -> bool {
    for inc_item in items {
        let DeviceTypeIncludeItem::Features(feats) = inc_item else {
            continue;
        };
        for feat_item in &feats.items {
            let FeaturesItem::Feature(feat) = feat_item else {
                continue;
            };
            let is_lt = feat
                .code
                .as_deref()
                .is_some_and(|code_str| code_str.eq_ignore_ascii_case("LT"))
                || feat
                    .name
                    .as_deref()
                    .is_some_and(|name_str| name_str.eq_ignore_ascii_case("Lighting"));
            if is_lt {
                return true;
            }
        }
    }
    false
}

fn parse_single_composed_device(sub_dt: ComposedDeviceTypeXml) -> ComposedEndpointDef {
    let mut sub_id = sub_dt.id.as_deref().and_then(parse_hex_or_dec);
    let sub_name = sub_dt.name.map(|raw_name| {
        let clean = raw_name.strip_prefix("Matter ").unwrap_or(&raw_name);
        sanitize_identifier(clean)
    });
    let mut clusters = Vec::new();

    for item in sub_dt.content {
        match item {
            ComposedDeviceTypeContentXml::Clusters(clusters_xml) => {
                for cluster_item in clusters_xml.items {
                    if let DeviceTypeClustersItem::Include(inc) = cluster_item {
                        clusters.push(sanitize_identifier(&inc.cluster));
                    }
                }
            }
            ComposedDeviceTypeContentXml::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() && sub_id.is_none() {
                    sub_id = parse_hex_or_dec(trimmed);
                }
            }
        }
    }

    ComposedEndpointDef {
        device_type_id: sub_id,
        device_type_name: sub_name,
        clusters,
    }
}

fn parse_endpoint_composition_sub_devices(
    endpoint_items: Vec<EndpointCompositionItem>,
    composed_endpoints: &mut Vec<ComposedEndpointDef>,
) {
    for comp_item in endpoint_items {
        let EndpointCompositionItem::Endpoint(endpoint) = comp_item else {
            continue;
        };
        for ep_item in endpoint.items {
            let EndpointItem::DeviceType(sub_dt) = ep_item else {
                continue;
            };
            composed_endpoints.push(parse_single_composed_device(sub_dt));
        }
    }
}

fn parse_device_type_clusters(
    c_items: Vec<DeviceTypeClustersItem>,
    includes: &mut Vec<DeviceTypeIncludeXml>,
    has_lighting_feature: &mut bool,
) {
    for ci in c_items {
        if let DeviceTypeClustersItem::Include(inc) = ci {
            if include_has_lighting_feature(&inc.items) {
                *has_lighting_feature = true;
            }
            includes.push(inc);
        }
    }
}

fn parse_single_device_type(dt: DeviceTypeXml) -> Option<ParsedDeviceType> {
    let DeviceTypeXml {
        id_attr,
        name_attr,
        items: dt_items,
    } = dt;

    let mut type_name_opt = None;
    let mut device_id_opt = id_attr.as_deref().and_then(parse_hex_or_dec);
    let mut domain = "CHIP".to_owned();
    let mut class = "Simple".to_owned();
    let mut scope = "Endpoint".to_owned();
    let mut superset = None;
    let mut revision = 1_u16;
    let mut includes = Vec::new();
    let mut has_lighting_feature = false;
    let mut composed_endpoints = Vec::new();

    for it in dt_items {
        match it {
            DeviceTypeItem::Name(SimpleTextXml { text }) => {
                if type_name_opt.is_none() {
                    type_name_opt = Some(text);
                }
            }
            DeviceTypeItem::Domain(SimpleTextXml { text }) => {
                domain = text;
            }
            DeviceTypeItem::TypeName(SimpleTextXml { text }) => {
                type_name_opt = Some(text);
            }
            DeviceTypeItem::DeviceId(EditableTextXml { text, .. }) => {
                if let Some(id) = parse_hex_or_dec(&text) {
                    device_id_opt = Some(id);
                }
            }
            DeviceTypeItem::Class(SimpleTextXml { text }) => {
                class = text;
            }
            DeviceTypeItem::Scope(SimpleTextXml { text }) => {
                scope = text;
            }
            DeviceTypeItem::Superset(SimpleTextXml { text }) => {
                let clean_superset = text.strip_prefix("Matter ").unwrap_or(&text);
                superset = Some(sanitize_identifier(clean_superset));
            }
            DeviceTypeItem::Revision(RevisionXml { value, text, .. }) => {
                let rev_num = value
                    .as_deref()
                    .or(text.as_deref())
                    .and_then(parse_hex_or_dec)
                    .and_then(|v| u16::try_from(v).ok());
                if let Some(rev) = rev_num {
                    revision = rev;
                }
            }
            DeviceTypeItem::Clusters(DeviceTypeClustersXml { items: c_items, .. }) => {
                parse_device_type_clusters(c_items, &mut includes, &mut has_lighting_feature);
            }
            DeviceTypeItem::EndpointComposition(EndpointCompositionXml { items }) => {
                parse_endpoint_composition_sub_devices(items, &mut composed_endpoints);
            }
            DeviceTypeItem::ProfileId(_) | DeviceTypeItem::Text(_) => {}
        }
    }

    let raw_type_name = type_name_opt.or(name_attr)?;
    let clean_type_name = raw_type_name
        .strip_prefix("Matter ")
        .unwrap_or(&raw_type_name)
        .to_owned();
    let device_id = device_id_opt?;
    let name = sanitize_identifier(&clean_type_name);

    let ParsedDeviceTypeClusters {
        mandatory_server,
        optional_server,
        mandatory_client,
        optional_client,
        disallowed,
        cluster_requirements,
        includes: parsed_includes,
    } = classify_device_type_includes(includes);

    Some(ParsedDeviceType {
        name,
        type_name: clean_type_name,
        device_id,
        revision,
        domain,
        class,
        scope,
        superset,
        mandatory_server_clusters: mandatory_server,
        optional_server_clusters: optional_server,
        mandatory_client_clusters: mandatory_client,
        optional_client_clusters: optional_client,
        disallowed_clusters: disallowed,
        has_lighting_feature,
        cluster_requirements,
        composed_endpoints,
        includes: parsed_includes,
        choices: Vec::new(),
    })
}
