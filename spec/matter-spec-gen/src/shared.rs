use std::collections::{BTreeMap, BTreeSet};

use heck::ToSnakeCase as _;

use crate::model::{
    AccessFlags, AccessPrivilege, AttributeDef, BitmapDef, ChoiceGroup, ClusterDef, CommandDef, CommandDirection,
    ConformanceOp, EnumDef, EventDef, FeatureDef, FieldDef, GlobalDefs, ListLength, StructDef,
};

/// Kind of shared item in `crate::__shared::<hash>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SharedKind {
    Cluster,
    Global,
}

/// A deduplicated `__shared::{clusters,globals}::<ident>` bucket with its exact `#[cfg(...)]` gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedBucket {
    pub hash: String,
    pub ident: String,
    pub kind: SharedKind,
    pub module_name: String,
    pub versions: BTreeSet<String>,
    pub cluster_cfgs: BTreeSet<String>,
    pub version_cluster_pairs: BTreeSet<(String, String)>,
    pub cfg_gate: String,
}

/// Complete `__shared` deduplication plan across all spec versions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SharedPlan {
    pub buckets: BTreeMap<String, SharedBucket>,
}

impl SharedPlan {
    /// Returns the module identifier in `crate::__shared::globals` for a shared global hash, if present.
    #[must_use]
    pub fn global_ident(&self, hash: &str) -> Option<&str> {
        self.buckets
            .get(hash)
            .filter(|b| b.kind == SharedKind::Global)
            .map(|b| b.ident.as_str())
    }
}

/// Assign deterministic, human-readable Rust module identifiers to all shared buckets.
///
/// Uses `<base>` when only one shared bucket exists for `(kind, base)`, and
/// `<base>_<v_min>_<v_max>` when multiple distinct shared variants exist (falling back
/// to `<base>_<v_min>_<v_max>_<short_hash>` if two variants share the same min/max span).
pub(crate) fn assign_bucket_idents(buckets: &mut BTreeMap<String, SharedBucket>) {
    let mut groups: BTreeMap<(SharedKind, String), Vec<String>> = BTreeMap::new();
    for (hash, bucket) in buckets.iter() {
        let base = match bucket.kind {
            SharedKind::Cluster => bucket.module_name.clone(),
            SharedKind::Global => bucket.module_name.to_snake_case(),
        };
        groups.entry((bucket.kind, base)).or_default().push(hash.clone());
    }

    for ((_, base), hashes) in groups {
        if let [only_hash] = hashes.as_slice() {
            if let Some(bucket) = buckets.get_mut(only_hash) {
                bucket.ident = base;
            }
            continue;
        }

        // Multiple shared variants for the same name: disambiguate with _<v_min>_<v_max>.
        let mut candidate_counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut candidates: Vec<(String, String)> = Vec::with_capacity(hashes.len());
        for hash in &hashes {
            if let Some(bucket) = buckets.get(hash) {
                let v_min = bucket
                    .versions
                    .first()
                    .map(|v| format_version_slug(v))
                    .unwrap_or_default();
                let v_max = bucket
                    .versions
                    .last()
                    .map(|v| format_version_slug(v))
                    .unwrap_or_default();
                let cand = format!("{base}_{v_min}_{v_max}");
                let count = candidate_counts.entry(cand.clone()).or_insert(0);
                *count = count.saturating_add(1);
                candidates.push((hash.clone(), cand));
            }
        }

        for (hash, cand) in candidates {
            if let Some(bucket) = buckets.get_mut(&hash) {
                if candidate_counts.get(&cand).copied().unwrap_or(0) > 1 {
                    let short = hash.strip_prefix("h_").unwrap_or(&hash);
                    let short = &short[..short.len().min(8)];
                    bucket.ident = format!("{cand}_{short}");
                } else {
                    bucket.ident = cand;
                }
            }
        }
    }
}

/// Compute transitive content hashes for all globals in a version (`name -> hash`).
pub(crate) fn compute_global_hashes(globals: &GlobalDefs) -> BTreeMap<String, String> {
    let mut base_hashes = BTreeMap::new();
    for e in &globals.enums {
        let mut h = DeterministicHasher::new();
        h.write_str("global_enum");
        hash_enum(&mut h, e);
        base_hashes.insert(e.name.clone(), h.finish_hex());
    }
    for b in &globals.bitmaps {
        let mut h = DeterministicHasher::new();
        h.write_str("global_bitmap");
        hash_bitmap(&mut h, b);
        base_hashes.insert(b.name.clone(), h.finish_hex());
    }

    let struct_map: BTreeMap<&str, &StructDef> = globals.structs.iter().map(|s| (s.name.as_str(), s)).collect();
    let mut out = base_hashes.clone();

    fn resolve_struct_hash<'a>(
        name: &'a str,
        struct_map: &BTreeMap<&'a str, &'a StructDef>,
        out: &mut BTreeMap<String, String>,
        visiting: &mut BTreeSet<&'a str>,
    ) -> Option<String> {
        if let Some(existing) = out.get(name) {
            return Some(existing.clone());
        }
        let s = struct_map.get(name).copied()?;
        if !visiting.insert(name) {
            // Cycle guard: hash struct definition alone without recursing.
            let mut h = DeterministicHasher::new();
            h.write_str("global_struct");
            hash_struct(&mut h, s);
            return Some(h.finish_hex());
        }

        let mut refs = BTreeSet::new();
        for f in &s.fields {
            if f.type_str != name {
                refs.insert(f.type_str.as_str());
            }
            if let Some(et) = f.entry_type.as_deref()
                && et != name
            {
                refs.insert(et);
            }
        }

        let mut dep_hashes = Vec::new();
        for r in refs {
            if let Some(dep_hash) = resolve_struct_hash(r, struct_map, out, visiting) {
                dep_hashes.push((r, dep_hash));
            }
        }
        visiting.remove(name);

        let mut h = DeterministicHasher::new();
        h.write_str("global_struct");
        hash_struct(&mut h, s);
        h.write_usize(dep_hashes.len());
        for (dep_name, dep_hash) in dep_hashes {
            h.write_str(dep_name);
            h.write_str(&dep_hash);
        }
        let final_hash = h.finish_hex();
        out.insert(name.to_owned(), final_hash.clone());
        Some(final_hash)
    }

    let mut visiting = BTreeSet::new();
    for s in &globals.structs {
        drop(resolve_struct_hash(&s.name, &struct_map, &mut out, &mut visiting));
    }

    out
}

/// Compute the complete, behaviorally-faithful content hash for a `ClusterDef`,
/// folding in hashes of all referenced global types.
pub(crate) fn compute_cluster_hash(cluster: &ClusterDef, global_hashes: &BTreeMap<String, String>) -> String {
    let mut h = DeterministicHasher::new();
    h.write_str("cluster");
    h.write_str(&cluster.name);
    h.write_u32(cluster.code);
    h.write_u16(cluster.cluster_revision);
    h.write_str(&cluster.domain);

    h.write_usize(cluster.features.len());
    for f in &cluster.features {
        hash_feature(&mut h, f);
    }
    h.write_usize(cluster.bitmaps.len());
    for b in &cluster.bitmaps {
        hash_bitmap(&mut h, b);
    }
    h.write_usize(cluster.enums.len());
    for e in &cluster.enums {
        hash_enum(&mut h, e);
    }
    h.write_usize(cluster.structs.len());
    for s in &cluster.structs {
        hash_struct(&mut h, s);
    }
    h.write_usize(cluster.attributes.len());
    for a in &cluster.attributes {
        hash_attribute(&mut h, a);
    }
    h.write_usize(cluster.commands.len());
    for c in &cluster.commands {
        hash_command(&mut h, c);
    }
    h.write_usize(cluster.events.len());
    for e in &cluster.events {
        hash_event(&mut h, e);
    }
    h.write_usize(cluster.choices.len());
    for c in &cluster.choices {
        hash_choice(&mut h, c);
    }

    // Fold referenced global hashes in deterministic sorted order.
    let mut referenced = BTreeSet::new();
    for a in &cluster.attributes {
        if global_hashes.contains_key(&a.type_str) {
            referenced.insert(a.type_str.as_str());
        }
        if let Some(et) = a.entry_type.as_deref()
            && global_hashes.contains_key(et)
        {
            referenced.insert(et);
        }
    }
    for s in &cluster.structs {
        for f in &s.fields {
            if global_hashes.contains_key(&f.type_str) {
                referenced.insert(f.type_str.as_str());
            }
            if let Some(et) = f.entry_type.as_deref()
                && global_hashes.contains_key(et)
            {
                referenced.insert(et);
            }
        }
    }
    for c in &cluster.commands {
        for f in &c.args {
            if global_hashes.contains_key(&f.type_str) {
                referenced.insert(f.type_str.as_str());
            }
            if let Some(et) = f.entry_type.as_deref()
                && global_hashes.contains_key(et)
            {
                referenced.insert(et);
            }
        }
    }
    for e in &cluster.events {
        for f in &e.fields {
            if global_hashes.contains_key(&f.type_str) {
                referenced.insert(f.type_str.as_str());
            }
            if let Some(et) = f.entry_type.as_deref()
                && global_hashes.contains_key(et)
            {
                referenced.insert(et);
            }
        }
    }

    h.write_usize(referenced.len());
    for gname in referenced {
        if let Some(ghash) = global_hashes.get(gname) {
            h.write_str(gname);
            h.write_str(ghash);
        }
    }

    h.finish_hex()
}

/// Format a `#[cfg(...)]` gate for a `SharedBucket`.
pub(crate) fn format_bucket_cfg(bucket: &SharedBucket) -> String {
    let version_expr = format_any_features(&bucket.versions);
    match bucket.kind {
        SharedKind::Cluster => {
            let cluster_expr = format_any_cfgs(&bucket.cluster_cfgs);
            match (cluster_expr.is_empty(), version_expr.is_empty()) {
                (false, false) => format!("all({cluster_expr}, {version_expr})"),
                (false, true) => cluster_expr,
                (true, false) => version_expr,
                (true, true) => String::new(),
            }
        }
        SharedKind::Global => {
            if bucket.version_cluster_pairs.is_empty() {
                return version_expr;
            }
            let mut by_version: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
            for (v, c) in &bucket.version_cluster_pairs {
                by_version.entry(v.as_str()).or_default().insert(c.as_str());
            }
            let mut per_ver = Vec::with_capacity(by_version.len());
            for (v, clusters) in by_version {
                let c_expr = format_any_cfgs_ref(&clusters);
                per_ver.push(format!("all(feature = \"{v}\", {c_expr})"));
            }
            if per_ver.len() == 1 {
                per_ver.into_iter().next().unwrap_or_default()
            } else {
                format!("any({})", per_ver.join(", "))
            }
        }
    }
}

fn format_version_slug(version: &str) -> String {
    let trimmed = version.strip_prefix('v').unwrap_or(version);
    format!("v{}", trimmed.replace('.', "_"))
}

/// Deterministic 128-bit FNV-1a hasher formatted as a 16-hex-char `h_<hex>` identifier.
struct DeterministicHasher {
    state: u128,
}

impl DeterministicHasher {
    const OFFSET_BASIS: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
    const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013b;

    fn new() -> Self {
        Self {
            state: Self::OFFSET_BASIS,
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state ^= u128::from(b);
            self.state = self.state.wrapping_mul(Self::PRIME);
        }
    }

    fn write_usize(&mut self, v: usize) {
        self.write_u64(u64::try_from(v).unwrap_or(u64::MAX));
    }

    fn write_str(&mut self, s: &str) {
        self.write_usize(s.len());
        self.write_bytes(s.as_bytes());
    }

    fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }

    fn write_bool(&mut self, v: bool) {
        self.write_bytes(&[u8::from(v)]);
    }

    fn write_opt_str(&mut self, v: Option<&str>) {
        match v {
            Some(s) => {
                self.write_bool(true);
                self.write_str(s);
            }
            None => self.write_bool(false),
        }
    }

    fn write_opt_u32(&mut self, v: Option<u32>) {
        match v {
            Some(n) => {
                self.write_bool(true);
                self.write_u32(n);
            }
            None => self.write_bool(false),
        }
    }

    fn finish_hex(&self) -> String {
        let hi = u64::try_from(self.state >> 64).unwrap_or(0);
        format!("h_{hi:016x}")
    }
}

fn privilege_tag(p: Option<AccessPrivilege>) -> u64 {
    match p {
        None => 0,
        Some(AccessPrivilege::View) => 1,
        Some(AccessPrivilege::Operate) => 2,
        Some(AccessPrivilege::Manage) => 3,
        Some(AccessPrivilege::Administer) => 4,
    }
}

fn hash_access(h: &mut DeterministicHasher, a: &AccessFlags) {
    h.write_bool(a.read);
    h.write_bool(a.write);
    h.write_bool(a.invoke);
    h.write_bool(a.fabric_scoped);
    h.write_bool(a.fabric_sensitive);
    h.write_bool(a.timed);
    h.write_u64(privilege_tag(a.read_privilege));
    h.write_u64(privilege_tag(a.write_privilege));
    h.write_u64(privilege_tag(a.invoke_privilege));
}

fn hash_list_length(h: &mut DeterministicHasher, ll: Option<&ListLength>) {
    match ll {
        Some(l) => {
            h.write_bool(true);
            h.write_opt_u32(l.min);
            h.write_opt_u32(l.max);
            h.write_opt_str(l.item_constraint.as_deref());
        }
        None => h.write_bool(false),
    }
}

fn hash_conformance(h: &mut DeterministicHasher, ops: &[ConformanceOp]) {
    h.write_usize(ops.len());
    for op in ops {
        match op {
            ConformanceOp::Mandatory => h.write_u32(1),
            ConformanceOp::Optional => h.write_u32(2),
            ConformanceOp::Disallow => h.write_u32(3),
            ConformanceOp::Provisional => h.write_u32(4),
            ConformanceOp::Deprecated => h.write_u32(5),
            ConformanceOp::Otherwise => h.write_u32(6),
            ConformanceOp::Feature(s) => {
                h.write_u32(7);
                h.write_str(s);
            }
            ConformanceOp::Attribute(s) => {
                h.write_u32(8);
                h.write_str(s);
            }
            ConformanceOp::Condition(s) => {
                h.write_u32(9);
                h.write_str(s);
            }
            ConformanceOp::Command(s) => {
                h.write_u32(16);
                h.write_str(s);
            }
            ConformanceOp::Literal(s) => {
                h.write_u32(10);
                h.write_str(s);
            }
            ConformanceOp::Not => h.write_u32(11),
            ConformanceOp::And(n) => {
                h.write_u32(12);
                h.write_usize(*n);
            }
            ConformanceOp::Or(n) => {
                h.write_u32(13);
                h.write_usize(*n);
            }
            ConformanceOp::Greater => h.write_u32(14),
            ConformanceOp::GreaterOrEqual => h.write_u32(15),
        }
    }
}

fn hash_field(h: &mut DeterministicHasher, f: &FieldDef) {
    h.write_u32(f.field_id);
    h.write_str(&f.name);
    h.write_str(&f.type_str);
    h.write_opt_str(f.entry_type.as_deref());
    h.write_bool(f.is_array);
    h.write_bool(f.is_nullable);
    h.write_bool(f.is_optional);
    h.write_opt_str(f.min.as_deref());
    h.write_opt_str(f.max.as_deref());
    h.write_opt_u32(f.length);
    h.write_opt_u32(f.min_length);
    hash_list_length(h, f.list_length.as_ref());
    hash_conformance(h, &f.conformance);
    h.write_opt_str(f.choice.as_deref());
}

fn hash_enum(h: &mut DeterministicHasher, e: &EnumDef) {
    h.write_str(&e.name);
    h.write_bool(e.is_16bit);
    h.write_usize(e.items.len());
    for (k, v) in &e.items {
        h.write_str(k);
        h.write_u32(*v);
    }
}

fn hash_bitmap(h: &mut DeterministicHasher, b: &BitmapDef) {
    h.write_str(&b.name);
    h.write_str(&b.type_str);
    h.write_usize(b.fields.len());
    for (k, v) in &b.fields {
        h.write_str(k);
        h.write_u64(*v);
    }
}

fn hash_struct(h: &mut DeterministicHasher, s: &StructDef) {
    h.write_str(&s.name);
    h.write_bool(s.is_fabric_scoped);
    h.write_usize(s.fields.len());
    for f in &s.fields {
        hash_field(h, f);
    }
}

fn hash_attribute(h: &mut DeterministicHasher, a: &AttributeDef) {
    h.write_str(&a.name);
    h.write_u32(a.code);
    h.write_str(&a.type_str);
    h.write_opt_str(a.entry_type.as_deref());
    h.write_bool(a.is_array);
    h.write_bool(a.is_nullable);
    h.write_bool(a.is_writable);
    h.write_bool(a.is_optional);
    h.write_opt_str(a.min.as_deref());
    h.write_opt_str(a.max.as_deref());
    h.write_opt_u32(a.length);
    h.write_opt_u32(a.min_length);
    hash_list_length(h, a.list_length.as_ref());
    h.write_opt_str(a.define.as_deref());
    hash_access(h, &a.access);
    hash_conformance(h, &a.conformance);
    h.write_opt_str(a.choice.as_deref());
}

fn hash_command(h: &mut DeterministicHasher, c: &CommandDef) {
    h.write_str(&c.name);
    h.write_u32(c.code);
    h.write_u32(match c.direction {
        CommandDirection::ClientToServer => 0,
        CommandDirection::ServerToClient => 1,
    });
    h.write_opt_str(c.response.as_deref());
    h.write_usize(c.args.len());
    for arg in &c.args {
        hash_field(h, arg);
    }
    hash_access(h, &c.access);
    hash_conformance(h, &c.conformance);
    h.write_opt_str(c.choice.as_deref());
}

fn hash_event(h: &mut DeterministicHasher, e: &EventDef) {
    h.write_str(&e.name);
    h.write_u32(e.code);
    h.write_str(&e.priority);
    h.write_usize(e.fields.len());
    for f in &e.fields {
        hash_field(h, f);
    }
    hash_access(h, &e.access);
    hash_conformance(h, &e.conformance);
    h.write_opt_str(e.choice.as_deref());
}

fn hash_feature(h: &mut DeterministicHasher, f: &FeatureDef) {
    h.write_u32(f.bit);
    h.write_str(&f.code);
    h.write_str(&f.name);
}

fn hash_choice(h: &mut DeterministicHasher, c: &ChoiceGroup) {
    h.write_str(&c.name);
    h.write_opt_u32(c.min);
    h.write_opt_u32(c.max);
    h.write_bool(c.more);
    h.write_usize(c.members.len());
    for m in &c.members {
        h.write_str(m);
    }
}

fn format_any_features(features: &BTreeSet<String>) -> String {
    let refs: BTreeSet<&str> = features.iter().map(String::as_str).collect();
    format_any_features_ref(&refs)
}

fn format_any_features_ref(features: &BTreeSet<&str>) -> String {
    if features.is_empty() {
        return String::new();
    }
    if features.len() == 1 {
        let f = features.iter().next().copied().unwrap_or_default();
        return format!("feature = \"{f}\"");
    }
    let inner: Vec<String> = features.iter().map(|f| format!("feature = \"{f}\"")).collect();
    format!("any({})", inner.join(", "))
}

fn format_any_cfgs(cfgs: &BTreeSet<String>) -> String {
    let refs: BTreeSet<&str> = cfgs.iter().map(String::as_str).collect();
    format_any_cfgs_ref(&refs)
}

fn format_any_cfgs_ref(cfgs: &BTreeSet<&str>) -> String {
    if cfgs.is_empty() {
        return String::new();
    }
    if cfgs.len() == 1 {
        return cfgs.iter().next().copied().unwrap_or_default().to_owned();
    }
    let inner: Vec<&str> = cfgs.iter().copied().collect();
    format!("any({})", inner.join(", "))
}
