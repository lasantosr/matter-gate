mod categories;
mod parse;
mod types;

use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    result::Result as StdResult,
    sync::Arc,
};

pub(crate) use categories::is_valid_domain_device;
#[cfg(test)]
pub(crate) use categories::resolve_device_type_categories;

use self::{
    categories::build_device_type_items_for_version,
    parse::process_raw_documents,
    types::{attach_standalone_types_to_clusters, promote_shared_cluster_types},
};
use crate::{
    category::Category,
    error::Error,
    items::{ClusterItem, DeviceTypeItem, GeneratedItem, GlobalsItem, SharedClusterContext},
    model::{ClusterDef, ParsedDeviceType, merge_cluster_extensions, to_snake_case},
    reconcile::reconcile_categories_across_versions,
    render::registry::render_registry_tables,
    shared::{
        SharedBucket, SharedKind, SharedPlan, assign_bucket_idents, compute_cluster_hash, compute_global_hashes,
        format_bucket_cfg,
    },
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
