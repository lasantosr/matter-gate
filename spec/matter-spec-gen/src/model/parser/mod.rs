use std::{collections::BTreeMap, result::Result as StdResult};

use crate::{error::Error, xml::*};

pub(super) type Result<T, E = Error> = StdResult<T, E>;

mod cluster;
pub(crate) mod conformance;
mod device_types;
mod identifiers;
#[cfg(test)]
pub(crate) mod tests;
mod types;

#[cfg(test)]
pub(crate) use cluster::parse_csa_clusters;
pub(crate) use cluster::{merge_cluster_extensions, parse_csa_configurator};
pub(crate) use conformance::*;
pub(crate) use device_types::parse_device_types;
pub(crate) use identifiers::{
    ParsedBound, format_dec_with_underscores, parse_bound_for_type, parse_hex_or_dec, sanitize_identifier,
    to_kebab_case, to_shouty_snake_case, to_snake_case,
};
pub(crate) use types::{parse_attribute_xml, parse_bitmap_def, parse_enum_def, parse_struct_def};
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FeatureDef {
    pub(crate) bit: u32,
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BitmapDef {
    pub(crate) name: String,
    pub(crate) type_str: String,
    pub(crate) fields: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct EnumDef {
    pub(crate) name: String,
    pub(crate) is_16bit: bool,
    pub(crate) items: Vec<(String, u32)>,
}

/// Common shape shared by struct fields, command arguments, and event fields.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FieldDef {
    pub(crate) field_id: u32,
    pub(crate) name: String,
    pub(crate) type_str: String,
    pub(crate) entry_type: Option<String>,
    pub(crate) is_array: bool,
    pub(crate) is_nullable: bool,
    pub(crate) is_optional: bool,
    pub(crate) min: Option<String>,
    pub(crate) max: Option<String>,
    pub(crate) length: Option<u32>,
    pub(crate) min_length: Option<u32>,
    pub(crate) list_length: Option<ListLength>,
    pub(crate) conformance: Vec<ConformanceOp>,
    pub(crate) choice: Option<String>,
}

pub(crate) type StructFieldDef = FieldDef;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StructDef {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) is_fabric_scoped: bool,
    pub(crate) fields: Vec<StructFieldDef>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AttributeDef {
    pub(crate) name: String,
    pub(crate) code: u32,
    pub(crate) type_str: String,
    pub(crate) entry_type: Option<String>,
    pub(crate) is_array: bool,
    pub(crate) is_nullable: bool,
    pub(crate) is_writable: bool,
    pub(crate) is_optional: bool,
    pub(crate) min: Option<String>,
    pub(crate) max: Option<String>,
    pub(crate) length: Option<u32>,
    pub(crate) min_length: Option<u32>,
    pub(crate) list_length: Option<ListLength>,
    pub(crate) default: Option<String>,
    pub(crate) define: Option<String>,
    pub(crate) access: AccessFlags,
    pub(crate) quality: QualityFlags,
    pub(crate) conformance: Vec<ConformanceOp>,
    pub(crate) choice: Option<String>,
    pub(crate) description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CommandDef {
    pub(crate) name: String,
    pub(crate) code: u32,
    pub(crate) source: String,
    pub(crate) direction: CommandDirection,
    pub(crate) response: Option<String>,
    pub(crate) args: Vec<FieldDef>,
    pub(crate) access: AccessFlags,
    pub(crate) quality: QualityFlags,
    pub(crate) conformance: Vec<ConformanceOp>,
    pub(crate) choice: Option<String>,
    pub(crate) description: String,
}

pub(crate) type CommandArgDef = FieldDef;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct EventDef {
    pub(crate) name: String,
    pub(crate) code: u32,
    pub(crate) priority: String,
    pub(crate) fields: Vec<FieldDef>,
    pub(crate) access: AccessFlags,
    pub(crate) conformance: Vec<ConformanceOp>,
    pub(crate) choice: Option<String>,
    pub(crate) description: String,
}

pub(crate) type EventFieldDef = FieldDef;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ClusterDef {
    pub(crate) name: String,
    pub(crate) code: u32,
    pub(crate) cluster_revision: u16,
    pub(crate) domain: String,
    pub(crate) description: String,
    pub(crate) features: Vec<FeatureDef>,
    pub(crate) bitmaps: Vec<BitmapDef>,
    pub(crate) enums: Vec<EnumDef>,
    pub(crate) structs: Vec<StructDef>,
    pub(crate) attributes: Vec<AttributeDef>,
    pub(crate) commands: Vec<CommandDef>,
    pub(crate) events: Vec<EventDef>,
    pub(crate) choices: Vec<ChoiceGroup>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GlobalDefs {
    pub(crate) enums: Vec<EnumDef>,
    pub(crate) bitmaps: Vec<BitmapDef>,
    pub(crate) structs: Vec<StructDef>,
    pub(crate) attributes: Vec<AttributeDef>,
    /// Whether `EventList` (`0xFFFA`) is present in this spec version (`v1.1`..=`v1.3` only).
    pub(crate) has_event_list: bool,
    /// Optional `cluster_*` cfg gates for auto-promoted shared cross-cluster types.
    pub(crate) cfg_gates: BTreeMap<String, Vec<String>>,
}

impl GlobalDefs {
    pub(crate) fn add_configurator_items(&mut self, items: Vec<ConfiguratorItem>) -> Result<()> {
        for item in items {
            match item {
                ConfiguratorItem::Enum(enm) => {
                    let parsed = parse_enum_def(&enm)?;
                    if !self.enums.iter().any(|enum_def| enum_def.name == parsed.name) {
                        self.enums.push(parsed);
                    }
                }
                ConfiguratorItem::Bitmap(bm) => {
                    let parsed = parse_bitmap_def(&bm);
                    if !self.bitmaps.iter().any(|bitmap| bitmap.name == parsed.name) {
                        self.bitmaps.push(parsed);
                    }
                }
                ConfiguratorItem::Struct(st) => {
                    let parsed = parse_struct_def(&st)?;
                    if !self.structs.iter().any(|struct_def| struct_def.name == parsed.name) {
                        self.structs.push(parsed);
                    }
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

    pub(crate) fn add_global_attributes(&mut self, config: ConfiguratorXml) {
        for item in config.items {
            let ConfiguratorItem::Global(global) = item else {
                continue;
            };
            for g_item in global.items {
                let GlobalItem::Attribute(attr_box) = g_item else {
                    continue;
                };
                if let Some(attr_def) = parse_attribute_xml(*attr_box) {
                    self.attributes.push(attr_def);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeviceTypeClusterRequirement {
    pub(crate) name: String,
    pub(crate) role_priority: u8,
    pub(crate) has_lighting_feature: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComposedEndpointDef {
    pub(crate) device_type_id: Option<u32>,
    pub(crate) device_type_name: Option<String>,
    pub(crate) clusters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedDeviceTypeInclude {
    pub(crate) cluster: String,
    pub(crate) is_server: bool,
    pub(crate) required_attributes: Vec<String>,
    pub(crate) required_commands: Vec<String>,
    pub(crate) required_events: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ParsedDeviceType {
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) device_id: u32,
    pub(crate) revision: u16,
    pub(crate) domain: String,
    pub(crate) class: String,
    pub(crate) scope: String,
    pub(crate) superset: Option<String>,
    pub(crate) mandatory_server_clusters: Vec<String>,
    pub(crate) optional_server_clusters: Vec<String>,
    pub(crate) mandatory_client_clusters: Vec<String>,
    pub(crate) optional_client_clusters: Vec<String>,
    pub(crate) disallowed_clusters: Vec<String>,
    pub(crate) has_lighting_feature: bool,
    pub(crate) cluster_requirements: Vec<DeviceTypeClusterRequirement>,
    pub(crate) composed_endpoints: Vec<ComposedEndpointDef>,
    pub(crate) includes: Vec<ParsedDeviceTypeInclude>,
    pub(crate) choices: Vec<ChoiceGroup>,
}
