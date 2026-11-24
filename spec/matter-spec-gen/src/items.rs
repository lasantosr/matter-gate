use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use genco::lang::rust::Tokens;

use crate::{
    category::Category,
    error::Error,
    model::{
        BitmapDef, ClusterDef, EnumDef, GlobalDefs, ParsedDeviceType, ResolvedType, StructDef, resolve_type,
        sanitize_identifier,
    },
    render::{Render, clusters, context, device_types, globals},
};

/// Strongly-typed generated cluster definition file.
#[derive(Debug, Clone)]
pub struct ClusterItem {
    pub(crate) spec_version: String,
    pub(crate) exact_tag: String,
    pub(crate) category: Category,
    pub(crate) module_name: String,
    pub(crate) original_name: String,
    pub(crate) cluster_cfg: String,
    pub(crate) cluster_def: ClusterDef,
    pub(crate) shared: Arc<SharedClusterContext>,
}

impl ClusterItem {
    #[must_use]
    pub fn category(&self) -> &Category {
        &self.category
    }

    #[must_use]
    pub fn spec_version(&self) -> &str {
        &self.spec_version
    }

    #[must_use]
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.original_name
    }

    #[must_use]
    pub fn exact_tag(&self) -> &str {
        &self.exact_tag
    }

    #[must_use]
    pub fn cluster_cfg(&self) -> &str {
        &self.cluster_cfg
    }

    #[must_use]
    pub fn cluster_id(&self) -> u32 {
        self.cluster_def.code
    }

    #[must_use]
    pub fn has_attributes(&self) -> bool {
        !self.cluster_def.attributes.is_empty()
    }

    #[must_use]
    pub fn has_commands(&self) -> bool {
        !self.cluster_def.commands.is_empty()
    }

    #[must_use]
    pub fn has_client_commands(&self) -> bool {
        self.cluster_def
            .commands
            .iter()
            .any(|c| c.source.eq_ignore_ascii_case("client"))
    }

    #[must_use]
    pub fn has_server_commands(&self) -> bool {
        self.cluster_def
            .commands
            .iter()
            .any(|c| c.source.eq_ignore_ascii_case("server"))
    }

    pub(crate) fn estimated_state_layout(&self) -> TypeLayout {
        let mut visiting = BTreeSet::new();
        let fields = self
            .cluster_def
            .attributes
            .iter()
            .map(|attr| {
                let base = resolve_type(
                    &self.cluster_def.name,
                    &attr.name,
                    &attr.type_str,
                    attr.entry_type.as_deref(),
                    attr.is_array,
                    &self.cluster_def.attributes,
                    &self.cluster_def.enums,
                    &self.cluster_def.bitmaps,
                    &self.cluster_def.structs,
                    &self.shared.global_defs,
                    &self.shared.all_enums,
                    &self.shared.all_bitmaps,
                    &self.shared.all_structs,
                )
                .map_or(
                    TypeLayout {
                        size: 8,
                        align: 8,
                        niches: 0,
                    },
                    |r| resolved_type_layout(&r, &self.cluster_def, &self.shared, &mut visiting),
                );
                let mut fl = if attr.is_nullable { wrap_enum_tag(base) } else { base };
                if attr.is_optional {
                    fl = wrap_enum_tag(fl);
                }
                fl
            })
            .collect();
        struct_layout(fields)
    }

    pub(crate) fn estimated_patch_layout(&self) -> TypeLayout {
        let mut visiting = BTreeSet::new();
        let fields = self
            .cluster_def
            .attributes
            .iter()
            .map(|attr| {
                let base = resolve_type(
                    &self.cluster_def.name,
                    &attr.name,
                    &attr.type_str,
                    attr.entry_type.as_deref(),
                    attr.is_array,
                    &self.cluster_def.attributes,
                    &self.cluster_def.enums,
                    &self.cluster_def.bitmaps,
                    &self.cluster_def.structs,
                    &self.shared.global_defs,
                    &self.shared.all_enums,
                    &self.shared.all_bitmaps,
                    &self.shared.all_structs,
                )
                .map_or(
                    TypeLayout {
                        size: 8,
                        align: 8,
                        niches: 0,
                    },
                    |r| resolved_type_layout(&r, &self.cluster_def, &self.shared, &mut visiting),
                );
                let inner = if attr.is_nullable { wrap_enum_tag(base) } else { base };
                wrap_enum_tag(inner)
            })
            .collect();
        struct_layout(fields)
    }

    #[must_use]
    pub fn is_large_state(&self) -> bool {
        self.estimated_state_layout().size > 64
    }

    #[must_use]
    pub fn is_large_patch(&self) -> bool {
        self.estimated_patch_layout().size > 64
    }

    #[must_use]
    pub fn has_events(&self) -> bool {
        !self.cluster_def.events.is_empty()
    }

    #[must_use]
    pub fn cluster_dependencies(&self) -> &[String] {
        &[]
    }

    /// Renders this cluster for emission into `crate::__shared::clusters::<ident>`.
    pub fn render_shared(&self, shared_plan: &crate::SharedPlan) -> Result<String, Error> {
        let tokens = clusters::generate_cluster_tokens(
            &self.cluster_def,
            &self.shared.global_defs,
            &self.shared.all_enums,
            &self.shared.all_bitmaps,
            &self.shared.all_structs,
            &self.exact_tag,
            self.category.as_str(),
            Some(shared_plan),
        )?;
        context::render(tokens)
    }
}

impl Render for ClusterItem {
    fn render_tokens(&self) -> Result<Tokens, Error> {
        clusters::generate_cluster_tokens(
            &self.cluster_def,
            &self.shared.global_defs,
            &self.shared.all_enums,
            &self.shared.all_bitmaps,
            &self.shared.all_structs,
            &self.exact_tag,
            self.category.as_str(),
            None,
        )
    }
}

/// Strongly-typed generated individual device type definition file (e.g. `on_off_light.rs`).
#[derive(Debug, Clone)]
pub struct DeviceTypeItem {
    pub(crate) spec_version: String,
    pub(crate) category: Category,
    pub(crate) module_name: String,
    pub(crate) original_name: String,
    pub(crate) feature_name: String,
    pub(crate) device_id: u32,
    pub(crate) cluster_dependencies: BTreeSet<String>,
    pub(crate) device_type: ParsedDeviceType,
    pub(crate) shared: Arc<SharedDeviceTypeContext>,
}

impl DeviceTypeItem {
    #[must_use]
    pub fn category(&self) -> &Category {
        &self.category
    }

    #[must_use]
    pub fn spec_version(&self) -> &str {
        &self.spec_version
    }

    #[must_use]
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.original_name
    }

    #[must_use]
    pub fn feature_name(&self) -> &str {
        &self.feature_name
    }

    #[must_use]
    pub fn device_id(&self) -> u32 {
        self.device_id
    }

    #[must_use]
    pub const fn cluster_dependencies(&self) -> &BTreeSet<String> {
        &self.cluster_dependencies
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        let avail_ref: Vec<&str> = self.shared.available_clusters.iter().map(String::as_str).collect();
        device_types::is_device_type_empty(&self.device_type, &avail_ref, &self.shared.cluster_defs)
    }

    #[must_use]
    pub fn is_large(&self) -> bool {
        let mut fields = Vec::new();
        for cluster in &self.device_type.mandatory_server_clusters {
            let clean = sanitize_identifier(cluster);
            if clean != "Descriptor"
                && self.shared.available_clusters.contains(&clean)
                && let Some(&layout) = self.shared.cluster_layouts.get(&clean)
            {
                fields.push(layout);
            }
        }
        for cluster in &self.device_type.optional_server_clusters {
            let clean = sanitize_identifier(cluster);
            if clean != "Descriptor"
                && self.shared.available_clusters.contains(&clean)
                && let Some(&layout) = self.shared.cluster_layouts.get(&clean)
            {
                fields.push(wrap_enum_tag(layout));
            }
        }
        struct_layout(fields).size > 64
    }
}

impl Render for DeviceTypeItem {
    fn render_tokens(&self) -> Result<Tokens, Error> {
        let avail_ref: Vec<&str> = self.shared.available_clusters.iter().map(String::as_str).collect();
        Ok(device_types::generate_single_device_type_tokens(
            &self.device_type,
            &avail_ref,
            &self.shared.cluster_defs,
            &self.shared.eq_clusters,
            &self.spec_version,
        ))
    }
}

/// Strongly-typed generated global attributes, enums, bitmaps, and structs file.
#[derive(Debug, Clone)]
pub struct GlobalsItem {
    pub(crate) spec_version: String,
    pub(crate) global_defs: GlobalDefs,
}

impl GlobalsItem {
    #[must_use]
    pub const fn category(&self) -> Option<&Category> {
        None
    }

    #[must_use]
    pub fn spec_version(&self) -> &str {
        &self.spec_version
    }

    #[must_use]
    pub const fn module_name(&self) -> &'static str {
        "globals"
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        "globals"
    }

    #[must_use]
    pub fn cluster_dependencies(&self) -> &[String] {
        &[]
    }

    /// Renders a single shared global enum, bitmap, or struct by name.
    pub fn render_single_global(&self, name: &str, shared_plan: &crate::SharedPlan) -> Result<String, Error> {
        let tokens = globals::generate_single_global_tokens(&self.global_defs, name, shared_plan)?;
        context::render(tokens)
    }

    /// Renders this version's `globals.rs`, re-exporting items present in `shared_plan`
    /// from `crate::__shared` and emitting version-local globals inline.
    pub fn render_with_shared(&self, shared_plan: &crate::SharedPlan) -> Result<String, Error> {
        let tokens = globals::generate_globals_tokens(&self.global_defs, &self.spec_version, Some(shared_plan))?;
        context::render(tokens)
    }
}

impl Render for GlobalsItem {
    fn render_tokens(&self) -> Result<Tokens, Error> {
        globals::generate_globals_tokens(&self.global_defs, &self.spec_version, None)
    }
}

/// A polymorphic generated model item.
#[derive(Debug, Clone)]
pub enum GeneratedItem {
    Cluster(Box<ClusterItem>),
    DeviceType(Box<DeviceTypeItem>),
    Globals(GlobalsItem),
}

impl GeneratedItem {
    #[must_use]
    pub fn category(&self) -> Option<&Category> {
        match self {
            Self::Cluster(cluster) => Some(cluster.category()),
            Self::DeviceType(device_type) => Some(device_type.category()),
            Self::Globals(globals) => globals.category(),
        }
    }

    #[must_use]
    pub fn spec_version(&self) -> &str {
        match self {
            Self::Cluster(cluster) => cluster.spec_version(),
            Self::DeviceType(device_type) => device_type.spec_version(),
            Self::Globals(globals) => globals.spec_version(),
        }
    }

    #[must_use]
    pub fn module_name(&self) -> &str {
        match self {
            Self::Cluster(cluster) => cluster.module_name(),
            Self::DeviceType(device_type) => device_type.module_name(),
            Self::Globals(globals) => globals.module_name(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Cluster(cluster) => cluster.name(),
            Self::DeviceType(device_type) => device_type.name(),
            Self::Globals(globals) => globals.name(),
        }
    }

    #[must_use]
    pub fn cluster_dependencies(&self) -> Vec<String> {
        match self {
            Self::Cluster(cluster) => cluster.cluster_dependencies().to_vec(),
            Self::DeviceType(device_type) => device_type.cluster_dependencies().iter().cloned().collect(),
            Self::Globals(globals) => globals.cluster_dependencies().to_vec(),
        }
    }
}

impl Render for GeneratedItem {
    fn render_tokens(&self) -> Result<Tokens, Error> {
        match self {
            Self::Cluster(cluster) => cluster.render_tokens(),
            Self::DeviceType(device_type) => device_type.render_tokens(),
            Self::Globals(globals) => globals.render_tokens(),
        }
    }

    fn render(&self) -> Result<String, Error> {
        match self {
            Self::Cluster(cluster) => cluster.render(),
            Self::DeviceType(device_type) => device_type.render(),
            Self::Globals(globals) => globals.render(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TypeLayout {
    pub(crate) size: usize,
    pub(crate) align: usize,
    pub(crate) niches: usize,
}

impl TypeLayout {
    const ZST: Self = Self {
        size: 0,
        align: 1,
        niches: 0,
    };
}

#[derive(Debug, Clone)]
pub(crate) struct SharedClusterContext {
    pub(crate) global_defs: GlobalDefs,
    pub(crate) all_enums: BTreeMap<String, EnumDef>,
    pub(crate) all_bitmaps: BTreeMap<String, BitmapDef>,
    pub(crate) all_structs: BTreeMap<String, StructDef>,
}

#[derive(Debug, Clone)]
pub(crate) struct SharedDeviceTypeContext {
    pub(crate) available_clusters: Vec<String>,
    pub(crate) cluster_defs: Vec<ClusterDef>,
    /// Names of clusters whose generated struct derives `Eq` and `Hash`.
    pub(crate) eq_clusters: BTreeSet<String>,
    /// Estimated `(size, align, niches)` for each cluster state struct by sanitized cluster name.
    pub(crate) cluster_layouts: BTreeMap<String, TypeLayout>,
}

const fn align_up(val: usize, align: usize) -> usize {
    if align <= 1 {
        val
    } else if let Some(rem) = val.checked_rem(align)
        && rem != 0
    {
        val.saturating_add(align.saturating_sub(rem))
    } else {
        val
    }
}

fn wrap_enum_tag(inner: TypeLayout) -> TypeLayout {
    if inner.niches >= 1 {
        TypeLayout {
            size: inner.size,
            align: inner.align,
            niches: inner.niches.saturating_sub(1),
        }
    } else {
        TypeLayout {
            size: align_up(inner.size.saturating_add(1), inner.align),
            align: inner.align,
            niches: 254,
        }
    }
}

fn struct_layout(mut fields: Vec<TypeLayout>) -> TypeLayout {
    if fields.is_empty() {
        return TypeLayout::ZST;
    }
    fields.sort_by(|a, b| b.align.cmp(&a.align).then_with(|| b.size.cmp(&a.size)));
    let mut offset = 0_usize;
    let mut max_align = 1_usize;
    let mut max_niches = 0_usize;
    for f in fields {
        offset = align_up(offset, f.align);
        offset = offset.saturating_add(f.size);
        max_align = max_align.max(f.align);
        max_niches = max_niches.max(f.niches);
    }
    TypeLayout {
        size: align_up(offset, max_align),
        align: max_align,
        niches: max_niches,
    }
}

fn primitive_type_layout(rust_type: &str) -> TypeLayout {
    match rust_type {
        "bool" => TypeLayout {
            size: 1,
            align: 1,
            niches: 254,
        },
        "u8" | "i8" => TypeLayout {
            size: 1,
            align: 1,
            niches: 0,
        },
        "u16" | "i16" => TypeLayout {
            size: 2,
            align: 2,
            niches: 0,
        },
        "u32" | "i32" | "f32" => TypeLayout {
            size: 4,
            align: 4,
            niches: 0,
        },
        "u64" | "i64" | "f64" => TypeLayout {
            size: 8,
            align: 8,
            niches: 0,
        },
        "()" => TypeLayout::ZST,
        "crate::Bytes" => TypeLayout {
            size: 32,
            align: 8,
            niches: 1,
        },
        _ => TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        },
    }
}

fn id_type_layout(rust_type: &str) -> TypeLayout {
    match rust_type {
        "FabricIndex" | "FieldId" | "ActionId" => TypeLayout {
            size: 1,
            align: 1,
            niches: 0,
        },
        "GroupId" | "EndpointId" | "VendorId" => TypeLayout {
            size: 2,
            align: 2,
            niches: 0,
        },
        "NodeId" | "FabricId" | "EventNumber" => TypeLayout {
            size: 8,
            align: 8,
            niches: 0,
        },
        _ => TypeLayout {
            size: 4,
            align: 4,
            niches: 0,
        },
    }
}

fn semantic_type_layout(rust_type: &str) -> TypeLayout {
    match rust_type {
        "types::Percent" => TypeLayout {
            size: 1,
            align: 1,
            niches: 0,
        },
        "types::Percent100ths" | "types::Temperature" | "types::TemperatureDifference" | "types::SignedTemperature" => {
            TypeLayout {
                size: 2,
                align: 2,
                niches: 0,
            }
        }
        "types::Ipv4Address" => TypeLayout {
            size: 4,
            align: 1,
            niches: 0,
        },
        "types::Ipv6Address" => TypeLayout {
            size: 16,
            align: 1,
            niches: 0,
        },
        "types::IpAddress" | "types::Ipv6Prefix" | "types::HardwareAddress" => TypeLayout {
            size: 32,
            align: 8,
            niches: 1,
        },
        "types::TimeOfDay" | "types::MatterDate" | "types::EpochS" | "types::ElapsedS" => TypeLayout {
            size: 4,
            align: 4,
            niches: 0,
        },
        _ => TypeLayout {
            size: 8,
            align: 8,
            niches: 0,
        },
    }
}

fn struct_type_layout(
    name: &str,
    cluster_def: &ClusterDef,
    shared: &SharedClusterContext,
    visiting: &mut BTreeSet<String>,
) -> TypeLayout {
    if !visiting.insert(name.to_owned()) {
        return TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        };
    }
    let st_opt = cluster_def
        .structs
        .iter()
        .chain(shared.global_defs.structs.iter())
        .chain(shared.all_structs.values())
        .find(|s| s.name.eq_ignore_ascii_case(name));
    let layout = if let Some(st) = st_opt {
        let field_layouts = st
            .fields
            .iter()
            .map(|f| {
                let base = resolve_type(
                    name,
                    &f.name,
                    &f.type_str,
                    f.entry_type.as_deref(),
                    f.is_array,
                    &cluster_def.attributes,
                    &cluster_def.enums,
                    &cluster_def.bitmaps,
                    &cluster_def.structs,
                    &shared.global_defs,
                    &shared.all_enums,
                    &shared.all_bitmaps,
                    &shared.all_structs,
                )
                .map_or(
                    TypeLayout {
                        size: 8,
                        align: 8,
                        niches: 0,
                    },
                    |r| resolved_type_layout(&r, cluster_def, shared, visiting),
                );
                let mut fl = if f.is_nullable { wrap_enum_tag(base) } else { base };
                if f.is_optional {
                    fl = wrap_enum_tag(fl);
                }
                fl
            })
            .collect();
        struct_layout(field_layouts)
    } else {
        TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        }
    };
    visiting.remove(name);
    layout
}

fn resolved_type_layout(
    resolved: &ResolvedType,
    cluster_def: &ClusterDef,
    shared: &SharedClusterContext,
    visiting: &mut BTreeSet<String>,
) -> TypeLayout {
    match resolved {
        ResolvedType::Primitive { rust_type, .. } => primitive_type_layout(rust_type),
        ResolvedType::IdType { rust_type, .. } => id_type_layout(rust_type),
        ResolvedType::SemanticType { rust_type, .. } => semantic_type_layout(rust_type),
        ResolvedType::Enum { is_16bit, .. } => {
            if *is_16bit {
                TypeLayout {
                    size: 2,
                    align: 2,
                    niches: 256,
                }
            } else {
                TypeLayout {
                    size: 1,
                    align: 1,
                    niches: 16,
                }
            }
        }
        ResolvedType::Bitmap { type_str, .. } => match type_str.to_ascii_lowercase().as_str() {
            "bitmap8" | "int8u" | "u8" => TypeLayout {
                size: 1,
                align: 1,
                niches: 0,
            },
            "bitmap16" | "int16u" | "u16" => TypeLayout {
                size: 2,
                align: 2,
                niches: 0,
            },
            "bitmap64" | "int64u" | "u64" => TypeLayout {
                size: 8,
                align: 8,
                niches: 0,
            },
            _ => TypeLayout {
                size: 4,
                align: 4,
                niches: 0,
            },
        },
        ResolvedType::Array { .. } => TypeLayout {
            size: 24,
            align: 8,
            niches: 1,
        },
        ResolvedType::Struct { name, .. } => struct_type_layout(name, cluster_def, shared, visiting),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AttributeDef, FieldDef};

    #[test]
    fn byte_size_threshold_boxes_large_structs_and_keeps_compact_unboxed() {
        let shared = Arc::new(SharedClusterContext {
            global_defs: GlobalDefs::default(),
            all_enums: BTreeMap::new(),
            all_bitmaps: BTreeMap::new(),
            all_structs: BTreeMap::new(),
        });

        let compact = ClusterItem {
            spec_version: "1.6".to_owned(),
            exact_tag: "v1.6".to_owned(),
            category: Category::general(),
            module_name: "on_off".to_owned(),
            original_name: "OnOff".to_owned(),
            cluster_cfg: "cluster_on_off".to_owned(),
            cluster_def: ClusterDef {
                name: "OnOff".to_owned(),
                attributes: vec![
                    AttributeDef {
                        name: "OnOff".to_owned(),
                        type_str: "boolean".to_owned(),
                        ..Default::default()
                    },
                    AttributeDef {
                        name: "OnTime".to_owned(),
                        type_str: "int16u".to_owned(),
                        is_optional: true,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            shared: Arc::clone(&shared),
        };
        assert!(!compact.is_large_state());
        assert!(!compact.is_large_patch());

        // Fewer than 8 attributes (4 attrs), but > 64 bytes due to Vec + String + nested struct.
        let large_op_state = ClusterItem {
            spec_version: "1.6".to_owned(),
            exact_tag: "v1.6".to_owned(),
            category: Category::general(),
            module_name: "operational_state".to_owned(),
            original_name: "OperationalState".to_owned(),
            cluster_cfg: "cluster_operational_state".to_owned(),
            cluster_def: ClusterDef {
                name: "OperationalState".to_owned(),
                structs: vec![StructDef {
                    name: "ErrorStateStruct".to_owned(),
                    fields: vec![
                        FieldDef {
                            name: "ErrorStateID".to_owned(),
                            type_str: "int8u".to_owned(),
                            ..Default::default()
                        },
                        FieldDef {
                            name: "ErrorStateLabel".to_owned(),
                            type_str: "char_string".to_owned(),
                            is_optional: true,
                            ..Default::default()
                        },
                        FieldDef {
                            name: "ErrorStateDetails".to_owned(),
                            type_str: "char_string".to_owned(),
                            is_optional: true,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                attributes: vec![
                    AttributeDef {
                        name: "PhaseList".to_owned(),
                        type_str: "array".to_owned(),
                        entry_type: Some("char_string".to_owned()),
                        is_array: true,
                        is_nullable: true,
                        ..Default::default()
                    },
                    AttributeDef {
                        name: "OperationalStateList".to_owned(),
                        type_str: "array".to_owned(),
                        entry_type: Some("int8u".to_owned()),
                        is_array: true,
                        ..Default::default()
                    },
                    AttributeDef {
                        name: "OperationalError".to_owned(),
                        type_str: "ErrorStateStruct".to_owned(),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            shared,
        };
        assert!(large_op_state.is_large_state(), "expected > 64B state");
        assert!(large_op_state.is_large_patch(), "expected > 64B patch");

        // DeviceTypeItem layout threshold: 1 large cluster (> 64B) makes the endpoint large even with 1 dependency.
        let mut cluster_layouts = BTreeMap::new();
        cluster_layouts.insert("OnOff".to_owned(), compact.estimated_state_layout());
        cluster_layouts.insert("OperationalState".to_owned(), large_op_state.estimated_state_layout());

        let dt_shared = Arc::new(SharedDeviceTypeContext {
            available_clusters: vec!["OnOff".to_owned(), "OperationalState".to_owned()],
            cluster_defs: vec![],
            eq_clusters: BTreeSet::new(),
            cluster_layouts,
        });

        let small_dt = DeviceTypeItem {
            spec_version: "1.6".to_owned(),
            category: Category::general(),
            module_name: "small_dev".to_owned(),
            original_name: "SmallDev".to_owned(),
            feature_name: "endpoint-small-dev".to_owned(),
            device_id: 0x0100,
            cluster_dependencies: BTreeSet::from(["cluster_on_off".to_owned()]),
            device_type: ParsedDeviceType {
                mandatory_server_clusters: vec!["OnOff".to_owned()],
                ..Default::default()
            },
            shared: Arc::clone(&dt_shared),
        };
        assert!(!small_dt.is_large());

        let large_dt = DeviceTypeItem {
            spec_version: "1.6".to_owned(),
            category: Category::general(),
            module_name: "large_dev".to_owned(),
            original_name: "LargeDev".to_owned(),
            feature_name: "endpoint-large-dev".to_owned(),
            device_id: 0x0101,
            cluster_dependencies: BTreeSet::from(["cluster_operational_state".to_owned()]),
            device_type: ParsedDeviceType {
                mandatory_server_clusters: vec!["OperationalState".to_owned()],
                ..Default::default()
            },
            shared: dt_shared,
        };
        assert!(large_dt.is_large());
    }
}
