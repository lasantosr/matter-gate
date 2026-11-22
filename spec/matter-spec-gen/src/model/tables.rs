use std::{
    collections::{BTreeMap, BTreeSet},
    result::Result as StdResult,
};

use super::parser::{AttributeDef, BitmapDef, EnumDef, GlobalDefs, StructDef, sanitize_identifier};
use crate::{
    error::Error,
    xml::{AtomicItem, AtomicTypeXml, AtomicXml, ConfiguratorItem, ConfiguratorXml},
};

/// Explicit table mapping attribute names to domain-specific boolean constant pairs (`false_label`, `true_label`).
pub(crate) const BOOLEAN_SEMANTICS_TABLE: &[(&str, (&str, &str))] = &[
    ("OnOff", ("OFF", "ON")),
    ("Occupied", ("UNOCCUPIED", "OCCUPIED")),
    ("Occupancy", ("UNOCCUPIED", "OCCUPIED")),
    ("Locked", ("UNLOCKED", "LOCKED")),
    ("LockState", ("UNLOCKED", "LOCKED")),
    ("Closed", ("OPEN", "CLOSED")),
    ("Open", ("CLOSED", "OPEN")),
    ("Active", ("INACTIVE", "ACTIVE")),
    ("Enabled", ("DISABLED", "ENABLED")),
    ("Present", ("ABSENT", "PRESENT")),
    ("Connected", ("DISCONNECTED", "CONNECTED")),
    ("Auto", ("MANUAL", "AUTO")),
    ("Running", ("STOPPED", "RUNNING")),
    ("Installed", ("UNINSTALLED", "INSTALLED")),
    ("Ready", ("NOT_READY", "READY")),
    ("Supported", ("UNSUPPORTED", "SUPPORTED")),
    ("Muted", ("UNMUTED", "MUTED")),
    ("Reachable", ("UNREACHABLE", "REACHABLE")),
    ("SceneValid", ("INVALID", "VALID")),
    ("FullDuplex", ("HALF_DUPLEX", "FULL_DUPLEX")),
    ("Optimal", ("SUBOPTIMAL", "OPTIMAL")),
    ("Sleep", ("AWAKE", "SLEEP")),
    ("Continuous", ("INTERMITTENT", "CONTINUOUS")),
    ("FlipFlop", ("LOW", "HIGH")),
    ("Movable", ("FIXED", "MOVABLE")),
    ("Unmounted", ("MOUNTED", "UNMOUNTED")),
    ("UpdatePossible", ("NOT_POSSIBLE", "POSSIBLE")),
    ("HardPrivacyModeOn", ("OFF", "ON")),
    ("ESACanGenerate", ("CANNOT_GENERATE", "CAN_GENERATE")),
    ("CarrierDetect", ("UNDETECTED", "DETECTED")),
    ("BatReplacementNeeded", ("NOT_NEEDED", "NEEDED")),
    ("BatFunctionalWhileCharging", ("NON_FUNCTIONAL", "FUNCTIONAL")),
    ("HardwareFaultAlert", ("NORMAL", "FAULT")),
    ("RequirePINforRemoteOperation", ("NOT_REQUIRED", "REQUIRED")),
    ("TCAcknowledgementsRequired", ("NOT_REQUIRED", "REQUIRED")),
    ("ObjectCountThresholdReached", ("NOT_REACHED", "REACHED")),
    ("ObjectIdentified", ("UNIDENTIFIED", "IDENTIFIED")),
    ("PresetsSchedulesEditable", ("READ_ONLY", "EDITABLE")),
    ("BlockUnrated", ("ALLOWED", "BLOCKED")),
    ("ImageFlipHorizontal", ("NORMAL", "FLIPPED")),
    ("ImageFlipVertical", ("NORMAL", "FLIPPED")),
];

pub(crate) const GENERIC_BOOLEAN_SEMANTICS: [(&str, bool); 2] = [("TRUE", true), ("FALSE", false)];

type Result<T, E = Error> = StdResult<T, E>;

pub(crate) fn is_semantic_boolean_attr(attr_name: &str) -> Option<(&'static str, &'static str)> {
    for (key, (false_label, true_label)) in BOOLEAN_SEMANTICS_TABLE {
        if key.eq_ignore_ascii_case(attr_name) {
            return Some((*false_label, *true_label));
        }
    }

    let lower = attr_name.to_lowercase();
    if lower.ends_with("enabled") || lower.ends_with("disabled") {
        return Some(("DISABLED", "ENABLED"));
    }
    if lower.ends_with("muted") {
        return Some(("UNMUTED", "MUTED"));
    }
    if lower.ends_with("present") {
        return Some(("ABSENT", "PRESENT"));
    }
    if lower.ends_with("detected") {
        return Some(("UNDETECTED", "DETECTED"));
    }
    if lower.ends_with("available") {
        return Some(("UNAVAILABLE", "AVAILABLE"));
    }
    if lower.starts_with("supports") || lower.ends_with("supported") {
        return Some(("UNSUPPORTED", "SUPPORTED"));
    }

    None
}

/// Lookup semantic boolean constants for an attribute name.
pub(crate) fn lookup_boolean_semantics(attr_name: &str) -> [(&'static str, bool); 2] {
    match is_semantic_boolean_attr(attr_name) {
        Some((false_label, true_label)) => [(true_label, true), (false_label, false)],
        None => GENERIC_BOOLEAN_SEMANTICS,
    }
}

pub(crate) fn to_upper_camel_case(input: &str) -> String {
    let mut result = String::new();
    for word in input.split('_') {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            result.push_str(&first.to_uppercase().to_string());
            for ch in chars {
                result.push_str(&ch.to_lowercase().to_string());
            }
        }
    }
    result
}

/// Parse `chip-types.xml` content and verify all atomic and derived types.
pub(crate) fn parse_chip_types_xml(content: &str) -> Result<BTreeMap<String, (&'static str, &'static str)>> {
    let ConfiguratorXml {
        xmlns_xsi: _xmlns_xsi,
        schema_location: _schema_location,
        no_namespace_schema_location: _no_namespace_schema_location,
        items,
    } = ConfiguratorXml::from_xml(content)?;
    let mut types_map = BTreeMap::new();

    for item in items {
        if let ConfiguratorItem::Atomic(AtomicXml { items: atomic_items }) = item {
            for a_item in atomic_items {
                parse_atomic_item(a_item, &mut types_map)?;
            }
        }
    }

    Ok(types_map)
}

/// Resolved Matter type mapping to Rust type system representations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedType {
    Primitive {
        rust_type: &'static str,
        attr_variant: &'static str,
        is_copy: bool,
    },
    IdType {
        rust_type: &'static str,
        attr_variant: &'static str,
    },
    SemanticType {
        rust_type: &'static str,
        attr_variant: &'static str,
        is_copy: bool,
    },
    Enum {
        name: String,
        is_16bit: bool,
        is_global: bool,
    },
    Bitmap {
        name: String,
        type_str: String,
        is_global: bool,
    },
    Struct {
        name: String,
        is_global: bool,
    },
    Array {
        element: Box<ResolvedType>,
    },
}

impl ResolvedType {
    pub(crate) fn rust_type_path(&self, in_submodule: bool) -> String {
        match self {
            Self::Primitive {
                rust_type,
                attr_variant: _attr_variant,
                is_copy: _is_copy,
            } => (*rust_type).to_owned(),
            Self::IdType {
                rust_type,
                attr_variant: _attr_variant,
            } => (*rust_type).to_owned(),
            Self::SemanticType {
                rust_type,
                attr_variant: _attr_variant,
                is_copy: _is_copy,
            } => (*rust_type).to_owned(),
            Self::Enum {
                name,
                is_16bit: _is_16bit,
                is_global,
            } => {
                if *is_global {
                    if in_submodule {
                        format!("super::globals::{name}")
                    } else {
                        format!("globals::{name}")
                    }
                } else if in_submodule {
                    format!("super::{name}")
                } else {
                    name.clone()
                }
            }
            Self::Bitmap {
                name,
                type_str: _type_str,
                is_global,
            } => {
                if *is_global {
                    if in_submodule {
                        format!("super::globals::{name}")
                    } else {
                        format!("globals::{name}")
                    }
                } else if in_submodule {
                    format!("super::{name}")
                } else {
                    name.clone()
                }
            }
            Self::Struct { name, is_global } => {
                if *is_global {
                    if in_submodule {
                        format!("super::globals::{name}")
                    } else {
                        format!("globals::{name}")
                    }
                } else if in_submodule {
                    format!("super::{name}")
                } else {
                    name.clone()
                }
            }
            Self::Array { element } => {
                format!("Vec<{}>", element.rust_type_path(in_submodule))
            }
        }
    }

    pub(crate) fn is_copy(&self) -> bool {
        match self {
            Self::Primitive {
                rust_type: _rust_type,
                attr_variant: _attr_variant,
                is_copy,
            } => *is_copy,
            Self::IdType {
                rust_type: _rust_type,
                attr_variant: _attr_variant,
            } => true,
            Self::SemanticType {
                rust_type: _rust_type,
                attr_variant: _attr_variant,
                is_copy,
            } => *is_copy,
            Self::Enum {
                name: _name,
                is_16bit: _is_16bit,
                is_global: _is_global,
            } => true,
            Self::Bitmap {
                name: _name,
                type_str: _type_str,
                is_global: _is_global,
            } => true,
            Self::Struct {
                name: _name,
                is_global: _is_global,
            } => false,
            Self::Array { element: _element } => false,
        }
    }

    pub(crate) fn can_derive_eq_hash(&self, eq_structs: Option<&BTreeSet<String>>) -> bool {
        match self {
            Self::Primitive {
                rust_type,
                attr_variant: _attr_variant,
                is_copy: _is_copy,
            } => *rust_type != "f32" && *rust_type != "f64" && *rust_type != "AttributeValue",
            Self::IdType {
                rust_type: _rust_type,
                attr_variant: _attr_variant,
            } => true,
            Self::SemanticType {
                rust_type: _rust_type,
                attr_variant: _attr_variant,
                is_copy: _is_copy,
            } => true,
            Self::Enum {
                name: _name,
                is_16bit: _is_16bit,
                is_global: _is_global,
            } => true,
            Self::Bitmap {
                name: _name,
                type_str: _type_str,
                is_global: _is_global,
            } => true,
            Self::Struct {
                name,
                is_global: _is_global,
            } => eq_structs.is_some_and(|set| set.contains(name)),
            Self::Array { element } => element.can_derive_eq_hash(eq_structs),
        }
    }
}

pub(crate) fn resolve_scalar_type(
    cluster_name: &str,
    context_name: &str,
    type_str: &str,
    cluster_attrs: &[AttributeDef],
    cluster_enums: &[EnumDef],
    cluster_bitmaps: &[BitmapDef],
    cluster_structs: &[StructDef],
    global_defs: &GlobalDefs,
    all_enums: &BTreeMap<String, EnumDef>,
    all_bitmaps: &BTreeMap<String, BitmapDef>,
    all_structs: &BTreeMap<String, StructDef>,
) -> Result<ResolvedType> {
    if let Some(resolved) = resolve_context_aware_enum(
        cluster_name,
        context_name,
        type_str,
        cluster_attrs,
        cluster_enums,
        all_enums,
    ) {
        return Ok(resolved);
    }

    if let Some(resolved) = resolve_atomic_or_id_type(type_str) {
        return Ok(resolved);
    }

    let clean_enum = type_str.strip_suffix("Enum").unwrap_or(type_str);
    let clean_bitmap = type_str.strip_suffix("Bitmap").unwrap_or(type_str);
    let clean_struct = type_str.strip_suffix("Struct").unwrap_or(type_str);

    if let Some(resolved) = resolve_defined_model_type(
        type_str,
        clean_enum,
        clean_bitmap,
        clean_struct,
        cluster_enums,
        cluster_bitmaps,
        cluster_structs,
        global_defs,
        all_enums,
        all_bitmaps,
        all_structs,
    ) {
        return Ok(resolved);
    }

    Err(Error::UnknownWireType {
        type_str: type_str.to_owned(),
        context_name: context_name.to_owned(),
        cluster_name: cluster_name.to_owned(),
    })
}

pub(crate) fn resolve_type(
    cluster_name: &str,
    context_name: &str,
    type_str: &str,
    entry_type: Option<&str>,
    is_array: bool,
    cluster_attrs: &[AttributeDef],
    cluster_enums: &[EnumDef],
    cluster_bitmaps: &[BitmapDef],
    cluster_structs: &[StructDef],
    global_defs: &GlobalDefs,
    all_enums: &BTreeMap<String, EnumDef>,
    all_bitmaps: &BTreeMap<String, BitmapDef>,
    all_structs: &BTreeMap<String, StructDef>,
) -> Result<ResolvedType> {
    if is_array || type_str.eq_ignore_ascii_case("array") {
        let elem_type_str = entry_type.unwrap_or(type_str);
        if elem_type_str.eq_ignore_ascii_case("array") {
            return Err(Error::MissingArrayEntryType {
                context_name: context_name.to_owned(),
                cluster_name: cluster_name.to_owned(),
            });
        }
        let elem_resolved = resolve_scalar_type(
            cluster_name,
            context_name,
            elem_type_str,
            cluster_attrs,
            cluster_enums,
            cluster_bitmaps,
            cluster_structs,
            global_defs,
            all_enums,
            all_bitmaps,
            all_structs,
        )?;
        Ok(ResolvedType::Array {
            element: Box::new(elem_resolved),
        })
    } else {
        resolve_scalar_type(
            cluster_name,
            context_name,
            type_str,
            cluster_attrs,
            cluster_enums,
            cluster_bitmaps,
            cluster_structs,
            global_defs,
            all_enums,
            all_bitmaps,
            all_structs,
        )
    }
}

/// Curated mapping of cluster names to product families.
/// Static map of all Matter atomic and derived types to their Rust representations and `AttributeValue` variants.
static CHIP_TYPES_MAP: &[(&str, (&str, &str))] = &[
    ("boolean", ("bool", "Bool")),
    ("bool", ("bool", "Bool")),
    ("bitmap8", ("u8", "U8")),
    ("bitmap16", ("u16", "U16")),
    ("bitmap24", ("u32", "U32")),
    ("bitmap32", ("u32", "U32")),
    ("bitmap64", ("u64", "U64")),
    ("int8u", ("u8", "U8")),
    ("u8", ("u8", "U8")),
    ("int16u", ("u16", "U16")),
    ("u16", ("u16", "U16")),
    ("int24u", ("u32", "U32")),
    ("int32u", ("u32", "U32")),
    ("u32", ("u32", "U32")),
    ("int40u", ("u64", "U64")),
    ("int48u", ("u64", "U64")),
    ("int56u", ("u64", "U64")),
    ("int64u", ("u64", "U64")),
    ("u64", ("u64", "U64")),
    ("int8s", ("i8", "I8")),
    ("int8", ("i8", "I8")),
    ("i8", ("i8", "I8")),
    ("int16s", ("i16", "I16")),
    ("int16", ("i16", "I16")),
    ("i16", ("i16", "I16")),
    ("int24s", ("i32", "I32")),
    ("int32s", ("i32", "I32")),
    ("int32", ("i32", "I32")),
    ("i32", ("i32", "I32")),
    ("int40s", ("i64", "I64")),
    ("int48s", ("i64", "I64")),
    ("int56s", ("i64", "I64")),
    ("int64s", ("i64", "I64")),
    ("int64", ("i64", "I64")),
    ("i64", ("i64", "I64")),
    ("single", ("f32", "F32")),
    ("float", ("f32", "F32")),
    ("double", ("f64", "F64")),
    ("octet_string", ("crate::Bytes", "Bytes")),
    ("long_octet_string", ("crate::Bytes", "Bytes")),
    ("char_string", ("String", "String")),
    ("long_char_string", ("String", "String")),
    ("power_mva", ("types::PowerMva", "I64")),
    ("power_mvar", ("types::PowerMvar", "I64")),
    ("power_mw", ("types::PowerMw", "I64")),
    ("amperage_ma", ("types::AmperageMa", "I64")),
    ("voltage_mv", ("types::VoltageMv", "I64")),
    ("energy_mwh", ("types::EnergyMwh", "I64")),
    ("energy_mvah", ("types::EnergyMvah", "I64")),
    ("energy_mvarh", ("types::EnergyMvarh", "I64")),
    ("percent", ("types::Percent", "U8")),
    ("percent100ths", ("types::Percent100ths", "U16")),
    ("tod", ("types::TimeOfDay", "U32")),
    ("date", ("types::MatterDate", "U32")),
    ("epoch_us", ("types::EpochUs", "U64")),
    ("epoch_s", ("types::EpochS", "U32")),
    ("posix_ms", ("types::PosixMs", "U64")),
    ("systime_us", ("types::SystimeUs", "U64")),
    ("systime_ms", ("types::SystimeMs", "U64")),
    ("elapsed_s", ("types::ElapsedS", "U32")),
    ("temperature", ("types::Temperature", "I16")),
    ("money", ("i64", "I64")),
    ("enum8", ("u8", "U8")),
    ("enum16", ("u16", "U16")),
    ("priority", ("u8", "U8")),
    ("status", ("u8", "U8")),
    ("group_id", ("GroupId", "U16")),
    ("endpoint_no", ("EndpointId", "U16")),
    ("vendor_id", ("VendorId", "U16")),
    ("devtype_id", ("DeviceTypeId", "U32")),
    ("fabric_id", ("FabricId", "U64")),
    ("fabric_idx", ("FabricIndex", "U8")),
    ("cluster_id", ("ClusterId", "U32")),
    ("attrib_id", ("AttributeId", "U32")),
    ("field_id", ("FieldId", "U8")),
    ("event_id", ("EventId", "U32")),
    ("command_id", ("CommandId", "U32")),
    ("action_id", ("ActionId", "U8")),
    ("trans_id", ("TransactionId", "U32")),
    ("node_id", ("NodeId", "U64")),
    ("entry_idx", ("u16", "U16")),
    ("data_ver", ("DataVersion", "U32")),
    ("event_no", ("EventNumber", "U64")),
    ("ipadr", ("types::IpAddress", "Bytes")),
    ("ipv4adr", ("types::Ipv4Address", "Bytes")),
    ("ipv6adr", ("types::Ipv6Address", "Bytes")),
    ("ipv6pre", ("types::Ipv6Prefix", "Bytes")),
    ("hwadr", ("types::HardwareAddress", "Bytes")),
    ("semtag", ("u32", "U32")),
    ("namespace", ("u8", "U8")),
    ("tag", ("u8", "U8")),
    ("no_data", ("()", "Null")),
    ("unknown", ("()", "Null")),
    ("array", ("Vec<AttributeValue>", "List")),
    ("struct", ("Vec<(FieldId, AttributeValue)>", "Struct")),
    ("ThermostatControlSequence", ("u8", "U8")),
    ("setpointadjustmode", ("u8", "U8")),
    ("dayofweek", ("u8", "U8")),
    ("modeforsequence", ("u8", "U8")),
];

fn parse_atomic_item(a_item: AtomicItem, types_map: &mut BTreeMap<String, (&'static str, &'static str)>) -> Result<()> {
    let AtomicItem::Type(AtomicTypeXml {
        id: _id,
        name,
        description: _description,
        size: _size,
        discrete: _discrete,
        analog: _analog,
        signed: _signed,
        composite: _composite,
    }) = a_item
    else {
        return Ok(());
    };
    let mapped = CHIP_TYPES_MAP
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(&name))
        .map(|(_, val)| *val);
    let Some(type_pair) = mapped else {
        return Err(Error::UnhandledAtomicType(name));
    };
    types_map.insert(name, type_pair);
    Ok(())
}

fn resolve_context_aware_enum(
    cluster_name: &str,
    context_name: &str,
    type_str: &str,
    cluster_attrs: &[AttributeDef],
    cluster_enums: &[EnumDef],
    all_enums: &BTreeMap<String, EnumDef>,
) -> Option<ResolvedType> {
    // Check if this context is a semantic boolean attribute in a cluster
    if (type_str.eq_ignore_ascii_case("boolean") || type_str.eq_ignore_ascii_case("bool"))
        && cluster_attrs
            .iter()
            .any(|attr| attr.name.eq_ignore_ascii_case(context_name))
        && is_semantic_boolean_attr(context_name).is_some()
    {
        return Some(ResolvedType::Enum {
            name: sanitize_identifier(context_name),
            is_16bit: false,
            is_global: false,
        });
    }

    // Context-aware enum resolutions for closed-set enum8 / enum16
    if (context_name.eq_ignore_ascii_case("OperationalStateID")
        || (context_name.eq_ignore_ascii_case("OperationalState")
            && cluster_name.to_lowercase().contains("operationalstate")))
        && (type_str.eq_ignore_ascii_case("enum8") || type_str.eq_ignore_ascii_case("int8u"))
        && let Some(enm) = cluster_enums
            .iter()
            .find(|entry| entry.name == "OperationalStateEnum")
            .or_else(|| all_enums.values().find(|entry| entry.name == "OperationalStateEnum"))
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: false,
        });
    }

    if (context_name.eq_ignore_ascii_case("ErrorStateID") || context_name.eq_ignore_ascii_case("CompletionErrorCode"))
        && (type_str.eq_ignore_ascii_case("enum8") || type_str.eq_ignore_ascii_case("int8u"))
        && let Some(enm) = cluster_enums
            .iter()
            .find(|entry| entry.name == "ErrorStateEnum")
            .or_else(|| all_enums.values().find(|entry| entry.name == "ErrorStateEnum"))
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: false,
        });
    }

    if context_name.eq_ignore_ascii_case("Status")
        && (type_str.eq_ignore_ascii_case("enum8") || type_str.eq_ignore_ascii_case("int8u"))
        && let Some(enm) = cluster_enums.iter().find(|entry| entry.name == "StatusCode")
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: false,
        });
    }

    if context_name.eq_ignore_ascii_case("Value")
        && (type_str.eq_ignore_ascii_case("enum16") || type_str.eq_ignore_ascii_case("int16u"))
        && let Some(enm) = cluster_enums.iter().find(|entry| entry.name == "ModeTag")
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: false,
        });
    }

    None
}

fn resolve_atomic_or_id_type(type_str: &str) -> Option<ResolvedType> {
    let (_, (rust_type, variant)) = CHIP_TYPES_MAP
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(type_str))?;
    let is_id = matches!(
        *rust_type,
        "GroupId"
            | "EndpointId"
            | "VendorId"
            | "DeviceTypeId"
            | "FabricId"
            | "FabricIndex"
            | "ClusterId"
            | "AttributeId"
            | "FieldId"
            | "EventId"
            | "CommandId"
            | "ActionId"
            | "TransactionId"
            | "NodeId"
            | "DataVersion"
            | "EventNumber"
    );
    if is_id {
        return Some(ResolvedType::IdType {
            rust_type,
            attr_variant: variant,
        });
    }
    if rust_type.starts_with("types::") {
        let is_copy = !matches!(
            *rust_type,
            "types::IpAddress" | "types::Ipv6Prefix" | "types::HardwareAddress"
        );
        return Some(ResolvedType::SemanticType {
            rust_type,
            attr_variant: variant,
            is_copy,
        });
    }
    let is_copy = matches!(
        *rust_type,
        "bool" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64"
    );
    Some(ResolvedType::Primitive {
        rust_type,
        attr_variant: variant,
        is_copy,
    })
}

fn resolve_defined_model_type(
    type_str: &str,
    clean_enum: &str,
    clean_bitmap: &str,
    clean_struct: &str,
    cluster_enums: &[EnumDef],
    cluster_bitmaps: &[BitmapDef],
    cluster_structs: &[StructDef],
    global_defs: &GlobalDefs,
    all_enums: &BTreeMap<String, EnumDef>,
    all_bitmaps: &BTreeMap<String, BitmapDef>,
    all_structs: &BTreeMap<String, StructDef>,
) -> Option<ResolvedType> {
    // 2. Check cluster enums
    if let Some(enm) = cluster_enums
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(type_str) || entry.name.eq_ignore_ascii_case(clean_enum))
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: false,
        });
    }

    // 3. Check cluster bitmaps
    if let Some(bm) = cluster_bitmaps
        .iter()
        .find(|bitmap| bitmap.name.eq_ignore_ascii_case(type_str) || bitmap.name.eq_ignore_ascii_case(clean_bitmap))
    {
        return Some(ResolvedType::Bitmap {
            name: bm.name.clone(),
            type_str: bm.type_str.clone(),
            is_global: false,
        });
    }

    // 4. Check cluster structs
    if let Some(st) = cluster_structs
        .iter()
        .find(|strct| strct.name.eq_ignore_ascii_case(type_str) || strct.name.eq_ignore_ascii_case(clean_struct))
    {
        return Some(ResolvedType::Struct {
            name: st.name.clone(),
            is_global: false,
        });
    }

    // 5. Check global enums
    if let Some(enm) = global_defs
        .enums
        .iter()
        .find(|enm_def| enm_def.name.eq_ignore_ascii_case(type_str) || enm_def.name.eq_ignore_ascii_case(clean_enum))
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: true,
        });
    }

    // 6. Check global bitmaps
    if let Some(bm) = global_defs
        .bitmaps
        .iter()
        .find(|bitmap| bitmap.name.eq_ignore_ascii_case(type_str) || bitmap.name.eq_ignore_ascii_case(clean_bitmap))
    {
        return Some(ResolvedType::Bitmap {
            name: bm.name.clone(),
            type_str: bm.type_str.clone(),
            is_global: true,
        });
    }

    // 7. Check global structs
    if let Some(st) = global_defs
        .structs
        .iter()
        .find(|strct| strct.name.eq_ignore_ascii_case(type_str) || strct.name.eq_ignore_ascii_case(clean_struct))
    {
        return Some(ResolvedType::Struct {
            name: st.name.clone(),
            is_global: true,
        });
    }

    // 8. Check all_enums (borrowed cross-cluster enums)
    if let Some(enm) = all_enums
        .values()
        .find(|enm_def| enm_def.name.eq_ignore_ascii_case(type_str) || enm_def.name.eq_ignore_ascii_case(clean_enum))
    {
        return Some(ResolvedType::Enum {
            name: enm.name.clone(),
            is_16bit: enm.is_16bit,
            is_global: false,
        });
    }

    // 9. Check all_bitmaps (borrowed cross-cluster bitmaps)
    if let Some(bm) = all_bitmaps
        .values()
        .find(|bitmap| bitmap.name.eq_ignore_ascii_case(type_str) || bitmap.name.eq_ignore_ascii_case(clean_bitmap))
    {
        return Some(ResolvedType::Bitmap {
            name: bm.name.clone(),
            type_str: bm.type_str.clone(),
            is_global: false,
        });
    }

    // 10. Check all_structs (borrowed cross-cluster structs like ModeOptionStruct, LabelStruct, etc.)
    if let Some(st) = all_structs
        .values()
        .find(|strct| strct.name.eq_ignore_ascii_case(type_str) || strct.name.eq_ignore_ascii_case(clean_struct))
    {
        return Some(ResolvedType::Struct {
            name: st.name.clone(),
            is_global: false,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chip_types_xml_and_percent100ths_mapping() {
        let sample_xml = r#"<configurator>
            <atomic>
                <type id="0x00" name="boolean" description="Boolean" size="1" discrete="true"/>
                <type id="0x21" name="percent100ths" description="Percent 100ths" size="2" analog="true"/>
                <type id="0x28" name="power_mw" description="Power in mW" size="8" signed="true"/>
                <type id="0x29" name="power_mva" description="Power in mVA" size="8" signed="true"/>
                <type id="0x2A" name="energy_mwh" description="Energy in mWh" size="8" signed="true"/>
                <type id="0xFB" name="node_id" description="Node ID" size="8"/>
                <type id="0xFC" name="cluster_id" description="Cluster ID" size="4"/>
                <type id="0xFD" name="endpoint_no" description="Endpoint ID" size="2"/>
                <type id="0xFE" name="fabric_idx" description="Fabric Index" size="1"/>
            </atomic>
        </configurator>"#;

        let types = parse_chip_types_xml(sample_xml).expect("parse chip-types");

        // Assert percent100ths is accurately mapped to types::Percent100ths / U16 (size 2)
        let percent100ths = types.get("percent100ths").expect("percent100ths type");
        assert_eq!(*percent100ths, ("types::Percent100ths", "U16"));

        // Assert energy/power types are mapped to semantic newtypes
        assert_eq!(*types.get("power_mw").unwrap(), ("types::PowerMw", "I64"));
        assert_eq!(*types.get("power_mva").unwrap(), ("types::PowerMva", "I64"));
        assert_eq!(*types.get("energy_mwh").unwrap(), ("types::EnergyMwh", "I64"));

        // Assert Matter ID types are mapped correctly
        assert_eq!(*types.get("node_id").unwrap(), ("NodeId", "U64"));
        assert_eq!(*types.get("cluster_id").unwrap(), ("ClusterId", "U32"));
        assert_eq!(*types.get("endpoint_no").unwrap(), ("EndpointId", "U16"));
        assert_eq!(*types.get("fabric_idx").unwrap(), ("FabricIndex", "U8"));
    }

    #[test]
    fn strict_type_mapping_fail_fast() {
        let enums = Vec::new();
        let bitmaps = Vec::new();
        let structs = Vec::new();
        let globals = GlobalDefs::default();
        let all_enums = BTreeMap::new();
        let all_bitmaps = BTreeMap::new();
        let all_structs = BTreeMap::new();

        let err = resolve_scalar_type(
            "TestCluster",
            "BadAttr",
            "CompletelyBogusType",
            &[],
            &enums,
            &bitmaps,
            &structs,
            &globals,
            &all_enums,
            &all_bitmaps,
            &all_structs,
        );
        let err_msg = err.unwrap_err().to_string();
        assert!(err_msg.contains("Unknown Matter wire type 'CompletelyBogusType'"));
    }

    #[test]
    fn boolean_semantics_closed_set_and_generic_fallback() {
        assert_eq!(is_semantic_boolean_attr("OnOff"), Some(("OFF", "ON")));
        let onoff_semantics = lookup_boolean_semantics("OnOff");
        assert_eq!(onoff_semantics, [("ON", true), ("OFF", false)]);

        assert_eq!(
            is_semantic_boolean_attr("FeatureEnabled"),
            Some(("DISABLED", "ENABLED"))
        );
        let enabled_semantics = lookup_boolean_semantics("FeatureEnabled");
        assert_eq!(enabled_semantics, [("ENABLED", true), ("DISABLED", false)]);

        assert_eq!(is_semantic_boolean_attr("Muted"), Some(("UNMUTED", "MUTED")));
        let muted_semantics = lookup_boolean_semantics("Muted");
        assert_eq!(muted_semantics, [("MUTED", true), ("UNMUTED", false)]);

        assert_eq!(is_semantic_boolean_attr("UserPresent"), Some(("ABSENT", "PRESENT")));
        let present_semantics = lookup_boolean_semantics("UserPresent");
        assert_eq!(present_semantics, [("PRESENT", true), ("ABSENT", false)]);

        assert_eq!(
            is_semantic_boolean_attr("MotionDetected"),
            Some(("UNDETECTED", "DETECTED"))
        );
        let motion_semantics = lookup_boolean_semantics("MotionDetected");
        assert_eq!(motion_semantics, [("DETECTED", true), ("UNDETECTED", false)]);

        assert_eq!(
            is_semantic_boolean_attr("ServiceAvailable"),
            Some(("UNAVAILABLE", "AVAILABLE"))
        );
        let available_semantics = lookup_boolean_semantics("ServiceAvailable");
        assert_eq!(available_semantics, [("AVAILABLE", true), ("UNAVAILABLE", false)]);

        assert_eq!(
            is_semantic_boolean_attr("SupportsFeature"),
            Some(("UNSUPPORTED", "SUPPORTED"))
        );
        let supports_semantics = lookup_boolean_semantics("SupportsFeature");
        assert_eq!(supports_semantics, [("SUPPORTED", true), ("UNSUPPORTED", false)]);

        // Unmapped boolean attribute defaults to GENERIC_BOOLEAN_SEMANTICS without error
        assert_eq!(is_semantic_boolean_attr("TotallyUnmappedFlag"), None);
        let unmapped_semantics = lookup_boolean_semantics("TotallyUnmappedFlag");
        assert_eq!(unmapped_semantics, [("TRUE", true), ("FALSE", false)]);
    }
}
