//! Raw XML ingestion and initial deserialization into version-local intermediate definitions.

use std::collections::BTreeMap;

use super::{Result, XmlDocument};
use crate::{
    model::{
        BitmapDef, ClusterDef, EnumDef, GlobalDefs, ParsedDeviceType, StructDef, parse_bitmap_def,
        parse_chip_types_xml, parse_csa_configurator, parse_device_types, parse_enum_def, parse_struct_def,
    },
    xml::{ClusterExtensionXml, ConfiguratorItem, ConfiguratorXml},
};

/// Unmerged intermediate tables collected while scanning all XML documents for one Matter release.
#[derive(Default)]
pub(super) struct ParsedRawVersionData {
    pub(super) chip_types: BTreeMap<String, (&'static str, &'static str)>,
    pub(super) global_defs: GlobalDefs,
    pub(super) device_types: Vec<ParsedDeviceType>,
    pub(super) all_enums: BTreeMap<String, EnumDef>,
    pub(super) all_bitmaps: BTreeMap<String, BitmapDef>,
    pub(super) all_structs: BTreeMap<String, StructDef>,
    pub(super) clusters: Vec<ClusterDef>,
    pub(super) cluster_extensions: Vec<ClusterExtensionXml>,
    pub(super) standalone_structs: Vec<(StructDef, Vec<u32>)>,
    pub(super) standalone_enums: Vec<(EnumDef, Vec<u32>)>,
    pub(super) standalone_bitmaps: Vec<(BitmapDef, Vec<u32>)>,
}

/// Sorts documents deterministically and parses chip types, globals, clusters, and device types.
pub(super) fn process_raw_documents<D, I>(documents: I) -> Result<ParsedRawVersionData>
where
    D: AsRef<str>,
    I: IntoIterator<Item = XmlDocument<D>>,
{
    let mut data = ParsedRawVersionData::default();
    let mut sorted_docs: Vec<_> = documents.into_iter().collect();
    sorted_docs.sort_by(|a, b| a.name.cmp(&b.name));

    for doc in sorted_docs {
        let filename = doc.name.trim();
        let content = doc.content.as_ref();

        if filename == "chip-types.xml" {
            data.chip_types.extend(parse_chip_types_xml(content)?);
            continue;
        }

        if filename == "matter-devices.xml" {
            data.device_types = parse_device_types(content)?;
            continue;
        }

        if filename == "global-attributes.xml" {
            let config = ConfiguratorXml::from_xml(content)?;
            data.global_defs.add_global_attributes(config);
            continue;
        }

        let config = ConfiguratorXml::from_xml(content)?;

        extract_configurator_types(
            &config,
            &mut data.all_enums,
            &mut data.all_bitmaps,
            &mut data.all_structs,
        )?;

        if matches!(
            filename,
            "global-enums.xml" | "semantic-tag-namespace-enums.xml" | "global-bitmaps.xml" | "global-structs.xml"
        ) {
            data.global_defs.add_configurator_items(config.items)?;
            continue;
        }

        if filename == "measurement-and-sensing.xml" {
            data.global_defs.add_configurator_items(config.items.clone())?;
        }

        let parsed_file = parse_csa_configurator(config)?;
        for cluster in parsed_file.clusters {
            if cluster.code > 0 && !cluster.name.is_empty() {
                data.clusters.push(cluster);
            }
        }
        data.cluster_extensions.extend(parsed_file.extensions);
        data.standalone_structs.extend(parsed_file.standalone_structs);
        data.standalone_enums.extend(parsed_file.standalone_enums);
        data.standalone_bitmaps.extend(parsed_file.standalone_bitmaps);
    }

    Ok(data)
}

fn insert_struct_def(all_structs: &mut BTreeMap<String, StructDef>, parsed: StructDef) {
    if let Some(existing) = all_structs.get(&parsed.name) {
        let existing_has_mode_tags = existing
            .fields
            .iter()
            .any(|field| field.name.eq_ignore_ascii_case("ModeTags"));
        let parsed_has_mode_tags = parsed
            .fields
            .iter()
            .any(|field| field.name.eq_ignore_ascii_case("ModeTags"));
        if !existing_has_mode_tags && parsed_has_mode_tags {
            all_structs.insert(parsed.name.clone(), parsed);
        }
    } else {
        all_structs.insert(parsed.name.clone(), parsed);
    }
}

fn extract_configurator_types(
    config: &ConfiguratorXml,
    all_enums: &mut BTreeMap<String, EnumDef>,
    all_bitmaps: &mut BTreeMap<String, BitmapDef>,
    all_structs: &mut BTreeMap<String, StructDef>,
) -> Result<()> {
    for item in &config.items {
        match item {
            ConfiguratorItem::Enum(enm) => {
                let parsed = parse_enum_def(enm)?;
                all_enums.insert(parsed.name.clone(), parsed);
            }
            ConfiguratorItem::Bitmap(bitmap) => {
                let parsed = parse_bitmap_def(bitmap);
                all_bitmaps.insert(parsed.name.clone(), parsed);
            }
            ConfiguratorItem::Struct(st_xml) => {
                let parsed = parse_struct_def(st_xml)?;
                insert_struct_def(all_structs, parsed);
            }
            ConfiguratorItem::Domain(_)
            | ConfiguratorItem::Cluster(_)
            | ConfiguratorItem::AccessControl(_)
            | ConfiguratorItem::Atomic(_)
            | ConfiguratorItem::ClusterExtension(_)
            | ConfiguratorItem::Global(_)
            | ConfiguratorItem::DeviceType(_)
            | ConfiguratorItem::Text(_) => {}
        }
    }
    Ok(())
}
