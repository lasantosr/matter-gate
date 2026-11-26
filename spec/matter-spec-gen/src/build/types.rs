//! Cross-cluster type attachment, global promotion, and feature-gate propagation.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{BitmapDef, ClusterDef, EnumDef, GlobalDefs, ResolvedType, StructDef, resolve_type, to_snake_case};

/// Promotes identical enums, bitmaps, and structs used by 2+ clusters into `global_defs`
/// with unioned `#[cfg(any(...))]` feature gates.
pub(super) fn promote_shared_cluster_types(clusters: &mut [ClusterDef], global_defs: &mut GlobalDefs) {
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

/// Attaches standalone XML types referencing specific cluster codes onto their owning clusters.
pub(super) fn attach_standalone_types_to_clusters(
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
