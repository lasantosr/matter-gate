mod layout;

#[cfg(test)]
mod tests;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use genco::lang::rust::Tokens;

pub(crate) use self::layout::TypeLayout;
use self::layout::{resolved_type_layout, struct_layout, wrap_enum_tag};
use crate::{
    category::Category,
    error::Error,
    model::{
        BitmapDef, ClusterDef, EnumDef, GlobalDefs, ParsedDeviceType, StructDef, resolve_type, sanitize_identifier,
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
