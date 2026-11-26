//! Domain category inference and fixed-point propagation across device type supersets and compositions.

use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::{
    category::Category,
    items::{ClusterItem, DeviceTypeItem, SharedDeviceTypeContext},
    model::{ClusterDef, ComposedEndpointDef, ParsedDeviceType, sanitize_identifier, to_kebab_case, to_snake_case},
    render::clusters::cluster_derives_eq_hash,
};

/// Returns `true` if `device_id` denotes a real domain device (excluding 0 and pseudo-device ranges `>= 0xF000`).
pub(crate) fn is_valid_domain_device(device_id: u32) -> bool {
    device_id != 0 && device_id < 0xF000
}

/// Resolves product domain categories for all device types via direct cluster scoring and fixed-point propagation.
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

fn build_device_type_items(
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

/// Builds [`DeviceTypeItem`] render units with resolved cluster dependencies and layout/trait metadata.
pub(super) fn build_device_type_items_for_version(
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
