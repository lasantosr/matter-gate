use std::collections::{BTreeMap, BTreeSet};

use super::{
    BitmapDef, ClusterDef, EnumDef, FeatureDef, Result, StructDef, parse_attribute_xml, parse_bitmap_def,
    parse_enum_def, parse_hex_or_dec, parse_struct_def, sanitize_identifier,
};
use crate::{
    Error,
    model::parser::types::{
        bitmap_cluster_codes, enum_cluster_codes, parse_command_xml, parse_event_xml, struct_cluster_codes,
    },
    xml::*,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct ParsedCsaFile {
    pub(crate) clusters: Vec<ClusterDef>,
    pub(crate) extensions: Vec<ClusterExtensionXml>,
    pub(crate) standalone_structs: Vec<(StructDef, Vec<u32>)>,
    pub(crate) standalone_enums: Vec<(EnumDef, Vec<u32>)>,
    pub(crate) standalone_bitmaps: Vec<(BitmapDef, Vec<u32>)>,
}

#[cfg(test)]
pub(crate) fn parse_csa_xml(xml: &str) -> Result<ParsedCsaFile> {
    let config = ConfiguratorXml::from_xml(xml)?;
    parse_csa_configurator(config)
}

struct RawCsaConfigurator {
    file_domain: Option<String>,
    clusters: Vec<ClusterXml>,
    enums: Vec<EnumXml>,
    bitmaps: Vec<BitmapXml>,
    structs: Vec<StructXml>,
    extensions: Vec<ClusterExtensionXml>,
}

fn partition_csa_items(items: Vec<ConfiguratorItem>) -> RawCsaConfigurator {
    let mut file_domain = None;
    let mut clusters = Vec::new();
    let mut enums = Vec::new();
    let mut bitmaps = Vec::new();
    let mut structs = Vec::new();
    let mut extensions = Vec::new();

    for item in items {
        match item {
            ConfiguratorItem::Domain(domain_xml) => {
                if file_domain.is_none() {
                    file_domain = domain_xml.as_str().map(ToOwned::to_owned);
                }
            }
            ConfiguratorItem::Cluster(cl) => clusters.push(cl),
            ConfiguratorItem::Enum(enm) => enums.push(enm),
            ConfiguratorItem::Bitmap(bm) => bitmaps.push(bm),
            ConfiguratorItem::Struct(st) => structs.push(st),
            ConfiguratorItem::ClusterExtension(ext) => extensions.push(ext),
            ConfiguratorItem::AccessControl(_)
            | ConfiguratorItem::Atomic(_)
            | ConfiguratorItem::Global(_)
            | ConfiguratorItem::DeviceType(_)
            | ConfiguratorItem::Text(_) => {}
        }
    }

    RawCsaConfigurator {
        file_domain,
        clusters,
        enums,
        bitmaps,
        structs,
        extensions,
    }
}

fn parse_global_attribute(global_attr: GlobalAttributeXml, cluster_revision: &mut u16, features: &mut Vec<FeatureDef>) {
    let GlobalAttributeXml {
        code,
        value,
        items: ga_items,
        ..
    } = global_attr;

    if code.eq_ignore_ascii_case("0xFFFD") {
        let rev_opt = value
            .as_deref()
            .and_then(parse_hex_or_dec)
            .and_then(|val| u16::try_from(val).ok());
        if let Some(rev) = rev_opt {
            *cluster_revision = rev;
        }
        return;
    }

    if !code.eq_ignore_ascii_case("0xFFFC") {
        return;
    }

    for gai in ga_items {
        let GlobalAttributeItem::FeatureBit(fb) = gai else {
            continue;
        };
        let bit_val = parse_hex_or_dec(&fb.bit).unwrap_or(0);
        let code_str = fb.code.or(fb.tag).unwrap_or_default();
        let name_str = fb.name.unwrap_or_else(|| code_str.clone());
        let summary_str = fb.summary.unwrap_or_else(|| name_str.clone());
        let clean_name = sanitize_identifier(&name_str);
        if !clean_name.is_empty() && !features.iter().any(|feat: &FeatureDef| feat.bit == bit_val) {
            features.push(FeatureDef {
                bit: bit_val,
                code: code_str,
                name: clean_name,
                summary: summary_str,
            });
        }
    }
}

fn parse_features_xml(features_xml: FeaturesXml, features: &mut Vec<FeatureDef>) {
    for fi in features_xml.items {
        let FeaturesItem::Feature(feat) = fi else { continue };
        let bit_val = feat.bit.as_deref().and_then(parse_hex_or_dec).unwrap_or(0);
        let feat_code = feat.code.unwrap_or_default();
        let feat_name = sanitize_identifier(feat.name.as_deref().unwrap_or(""));
        let feat_summary = feat.summary.unwrap_or_default();
        if !feat_name.is_empty() && !features.iter().any(|existing: &FeatureDef| existing.bit == bit_val) {
            features.push(FeatureDef {
                bit: bit_val,
                code: feat_code,
                name: feat_name,
                summary: feat_summary,
            });
        }
    }
}

fn filter_applicable_enums(parsed_enums: &[(EnumDef, Vec<u32>)], code: u32) -> Vec<EnumDef> {
    let mut enums = Vec::new();
    let mut seen = BTreeSet::new();
    for (enm_def, codes) in parsed_enums {
        let applies = if codes.is_empty() { true } else { codes.contains(&code) };
        if applies && seen.insert(enm_def.name.clone()) {
            enums.push(enm_def.clone());
        }
    }
    enums
}

fn filter_applicable_bitmaps(parsed_bitmaps: &[(BitmapDef, Vec<u32>)], code: u32) -> Vec<BitmapDef> {
    let mut bitmaps = Vec::new();
    let mut seen = BTreeSet::new();
    for (bm_def, codes) in parsed_bitmaps {
        let applies = if codes.is_empty() { true } else { codes.contains(&code) };
        if applies && seen.insert(bm_def.name.clone()) {
            bitmaps.push(bm_def.clone());
        }
    }
    bitmaps
}

fn filter_applicable_structs(parsed_structs: &[(StructDef, Vec<u32>)], code: u32) -> Vec<StructDef> {
    let mut structs = Vec::new();
    let mut seen = BTreeSet::new();
    for (st_def, codes) in parsed_structs {
        let applies = if codes.is_empty() { true } else { codes.contains(&code) };
        if applies && seen.insert(st_def.name.clone()) {
            structs.push(st_def.clone());
        }
    }
    structs
}

fn build_cluster_def(
    cluster_xml: ClusterXml,
    parsed_enums: &[(EnumDef, Vec<u32>)],
    parsed_bitmaps: &[(BitmapDef, Vec<u32>)],
    parsed_structs: &[(StructDef, Vec<u32>)],
    default_domain: Option<&str>,
) -> Result<ClusterDef> {
    let ClusterXml {
        code_attr,
        items: c_items,
        ..
    } = cluster_xml;

    let mut code_val = code_attr.as_deref().and_then(parse_hex_or_dec);
    let mut raw_name = String::new();
    let mut cluster_domain = None;
    let mut description = String::new();
    let mut cluster_revision = 1_u16;
    let mut features = Vec::new();
    let mut raw_attributes = Vec::new();
    let mut raw_commands = Vec::new();
    let mut raw_events = Vec::new();

    for c_item in c_items {
        match c_item {
            ClusterItem::Name(SimpleTextXml { text }) => {
                raw_name = text;
            }
            ClusterItem::Domain(domain_xml) => {
                if let Some(domain_name) = domain_xml.as_str() {
                    cluster_domain = Some(domain_name.to_owned());
                }
            }
            ClusterItem::Description(SimpleTextXml { text }) => {
                description = text;
            }
            ClusterItem::Code(SimpleTextXml { text }) => {
                if let Some(code) = parse_hex_or_dec(&text) {
                    code_val = Some(code);
                }
            }
            ClusterItem::GlobalAttribute(global_attr) => {
                parse_global_attribute(global_attr, &mut cluster_revision, &mut features);
            }
            ClusterItem::Features(features_xml) => {
                parse_features_xml(features_xml, &mut features);
            }
            ClusterItem::Attribute(attr_xml) => {
                raw_attributes.push(attr_xml);
            }
            ClusterItem::Command(cmd_xml) => {
                raw_commands.push(cmd_xml);
            }
            ClusterItem::Event(event_xml) => {
                raw_events.push(event_xml);
            }
            ClusterItem::Define(_)
            | ClusterItem::Client(_)
            | ClusterItem::Server(_)
            | ClusterItem::Tag(_)
            | ClusterItem::Text(_) => {}
        }
    }

    let code = code_val.unwrap_or(0);
    let name = sanitize_identifier(&raw_name);
    let domain_ref = cluster_domain.as_deref().or(default_domain).unwrap_or("General");
    let domain = domain_ref.to_owned();

    let enums = filter_applicable_enums(parsed_enums, code);
    let bitmaps = filter_applicable_bitmaps(parsed_bitmaps, code);
    let structs = filter_applicable_structs(parsed_structs, code);

    let mut attributes = Vec::new();
    let mut seen_attributes = BTreeSet::new();
    for attr_xml in raw_attributes {
        if let Some(attr_def) = parse_attribute_xml(attr_xml)
            && seen_attributes.insert(attr_def.code)
        {
            attributes.push(attr_def);
        }
    }

    let mut commands = Vec::new();
    let mut seen_commands = BTreeSet::new();
    for cmd_xml in raw_commands {
        if let Some(cmd_def) = parse_command_xml(cmd_xml)?
            && seen_commands.insert((cmd_def.code, cmd_def.source.clone()))
        {
            commands.push(cmd_def);
        }
    }

    let mut events = Vec::new();
    let mut seen_events = BTreeSet::new();
    for event_xml in raw_events {
        if let Some(event_def) = parse_event_xml(event_xml)?
            && seen_events.insert(event_def.code)
        {
            events.push(event_def);
        }
    }

    let mut choice_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for attr in &attributes {
        if let Some(ch) = &attr.choice {
            choice_map.entry(ch.clone()).or_default().push(attr.name.clone());
        }
    }
    for cmd in &commands {
        if let Some(ch) = &cmd.choice {
            choice_map.entry(ch.clone()).or_default().push(cmd.name.clone());
        }
    }
    for ev in &events {
        if let Some(ch) = &ev.choice {
            choice_map.entry(ch.clone()).or_default().push(ev.name.clone());
        }
    }
    let choices = choice_map
        .into_iter()
        .map(|(choice_name, members)| super::ChoiceGroup {
            name: choice_name,
            min: Some(1),
            max: Some(1),
            more: false,
            members,
        })
        .collect();

    Ok(ClusterDef {
        name,
        code,
        cluster_revision,
        domain,
        description,
        features,
        bitmaps,
        enums,
        structs,
        attributes,
        commands,
        events,
        choices,
    })
}

pub(crate) fn parse_csa_configurator(config: ConfiguratorXml) -> Result<ParsedCsaFile> {
    let RawCsaConfigurator {
        file_domain,
        clusters: raw_clusters,
        enums: raw_enums,
        bitmaps: raw_bitmaps,
        structs: raw_structs,
        extensions: raw_extensions,
    } = partition_csa_items(config.items);

    let parsed_enums: Vec<(EnumDef, Vec<u32>)> = raw_enums
        .iter()
        .map(|enm| Ok((parse_enum_def(enm)?, enum_cluster_codes(enm))))
        .collect::<Result<Vec<_>>>()?;

    let parsed_bitmaps: Vec<(BitmapDef, Vec<u32>)> = raw_bitmaps
        .iter()
        .map(|bm| (parse_bitmap_def(bm), bitmap_cluster_codes(bm)))
        .collect();

    let parsed_structs: Vec<(StructDef, Vec<u32>)> = raw_structs
        .iter()
        .map(|st| Ok((parse_struct_def(st)?, struct_cluster_codes(st))))
        .collect::<Result<Vec<_>>>()?;

    if raw_clusters.is_empty() {
        return Ok(ParsedCsaFile {
            clusters: Vec::new(),
            extensions: raw_extensions,
            standalone_structs: parsed_structs,
            standalone_enums: parsed_enums,
            standalone_bitmaps: parsed_bitmaps,
        });
    }

    let default_domain = file_domain.as_deref();
    let clusters = raw_clusters
        .into_iter()
        .map(|cl| build_cluster_def(cl, &parsed_enums, &parsed_bitmaps, &parsed_structs, default_domain))
        .collect::<Result<Vec<_>>>()?;

    Ok(ParsedCsaFile {
        clusters,
        extensions: raw_extensions,
        standalone_structs: parsed_structs,
        standalone_enums: parsed_enums,
        standalone_bitmaps: parsed_bitmaps,
    })
}

#[cfg(test)]
pub(crate) fn parse_csa_clusters(xml: &str) -> Result<Vec<ClusterDef>> {
    let parsed = parse_csa_xml(xml)?;
    let mut clusters = parsed.clusters;
    if !parsed.extensions.is_empty() {
        merge_cluster_extensions_internal(&mut clusters, parsed.extensions, false)?;
    }
    Ok(clusters)
}

pub(crate) fn merge_cluster_extensions(
    clusters: &mut [ClusterDef],
    extensions: Vec<ClusterExtensionXml>,
) -> Result<()> {
    merge_cluster_extensions_internal(clusters, extensions, true)
}

fn merge_cluster_extensions_internal(
    clusters: &mut [ClusterDef],
    extensions: Vec<ClusterExtensionXml>,
    strict: bool,
) -> Result<()> {
    for ext in extensions {
        let ClusterExtensionXml {
            code: ext_code_str,
            items,
        } = ext;
        let Some(ext_code) = parse_hex_or_dec(&ext_code_str) else {
            if strict {
                return Err(Error::InvalidClusterExtensionCode(ext_code_str));
            }
            continue;
        };

        let Some(cluster) = clusters.iter_mut().find(|cluster| cluster.code == ext_code) else {
            if strict {
                return Err(Error::UnknownClusterExtensionCode { code: ext_code });
            }
            continue;
        };

        let mut seen_attr_codes: BTreeSet<u32> = cluster.attributes.iter().map(|attr| attr.code).collect();
        let mut seen_cmd_keys: BTreeSet<(u32, String)> = cluster
            .commands
            .iter()
            .map(|cmd| (cmd.code, cmd.source.clone()))
            .collect();

        for item in items {
            match item {
                ClusterExtensionItem::Attribute(attr_xml) => {
                    if let Some(attr_def) = parse_attribute_xml(attr_xml)
                        && seen_attr_codes.insert(attr_def.code)
                    {
                        cluster.attributes.push(attr_def);
                    }
                }
                ClusterExtensionItem::Command(cmd_xml) => {
                    if let Some(cmd_def) = parse_command_xml(cmd_xml)?
                        && seen_cmd_keys.insert((cmd_def.code, cmd_def.source.clone()))
                    {
                        cluster.commands.push(cmd_def);
                    }
                }
                ClusterExtensionItem::Text(_) => {}
            }
        }
    }
    Ok(())
}
