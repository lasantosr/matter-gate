use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    result::Result as StdResult,
    sync::Arc,
};

use crate::{
    category::Category,
    error::Error,
    items::{ClusterItem, DeviceTypeItem, GeneratedItem, GlobalsItem, SharedClusterContext, SharedDeviceTypeContext},
    model::{
        BitmapDef, ClusterDef, ComposedEndpointDef, EnumDef, GlobalDefs, ParsedDeviceType, ResolvedType, StructDef,
        merge_cluster_extensions, parse_bitmap_def, parse_chip_types_xml, parse_csa_configurator, parse_device_types,
        parse_enum_def, parse_struct_def, resolve_type, sanitize_identifier, to_kebab_case, to_snake_case,
    },
    reconcile::reconcile_categories_across_versions,
    render::{clusters::cluster_derives_eq_hash, registry::render_registry_tables},
    shared::{
        SharedBucket, SharedKind, SharedPlan, assign_bucket_idents, compute_cluster_hash, compute_global_hashes,
        format_bucket_cfg,
    },
    xml::{ClusterExtensionXml, ConfiguratorItem, ConfiguratorXml},
};

type Result<T, E = Error> = StdResult<T, E>;

/// In-memory XML document tagged by its filename or descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlDocument<D> {
    pub name: String,
    pub content: D,
}

impl<D: AsRef<str>> XmlDocument<D> {
    pub fn new(name: impl Into<String>, content: D) -> Self {
        Self {
            name: name.into(),
            content,
        }
    }
}

/// Input configuration for a single Matter specification version.
#[derive(Debug, Clone)]
pub struct SpecInput<D, I = Vec<XmlDocument<D>>> {
    pub version: String,
    pub tag: String,
    pub documents: I,
    pub(crate) _marker: PhantomData<fn() -> D>,
}

impl<D: AsRef<str>, I: IntoIterator<Item = XmlDocument<D>>> SpecInput<D, I> {
    pub fn new(version: impl Into<String>, tag: impl Into<String>, documents: I) -> Self {
        Self {
            version: version.into(),
            tag: tag.into(),
            documents,
            _marker: PhantomData,
        }
    }
}

/// Generated models for a specific Matter specification version.
#[derive(Debug, Clone)]
pub struct VersionSpec {
    pub version: String,
    pub tag: String,
    pub globals: GlobalsItem,
    pub clusters: Vec<ClusterItem>,
    pub device_types: Vec<DeviceTypeItem>,
}

impl VersionSpec {
    #[must_use]
    pub fn items(&self) -> Vec<GeneratedItem> {
        let cap = 1_usize
            .saturating_add(self.clusters.len())
            .saturating_add(self.device_types.len());
        let mut items = Vec::with_capacity(cap);
        items.push(GeneratedItem::Globals(self.globals.clone()));
        for cluster in &self.clusters {
            items.push(GeneratedItem::Cluster(Box::new(cluster.clone())));
        }
        for device_type in &self.device_types {
            items.push(GeneratedItem::DeviceType(Box::new(device_type.clone())));
        }
        items
    }

    /// Renders the `#[cfg(feature = "registry")]` static `SpecRegistry` module for this version.
    pub fn render_registry(&self) -> Result<String, Error> {
        let cluster_refs: Vec<&ClusterDef> = self.clusters.iter().map(|c| &c.cluster_def).collect();
        let dt_refs: Vec<&ParsedDeviceType> = self.device_types.iter().map(|d| &d.device_type).collect();
        if let Some(first) = self.clusters.first() {
            render_registry_tables(
                &self.version,
                &cluster_refs,
                &dt_refs,
                &self.globals.global_defs,
                &first.shared.all_enums,
                &first.shared.all_bitmaps,
                &first.shared.all_structs,
            )
        } else {
            let empty_e = BTreeMap::new();
            let empty_b = BTreeMap::new();
            let empty_s = BTreeMap::new();
            render_registry_tables(
                &self.version,
                &cluster_refs,
                &dt_refs,
                &self.globals.global_defs,
                &empty_e,
                &empty_b,
                &empty_s,
            )
        }
    }
}

/// Entire generated specification suite covering one or more Matter releases.
#[derive(Debug, Clone)]
pub struct Spec {
    pub versions: Vec<VersionSpec>,
}

impl Spec {
    pub fn items(&self) -> impl Iterator<Item = GeneratedItem> + '_ {
        self.versions.iter().flat_map(VersionSpec::items)
    }

    pub fn reconcile_categories(&mut self) {
        reconcile_categories_across_versions(&mut self.versions);
    }

    /// Compute a deterministic `__shared` deduplication plan across all spec versions.
    ///
    /// Only items shared by at least two spec versions are placed in `__shared`;
    /// single-version items are emitted directly inside their owning version module.
    pub fn shared_plan(&self) -> SharedPlan {
        let mut buckets: BTreeMap<String, SharedBucket> = BTreeMap::new();

        for ver in &self.versions {
            let global_hashes = compute_global_hashes(&ver.globals.global_defs);

            for (gname, ghash) in &global_hashes {
                let bucket = buckets.entry(ghash.clone()).or_insert_with(|| SharedBucket {
                    hash: ghash.clone(),
                    ident: String::new(),
                    kind: SharedKind::Global,
                    module_name: gname.clone(),
                    versions: BTreeSet::new(),
                    cluster_cfgs: BTreeSet::new(),
                    version_cluster_pairs: BTreeSet::new(),
                    cfg_gate: String::new(),
                });
                bucket.versions.insert(ver.version.clone());
                let gates = ver
                    .globals
                    .global_defs
                    .cfg_gates
                    .get(gname)
                    .map_or(&[][..], Vec::as_slice);
                for gate in gates {
                    bucket.cluster_cfgs.insert(gate.clone());
                    bucket.version_cluster_pairs.insert((ver.version.clone(), gate.clone()));
                }
            }

            for cluster in &ver.clusters {
                let chash = compute_cluster_hash(&cluster.cluster_def, &global_hashes);
                let bucket = buckets.entry(chash.clone()).or_insert_with(|| SharedBucket {
                    hash: chash,
                    ident: String::new(),
                    kind: SharedKind::Cluster,
                    module_name: cluster.module_name().to_owned(),
                    versions: BTreeSet::new(),
                    cluster_cfgs: BTreeSet::new(),
                    version_cluster_pairs: BTreeSet::new(),
                    cfg_gate: String::new(),
                });
                bucket.versions.insert(ver.version.clone());
                bucket.cluster_cfgs.insert(cluster.cluster_cfg().to_owned());
                bucket
                    .version_cluster_pairs
                    .insert((ver.version.clone(), cluster.module_name().to_owned()));
            }
        }

        buckets.retain(|_, bucket| bucket.versions.len() > 1);
        assign_bucket_idents(&mut buckets);

        for bucket in buckets.values_mut() {
            bucket.cfg_gate = format_bucket_cfg(bucket);
        }

        SharedPlan { buckets }
    }
}

/// High-level generation entry point: converts in-memory XML documents into typed specification models.
pub fn build<V, D, I>(inputs: V) -> Result<Spec>
where
    V: IntoIterator<Item = SpecInput<D, I>>,
    D: AsRef<str>,
    I: IntoIterator<Item = XmlDocument<D>>,
{
    let mut versions = Vec::new();
    for input in inputs {
        versions.push(build_version_spec(input)?);
    }
    if versions.len() > 1 {
        reconcile_categories_across_versions(&mut versions);
    }
    Ok(Spec { versions })
}

pub(crate) fn is_valid_domain_device(device_id: u32) -> bool {
    device_id != 0 && device_id < 0xF000
}

pub(crate) fn resolve_device_type_categories(
    device_types: &[ParsedDeviceType],
    cluster_to_category: &BTreeMap<String, Category>,
) -> Vec<Category> {
    let mut categories: Vec<Option<Category>> = device_types
        .iter()
        .map(|dt| {
            if is_valid_domain_device(dt.device_id) {
                direct_cluster_category(dt, cluster_to_category)
            } else {
                Some(Category::general())
            }
        })
        .collect();

    // Pass 2 & Pass 3: Relationship & Composition Propagation (Fixed-Point Loop)
    loop {
        let mut progress = false;
        progress |= propagate_superset_downward(device_types, &mut categories);
        progress |= propagate_superset_upward(device_types, &mut categories);
        progress |= propagate_endpoint_composition_upward(device_types, &mut categories, cluster_to_category);
        progress |= propagate_endpoint_composition_downward(device_types, &mut categories);

        if !progress {
            break;
        }
    }

    categories
        .into_iter()
        .map(|opt| opt.unwrap_or_else(Category::general))
        .collect()
}

pub(crate) fn build_device_type_items(
    device_types: &[ParsedDeviceType],
    categories: &[Category],
    version: &str,
    cluster_items: &[ClusterItem],
    shared_dt_ctx: &Arc<SharedDeviceTypeContext>,
) -> Vec<DeviceTypeItem> {
    let mut dt_items = Vec::new();
    for (dt, cat) in device_types.iter().zip(categories) {
        let mod_name = to_snake_case(&dt.name);
        let feat_name = format!("endpoint-{}", to_kebab_case(&dt.name));
        let dt_deps = resolve_single_device_cluster_deps(dt, cluster_items);

        dt_items.push(DeviceTypeItem {
            spec_version: version.to_owned(),
            category: cat.clone(),
            module_name: mod_name,
            original_name: dt.name.clone(),
            feature_name: feat_name,
            device_id: dt.device_id,
            cluster_dependencies: dt_deps,
            device_type: dt.clone(),
            shared: Arc::clone(shared_dt_ctx),
        });
    }
    dt_items
}

#[derive(Default)]
struct ParsedRawVersionData {
    chip_types: BTreeMap<String, (&'static str, &'static str)>,
    global_defs: GlobalDefs,
    device_types: Vec<ParsedDeviceType>,
    all_enums: BTreeMap<String, EnumDef>,
    all_bitmaps: BTreeMap<String, BitmapDef>,
    all_structs: BTreeMap<String, StructDef>,
    clusters: Vec<ClusterDef>,
    cluster_extensions: Vec<ClusterExtensionXml>,
    standalone_structs: Vec<(StructDef, Vec<u32>)>,
    standalone_enums: Vec<(EnumDef, Vec<u32>)>,
    standalone_bitmaps: Vec<(BitmapDef, Vec<u32>)>,
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

fn process_raw_documents<D, I>(documents: I) -> Result<ParsedRawVersionData>
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

fn record_type_ref(type_str: &str, cluster_cfg: &str, refs: &mut BTreeMap<String, BTreeSet<String>>) {
    let clean = type_str
        .strip_suffix("Struct")
        .or_else(|| type_str.strip_suffix("Enum"))
        .or_else(|| type_str.strip_suffix("Bitmap"))
        .unwrap_or(type_str);
    refs.entry(type_str.to_lowercase())
        .or_default()
        .insert(cluster_cfg.to_owned());
    refs.entry(clean.to_lowercase())
        .or_default()
        .insert(cluster_cfg.to_owned());
}

fn collect_cluster_type_refs(clusters: &[ClusterDef]) -> BTreeMap<String, BTreeSet<String>> {
    let mut refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for cluster in clusters {
        let cluster_cfg = format!("cluster_{}", to_snake_case(&cluster.name));
        for attr in &cluster.attributes {
            record_type_ref(&attr.type_str, &cluster_cfg, &mut refs);
            if let Some(et) = &attr.entry_type {
                record_type_ref(et, &cluster_cfg, &mut refs);
            }
        }
        for cmd in &cluster.commands {
            for arg in &cmd.args {
                record_type_ref(&arg.type_str, &cluster_cfg, &mut refs);
                if let Some(et) = &arg.entry_type {
                    record_type_ref(et, &cluster_cfg, &mut refs);
                }
            }
        }
        for ev in &cluster.events {
            for field in &ev.fields {
                record_type_ref(&field.type_str, &cluster_cfg, &mut refs);
                if let Some(et) = &field.entry_type {
                    record_type_ref(et, &cluster_cfg, &mut refs);
                }
            }
        }
        for st in &cluster.structs {
            record_type_ref(&st.name, &cluster_cfg, &mut refs);
            for field in &st.fields {
                record_type_ref(&field.type_str, &cluster_cfg, &mut refs);
                if let Some(et) = &field.entry_type {
                    record_type_ref(et, &cluster_cfg, &mut refs);
                }
            }
        }
        for enm in &cluster.enums {
            record_type_ref(&enm.name, &cluster_cfg, &mut refs);
        }
        for bm in &cluster.bitmaps {
            record_type_ref(&bm.name, &cluster_cfg, &mut refs);
        }
    }
    refs
}

fn struct_fields_resolve_globally(st: &StructDef, clusters: &[ClusterDef], global_defs: &GlobalDefs) -> bool {
    let empty_enums = BTreeMap::new();
    let empty_bitmaps = BTreeMap::new();
    let empty_structs = BTreeMap::new();
    let is_global_safe = |resolved: &ResolvedType| match resolved {
        ResolvedType::Enum { is_global, .. }
        | ResolvedType::Bitmap { is_global, .. }
        | ResolvedType::Struct { is_global, .. } => *is_global,
        ResolvedType::Array { element } => matches!(
            element.as_ref(),
            ResolvedType::Primitive { .. }
                | ResolvedType::IdType { .. }
                | ResolvedType::SemanticType { .. }
                | ResolvedType::Enum { is_global: true, .. }
                | ResolvedType::Bitmap { is_global: true, .. }
                | ResolvedType::Struct { is_global: true, .. }
        ),
        _ => true,
    };

    for cluster in clusters {
        if !cluster.structs.iter().any(|cs| cs.name.eq_ignore_ascii_case(&st.name)) {
            continue;
        }
        for field in &st.fields {
            let Ok(resolved) = resolve_type(
                &st.name,
                &field.name,
                &field.type_str,
                field.entry_type.as_deref(),
                field.is_array,
                &cluster.attributes,
                &cluster.enums,
                &cluster.bitmaps,
                &cluster.structs,
                global_defs,
                &empty_enums,
                &empty_bitmaps,
                &empty_structs,
            ) else {
                return false;
            };
            if !is_global_safe(&resolved) {
                return false;
            }
        }
    }
    true
}

fn promote_shared_cluster_types(clusters: &mut [ClusterDef], global_defs: &mut GlobalDefs) {
    let refs = collect_cluster_type_refs(clusters);

    // 1. Promote identical enums shared across >= 2 clusters
    let mut enum_defs: BTreeMap<String, Vec<EnumDef>> = BTreeMap::new();
    for cluster in clusters.iter() {
        for enm in &cluster.enums {
            if !global_defs.enums.iter().any(|g| g.name.eq_ignore_ascii_case(&enm.name)) {
                enum_defs.entry(enm.name.to_lowercase()).or_default().push(enm.clone());
            }
        }
    }
    let mut promoted_enums = BTreeSet::new();
    for (lower_name, defs) in enum_defs {
        let Some(first) = defs.first() else { continue };
        let Some(users) = refs.get(&lower_name) else { continue };
        if users.len() >= 2
            && defs
                .iter()
                .all(|d| d.is_16bit == first.is_16bit && d.items == first.items)
        {
            global_defs.enums.push(first.clone());
            global_defs
                .cfg_gates
                .insert(first.name.clone(), users.iter().cloned().collect());
            promoted_enums.insert(lower_name);
        }
    }
    if !promoted_enums.is_empty() {
        for cluster in clusters.iter_mut() {
            cluster
                .enums
                .retain(|e| !promoted_enums.contains(&e.name.to_lowercase()));
        }
    }

    // 2. Promote identical bitmaps shared across >= 2 clusters
    let mut bitmap_defs: BTreeMap<String, Vec<BitmapDef>> = BTreeMap::new();
    for cluster in clusters.iter() {
        for bm in &cluster.bitmaps {
            if !global_defs
                .bitmaps
                .iter()
                .any(|g| g.name.eq_ignore_ascii_case(&bm.name))
            {
                bitmap_defs.entry(bm.name.to_lowercase()).or_default().push(bm.clone());
            }
        }
    }
    let mut promoted_bitmaps = BTreeSet::new();
    for (lower_name, defs) in bitmap_defs {
        let Some(first) = defs.first() else { continue };
        let Some(users) = refs.get(&lower_name) else { continue };
        if users.len() >= 2
            && defs
                .iter()
                .all(|d| d.type_str.eq_ignore_ascii_case(&first.type_str) && d.fields == first.fields)
        {
            global_defs.bitmaps.push(first.clone());
            global_defs
                .cfg_gates
                .insert(first.name.clone(), users.iter().cloned().collect());
            promoted_bitmaps.insert(lower_name);
        }
    }
    if !promoted_bitmaps.is_empty() {
        for cluster in clusters.iter_mut() {
            cluster
                .bitmaps
                .retain(|b| !promoted_bitmaps.contains(&b.name.to_lowercase()));
        }
    }

    // 3. Promote identical structs shared across >= 2 clusters (fixed-point for nested structs)
    let mut promoted_structs = BTreeSet::new();
    loop {
        let mut progress = false;
        let mut struct_defs: BTreeMap<String, Vec<StructDef>> = BTreeMap::new();
        for cluster in clusters.iter() {
            for st in &cluster.structs {
                let lower = st.name.to_lowercase();
                if !promoted_structs.contains(&lower)
                    && !global_defs
                        .structs
                        .iter()
                        .any(|g| g.name.eq_ignore_ascii_case(&st.name))
                {
                    struct_defs.entry(lower).or_default().push(st.clone());
                }
            }
        }
        for (lower_name, defs) in struct_defs {
            let Some(first) = defs.first() else { continue };
            let Some(users) = refs.get(&lower_name) else { continue };
            if users.len() >= 2
                && defs
                    .iter()
                    .all(|d| d.is_fabric_scoped == first.is_fabric_scoped && d.fields == first.fields)
                && struct_fields_resolve_globally(first, clusters, global_defs)
            {
                global_defs.structs.push(first.clone());
                global_defs
                    .cfg_gates
                    .insert(first.name.clone(), users.iter().cloned().collect());
                promoted_structs.insert(lower_name);
                progress = true;
            }
        }
        if !progress {
            break;
        }
    }
    if !promoted_structs.is_empty() {
        for cluster in clusters.iter_mut() {
            cluster
                .structs
                .retain(|s| !promoted_structs.contains(&s.name.to_lowercase()));
        }
    }

    propagate_cfg_gates(global_defs);
}

fn merge_parent_gates_into(
    cfg_gates: &mut BTreeMap<String, Vec<String>>,
    type_name: &str,
    parent_gates: &[String],
) -> bool {
    if type_name.is_empty() {
        return false;
    }
    let Some(target_key) = cfg_gates.keys().find(|k| k.eq_ignore_ascii_case(type_name)).cloned() else {
        return false;
    };
    let Some(target_gates) = cfg_gates.get_mut(&target_key) else {
        return false;
    };
    let mut changed = false;
    for pg in parent_gates {
        if !target_gates.contains(pg) {
            target_gates.push(pg.clone());
            changed = true;
        }
    }
    if changed {
        target_gates.sort();
    }
    changed
}

fn propagate_cfg_gates(global_defs: &mut GlobalDefs) {
    loop {
        let mut gate_progress = false;
        for st in &global_defs.structs {
            let Some(parent_gates) = global_defs.cfg_gates.get(&st.name).cloned() else {
                continue;
            };
            for field in &st.fields {
                gate_progress |= merge_parent_gates_into(&mut global_defs.cfg_gates, &field.type_str, &parent_gates);
                if let Some(et) = field.entry_type.as_deref() {
                    gate_progress |= merge_parent_gates_into(&mut global_defs.cfg_gates, et, &parent_gates);
                }
            }
        }
        if !gate_progress {
            break;
        }
    }
}

fn attach_standalone_types_to_clusters(
    clusters: &mut [ClusterDef],
    standalone_structs: &[(StructDef, Vec<u32>)],
    standalone_enums: &[(EnumDef, Vec<u32>)],
    standalone_bitmaps: &[(BitmapDef, Vec<u32>)],
) {
    for cluster in clusters {
        for (st_def, codes) in standalone_structs {
            if codes.contains(&cluster.code)
                && !cluster
                    .structs
                    .iter()
                    .any(|strct| strct.name.eq_ignore_ascii_case(&st_def.name))
            {
                cluster.structs.push(st_def.clone());
            }
        }
        for (enm_def, codes) in standalone_enums {
            if codes.contains(&cluster.code)
                && !cluster
                    .enums
                    .iter()
                    .any(|enm| enm.name.eq_ignore_ascii_case(&enm_def.name))
            {
                cluster.enums.push(enm_def.clone());
            }
        }
        for (bm_def, codes) in standalone_bitmaps {
            if codes.contains(&cluster.code)
                && !cluster
                    .bitmaps
                    .iter()
                    .any(|bitmap| bitmap.name.eq_ignore_ascii_case(&bm_def.name))
            {
                cluster.bitmaps.push(bm_def.clone());
            }
        }
    }
}

fn is_testing_cluster(name: &str) -> bool {
    matches!(
        name,
        "Unit Testing"
            | "UnitTesting"
            | "Test Hidden Manufacturer Specific"
            | "TestHiddenManufacturerSpecific"
            | "Fault Injection"
            | "FaultInjection"
            | "Sample MEI"
            | "SampleMei"
    )
}

fn build_cluster_items(
    clusters: Vec<ClusterDef>,
    version: &str,
    tag: &str,
    shared_cluster_ctx: &Arc<SharedClusterContext>,
) -> Result<(Vec<ClusterItem>, Vec<String>), Error> {
    let mut cluster_items = Vec::with_capacity(clusters.len());
    let mut cluster_names = Vec::with_capacity(clusters.len());

    for cluster in clusters {
        let category = if is_testing_cluster(&cluster.name) {
            Category::testing()
        } else {
            Category::from_xml_domain(&cluster.domain)?
        };
        let snake_name = to_snake_case(&cluster.name);
        let cluster_cfg = format!("cluster_{snake_name}");

        cluster_names.push(cluster.name.clone());
        cluster_items.push(ClusterItem {
            spec_version: version.to_owned(),
            exact_tag: tag.to_owned(),
            category,
            module_name: snake_name,
            original_name: cluster.name.clone(),
            cluster_cfg,
            cluster_def: cluster,
            shared: Arc::clone(shared_cluster_ctx),
        });
    }

    Ok((cluster_items, cluster_names))
}

fn resolve_single_device_cluster_deps(dt: &ParsedDeviceType, cluster_items: &[ClusterItem]) -> BTreeSet<String> {
    let mut dt_deps = BTreeSet::new();
    for cluster_name in dt
        .mandatory_server_clusters
        .iter()
        .chain(dt.optional_server_clusters.iter())
        .chain(dt.mandatory_client_clusters.iter())
        .chain(dt.optional_client_clusters.iter())
    {
        let clean = sanitize_identifier(cluster_name);
        if let Some(matching_cluster) = cluster_items.iter().find(|cl| cl.name().eq_ignore_ascii_case(&clean)) {
            dt_deps.insert(matching_cluster.cluster_cfg().to_owned());
        }
    }
    dt_deps
}

fn find_device_type_index(
    device_types: &[ParsedDeviceType],
    device_id: Option<u32>,
    name: Option<&str>,
) -> Option<usize> {
    if let Some(target_id) = device_id {
        device_types.iter().position(|dev| dev.device_id == target_id)
    } else if let Some(target_name) = name {
        device_types.iter().position(|dev| {
            dev.name.eq_ignore_ascii_case(target_name) || dev.type_name.eq_ignore_ascii_case(target_name)
        })
    } else {
        None
    }
}

fn direct_cluster_category(
    dt: &ParsedDeviceType,
    cluster_to_category: &BTreeMap<String, Category>,
) -> Option<Category> {
    if !is_valid_domain_device(dt.device_id) {
        return None;
    }

    struct CategoryStats {
        category: Category,
        highest_role_priority: u8,
        count_at_highest_priority: usize,
        total_count: usize,
        first_appearance_index: usize,
    }

    let mut stats_map: BTreeMap<String, CategoryStats> = BTreeMap::new();

    for (idx, req) in dt.cluster_requirements.iter().enumerate() {
        let cat = if req.has_lighting_feature && req.role_priority >= 2 {
            Category::from_xml_domain("Lighting").ok()
        } else {
            cluster_to_category
                .get(&req.name)
                .filter(|category| category.as_str() != "general")
                .cloned()
        };

        let Some(cat) = cat else {
            continue;
        };

        let entry = stats_map
            .entry(cat.as_str().to_owned())
            .or_insert_with(|| CategoryStats {
                category: cat.clone(),
                highest_role_priority: req.role_priority,
                count_at_highest_priority: 0,
                total_count: 0,
                first_appearance_index: idx,
            });

        entry.total_count = entry.total_count.saturating_add(1);

        match req.role_priority.cmp(&entry.highest_role_priority) {
            Ordering::Greater => {
                entry.highest_role_priority = req.role_priority;
                entry.count_at_highest_priority = 1;
            }
            Ordering::Equal => {
                entry.count_at_highest_priority = entry.count_at_highest_priority.saturating_add(1);
            }
            Ordering::Less => {}
        }
    }

    stats_map
        .into_values()
        .max_by_key(|stats| {
            (
                stats.highest_role_priority,
                stats.count_at_highest_priority,
                stats.total_count,
                Reverse(stats.first_appearance_index),
            )
        })
        .map(|stats| stats.category)
}

fn propagate_superset_downward(device_types: &[ParsedDeviceType], categories: &mut [Option<Category>]) -> bool {
    let mut progress = false;
    for (idx, dt) in device_types.iter().enumerate() {
        if categories.get(idx).and_then(Option::as_ref).is_some() || !is_valid_domain_device(dt.device_id) {
            continue;
        }
        let Some(sup_name) = &dt.superset else {
            continue;
        };
        let Some(parent_idx) = find_device_type_index(device_types, None, Some(sup_name)) else {
            continue;
        };
        let Some(parent) = device_types.get(parent_idx) else {
            continue;
        };
        if !is_valid_domain_device(parent.device_id) {
            continue;
        }
        let Some(parent_cat) = categories
            .get(parent_idx)
            .and_then(Option::as_ref)
            .filter(|cat| cat.as_str() != "general")
            .cloned()
        else {
            continue;
        };
        if let Some(slot) = categories.get_mut(idx) {
            *slot = Some(parent_cat);
            progress = true;
        }
    }
    progress
}

fn propagate_superset_upward(device_types: &[ParsedDeviceType], categories: &mut [Option<Category>]) -> bool {
    let mut progress = false;
    for (idx, dt) in device_types.iter().enumerate() {
        if categories.get(idx).and_then(Option::as_ref).is_some() || !is_valid_domain_device(dt.device_id) {
            continue;
        }

        let parent_has_no_resolved_superset = match &dt.superset {
            None => true,
            Some(sup_name) => match find_device_type_index(device_types, None, Some(sup_name)) {
                None => true,
                Some(parent_idx) => categories
                    .get(parent_idx)
                    .and_then(Option::as_ref)
                    .is_none_or(|cat| cat.as_str() == "general"),
            },
        };

        if !parent_has_no_resolved_superset {
            continue;
        }

        let resolved_children_cats: Vec<Category> = device_types
            .iter()
            .enumerate()
            .filter(|(_, child)| is_valid_domain_device(child.device_id))
            .filter(|(_, child)| {
                child
                    .superset
                    .as_deref()
                    .is_some_and(|sup| sup.eq_ignore_ascii_case(&dt.name) || sup.eq_ignore_ascii_case(&dt.type_name))
            })
            .filter_map(|(c_idx, _)| categories.get(c_idx).and_then(Option::as_ref))
            .filter(|cat| cat.as_str() != "general")
            .cloned()
            .collect();

        if let Some(first_cat) = resolved_children_cats.first()
            && resolved_children_cats.iter().all(|child_cat| child_cat == first_cat)
            && let Some(slot) = categories.get_mut(idx)
        {
            *slot = Some(first_cat.clone());
            progress = true;
        }
    }
    progress
}

fn compute_endpoint_composition_votes(
    composed_endpoints: &[ComposedEndpointDef],
    device_types: &[ParsedDeviceType],
    categories: &[Option<Category>],
    cluster_to_category: &BTreeMap<String, Category>,
) -> Vec<(Category, usize, usize)> {
    let mut votes: Vec<(Category, usize, usize)> = Vec::new();
    for (ep_idx, comp) in composed_endpoints.iter().enumerate() {
        let mut comp_cluster_cats = Vec::new();
        for cluster_name in &comp.clusters {
            if let Some(cat) = cluster_to_category
                .get(cluster_name)
                .filter(|cat| cat.as_str() != "general")
            {
                comp_cluster_cats.push(cat.clone());
            }
        }
        if !comp_cluster_cats.is_empty() {
            let mut cat_counts: BTreeMap<Category, usize> = BTreeMap::new();
            for cluster_cat in &comp_cluster_cats {
                let entry = cat_counts.entry(cluster_cat.clone()).or_default();
                *entry = entry.saturating_add(1);
            }
            if let Some((best_cat, _)) = cat_counts.into_iter().max_by_key(|(_, count)| *count) {
                votes.push((best_cat, comp_cluster_cats.len(), ep_idx));
                continue;
            }
        }

        let referenced_dt = find_device_type_index(device_types, comp.device_type_id, comp.device_type_name.as_deref());

        let Some(r_idx) = referenced_dt else {
            continue;
        };
        let Some(r_dt) = device_types.get(r_idx) else {
            continue;
        };
        if !is_valid_domain_device(r_dt.device_id) {
            continue;
        }
        let Some(r_cat) = categories
            .get(r_idx)
            .and_then(Option::as_ref)
            .filter(|cat| cat.as_str() != "general")
        else {
            continue;
        };

        let non_gen_count = r_dt
            .mandatory_server_clusters
            .iter()
            .chain(&r_dt.optional_server_clusters)
            .filter(|cluster_name| {
                cluster_to_category
                    .get(&sanitize_identifier(cluster_name))
                    .is_some_and(|cat| cat.as_str() != "general")
            })
            .count();
        let weight = 1_usize.saturating_add(non_gen_count);
        votes.push((r_cat.clone(), weight, ep_idx));
    }
    votes
}

fn propagate_endpoint_composition_upward(
    device_types: &[ParsedDeviceType],
    categories: &mut [Option<Category>],
    cluster_to_category: &BTreeMap<String, Category>,
) -> bool {
    let mut progress = false;
    for (idx, dt) in device_types.iter().enumerate() {
        if categories.get(idx).and_then(Option::as_ref).is_some()
            || !is_valid_domain_device(dt.device_id)
            || dt.composed_endpoints.is_empty()
        {
            continue;
        }

        let votes =
            compute_endpoint_composition_votes(&dt.composed_endpoints, device_types, categories, cluster_to_category);

        if votes.is_empty() {
            continue;
        }

        struct EpCatStats {
            total_weight: usize,
            endpoint_count: usize,
            first_appearance_index: usize,
        }
        let mut agg: BTreeMap<Category, EpCatStats> = BTreeMap::new();
        for (cat, weight, ep_idx) in votes {
            agg.entry(cat)
                .and_modify(|stats| {
                    stats.total_weight = stats.total_weight.saturating_add(weight);
                    stats.endpoint_count = stats.endpoint_count.saturating_add(1);
                })
                .or_insert(EpCatStats {
                    total_weight: weight,
                    endpoint_count: 1,
                    first_appearance_index: ep_idx,
                });
        }

        if let Some((best_cat, _)) = agg.into_iter().max_by_key(|(_, stats)| {
            (
                stats.total_weight,
                stats.endpoint_count,
                Reverse(stats.first_appearance_index),
            )
        }) && let Some(slot) = categories.get_mut(idx)
        {
            *slot = Some(best_cat);
            progress = true;
        }
    }
    progress
}

fn propagate_endpoint_composition_downward(
    device_types: &[ParsedDeviceType],
    categories: &mut [Option<Category>],
) -> bool {
    let mut progress = false;
    for (idx, parent) in device_types.iter().enumerate() {
        if !is_valid_domain_device(parent.device_id) {
            continue;
        }
        let Some(parent_cat) = categories
            .get(idx)
            .and_then(Option::as_ref)
            .filter(|cat| cat.as_str() != "general")
            .cloned()
        else {
            continue;
        };

        for comp in &parent.composed_endpoints {
            let child_idx = find_device_type_index(device_types, comp.device_type_id, comp.device_type_name.as_deref());

            let Some(c_idx) = child_idx else {
                continue;
            };
            let Some(child) = device_types.get(c_idx) else {
                continue;
            };
            if !is_valid_domain_device(child.device_id) {
                continue;
            }
            let child_is_unresolved = categories
                .get(c_idx)
                .and_then(Option::as_ref)
                .is_none_or(|cat| cat.as_str() == "general");
            if child_is_unresolved && let Some(slot) = categories.get_mut(c_idx) {
                *slot = Some(parent_cat.clone());
                progress = true;
            }
        }
    }
    progress
}

fn build_device_type_items_for_version(
    device_types: Vec<ParsedDeviceType>,
    version: &str,
    cluster_items: &[ClusterItem],
    cluster_names: Vec<String>,
) -> Vec<DeviceTypeItem> {
    let mut cluster_to_category: BTreeMap<String, Category> = BTreeMap::new();
    for cl in cluster_items {
        cluster_to_category.insert(cl.name().to_owned(), cl.category().clone());
    }

    let categories = resolve_device_type_categories(&device_types, &cluster_to_category);

    let all_cluster_defs: Vec<ClusterDef> = cluster_items
        .iter()
        .map(|cluster_item| cluster_item.cluster_def.clone())
        .collect();
    let eq_clusters: BTreeSet<String> = cluster_items
        .iter()
        .filter(|cluster_item| {
            cluster_derives_eq_hash(
                &cluster_item.cluster_def,
                &cluster_item.shared.global_defs,
                &cluster_item.shared.all_enums,
                &cluster_item.shared.all_bitmaps,
                &cluster_item.shared.all_structs,
            )
        })
        .map(|cluster_item| cluster_item.name().to_owned())
        .collect();
    let cluster_layouts = cluster_items
        .iter()
        .map(|cluster_item| (cluster_item.name().to_owned(), cluster_item.estimated_state_layout()))
        .collect();
    let shared_dt_ctx = Arc::new(SharedDeviceTypeContext {
        available_clusters: cluster_names,
        cluster_defs: all_cluster_defs,
        eq_clusters,
        cluster_layouts,
    });

    build_device_type_items(&device_types, &categories, version, cluster_items, &shared_dt_ctx)
}

fn build_version_spec<D, I>(input: SpecInput<D, I>) -> Result<VersionSpec>
where
    D: AsRef<str>,
    I: IntoIterator<Item = XmlDocument<D>>,
{
    let version = input.version;
    let tag = input.tag;

    let mut data = process_raw_documents(input.documents)?;

    // Attach global definitions to all_types tables
    for enm in &data.global_defs.enums {
        data.all_enums.insert(enm.name.clone(), enm.clone());
    }
    for bitmap in &data.global_defs.bitmaps {
        data.all_bitmaps.insert(bitmap.name.clone(), bitmap.clone());
    }
    for st_def in &data.global_defs.structs {
        data.all_structs.insert(st_def.name.clone(), st_def.clone());
    }

    attach_standalone_types_to_clusters(
        &mut data.clusters,
        &data.standalone_structs,
        &data.standalone_enums,
        &data.standalone_bitmaps,
    );

    merge_cluster_extensions(&mut data.clusters, data.cluster_extensions)?;
    promote_shared_cluster_types(&mut data.clusters, &mut data.global_defs);
    data.clusters.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    let shared_cluster_ctx = Arc::new(SharedClusterContext {
        global_defs: data.global_defs.clone(),
        all_enums: data.all_enums,
        all_bitmaps: data.all_bitmaps,
        all_structs: data.all_structs,
    });

    let (cluster_items, cluster_names) = build_cluster_items(data.clusters, &version, &tag, &shared_cluster_ctx)?;

    let device_type_items =
        build_device_type_items_for_version(data.device_types, &version, &cluster_items, cluster_names);

    let globals_item = GlobalsItem {
        spec_version: version.clone(),
        global_defs: data.global_defs,
    };

    Ok(VersionSpec {
        version,
        tag,
        globals: globals_item,
        clusters: cluster_items,
        device_types: device_type_items,
    })
}
