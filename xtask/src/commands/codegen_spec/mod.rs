mod format;
mod git;
mod layout;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context as _, Result, bail};
use git::{version_to_feature_name, version_to_module_name};
use matter_spec_gen::{Category, Render as _, SharedKind, SharedPlan, Spec, SpecInput, XmlDocument, build};
use serde::Serialize;

use crate::util::write_if_changed;

#[derive(Debug, Clone)]
pub(crate) struct CodegenSpecOptions {
    pub(crate) repo: String,
    pub(crate) path: String,
    pub(crate) fetch: bool,
    pub(crate) pin: Vec<String>,
    pub(crate) force: bool,
    pub(crate) list: bool,
    pub(crate) json: bool,
}

/// Top-level coordinator: fetch CSA definitions for requested Matter versions and generate strongly-typed Rust models.
pub(crate) fn run_codegen_spec(root: &Path, options: &CodegenSpecOptions) -> Result<()> {
    if options.force && !options.fetch && options.pin.is_empty() {
        bail!("--force requires --fetch or --pin");
    }

    let spec_data_dir = root.join("xtask").join("spec-data");
    if options.list {
        git::print_available_versions(&options.repo, &spec_data_dir, options.fetch, options.json)?;
        return Ok(());
    }

    sync_spec_cache(options, &spec_data_dir)?;

    let cached_versions = git::discover_local_versions(&spec_data_dir)?;
    if cached_versions.is_empty() {
        bail!(
            "No cached Matter XML specifications found in {}. Run `cargo xtask codegen-spec --fetch` to download them.",
            spec_data_dir.display()
        );
    }

    let spec_crate_dir = root.join("spec").join("matter-spec");
    let active_version_modules: BTreeSet<String> =
        cached_versions.iter().map(|(v, _)| version_to_module_name(v)).collect();
    prune_stale_version_dirs(&spec_crate_dir.join("src"), &active_version_modules)?;

    let mut family_to_clusters: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut family_to_endpoints: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut endpoint_to_clusters: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut all_domain_categories: BTreeSet<Category> = BTreeSet::new();
    let mut all_cluster_cfgs: BTreeSet<String> = BTreeSet::new();
    let mut version_info_list = Vec::new();

    let mut inputs = Vec::new();
    for (version, tag) in &cached_versions {
        let cache_dir = spec_data_dir.join(tag);
        let docs = load_xml_documents(&cache_dir)?;
        if !options.json {
            println!(
                "Using cached Matter spec {version} ({tag}, {} XML definitions) from {}",
                docs.len(),
                cache_dir.display()
            );
            println!("Parsing {} CSA Matter cluster XML files for {version}...", docs.len());
        }
        inputs.push(SpecInput::new(version, tag, docs));
    }

    let spec = build(inputs)?;
    let shared_plan = spec.shared_plan();
    let legacy_pool_dir = spec_crate_dir.join("src").join("__pool");
    if legacy_pool_dir.exists() {
        fs::remove_dir_all(&legacy_pool_dir)?;
    }
    let cluster_hashes = emit_shared_modules(&spec_crate_dir.join("src").join("__shared"), &spec, &shared_plan)?;

    let mut ctx = GenerationContext {
        spec_crate_dir: &spec_crate_dir,
        spec_data_dir: &spec_data_dir,
        options,
        all_domain_categories: &mut all_domain_categories,
        family_to_clusters: &mut family_to_clusters,
        family_to_endpoints: &mut family_to_endpoints,
        endpoint_to_clusters: &mut endpoint_to_clusters,
        all_cluster_cfgs: &mut all_cluster_cfgs,
        shared_plan: &shared_plan,
        cluster_hashes: &cluster_hashes,
    };

    for ver in &spec.versions {
        let version_info = render_single_version_artifacts(&mut ctx, ver)?;
        version_info_list.push(version_info);
    }

    // 4. Generate top-level src/lib.rs
    layout::generate_top_lib_rs(&spec_crate_dir.join("src").join("lib.rs"), &version_info_list)?;

    // 5. Update spec/matter-spec/Cargo.toml features table and spec/matter-spec/build.rs
    if family_to_clusters.contains_key("testing") {
        all_domain_categories.insert(Category::testing());
    }
    let domain_families: Vec<Category> = all_domain_categories.into_iter().collect();
    layout::update_spec_cargo_toml(
        &spec_crate_dir.join("Cargo.toml"),
        &version_info_list,
        &domain_families,
        &family_to_clusters,
        &family_to_endpoints,
        &endpoint_to_clusters,
    )?;
    layout::generate_spec_build_rs(
        &spec_crate_dir.join("build.rs"),
        &domain_families,
        &family_to_clusters,
        &endpoint_to_clusters,
        &all_cluster_cfgs,
    )?;

    // 6. Format generated spec crate code using nightly rustfmt and taplo
    format::format_spec_crate(root, options.json)?;

    if options.json {
        let report = CodegenReport {
            versions: version_info_list
                .into_iter()
                .map(|(tag, module, feature)| GeneratedVersionEntry { tag, module, feature })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("\nMatter specification codegen complete! Spec crate features and models are fully up to date.");
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct CodegenReport {
    versions: Vec<GeneratedVersionEntry>,
}

#[derive(Debug, Serialize)]
struct GeneratedVersionEntry {
    tag: String,
    module: String,
    feature: String,
}

fn format_any_cfg(items: &[String]) -> String {
    match items {
        [single] => single.clone(),
        _ => format!("any({})", items.join(", ")),
    }
}

fn load_xml_documents(dir: &Path) -> Result<Vec<XmlDocument<String>>> {
    let mut xml_paths = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read directory at {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("xml")) {
            xml_paths.push(path);
        }
    }
    xml_paths.sort();

    let mut docs = Vec::with_capacity(xml_paths.len());
    for path in xml_paths {
        let name = path
            .file_name()
            .and_then(|os| os.to_str())
            .with_context(|| format!("Invalid UTF-8 file name in {}", path.display()))?
            .to_owned();
        let content =
            fs::read_to_string(&path).with_context(|| format!("Failed to read XML file at {}", path.display()))?;
        docs.push(XmlDocument::new(name, content));
    }
    Ok(docs)
}

fn prune_stale_rs_files(dir: &Path, expected_files: &BTreeSet<String>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read directory at {}", dir.display()))? {
        let entry = entry.with_context(|| format!("Failed to read directory entry in {}", dir.display()))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs")
            && let Some(file_name) = path.file_name().and_then(|os| os.to_str())
            && !expected_files.contains(file_name)
        {
            fs::remove_file(&path).with_context(|| format!("Failed to remove stale file at {}", path.display()))?;
        }
    }
    Ok(())
}

fn prune_stale_version_dirs(src_dir: &Path, active_modules: &BTreeSet<String>) -> Result<()> {
    if !src_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(src_dir).with_context(|| format!("Failed to read {}", src_dir.display()))? {
        let entry = entry.with_context(|| format!("Failed to read directory entry in {}", src_dir.display()))?;
        let path = entry.path();
        if path.is_dir()
            && let Some(name) = path.file_name().and_then(|os| os.to_str())
            && name.starts_with('v')
            && !active_modules.contains(name)
        {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove stale version directory at {}", path.display()))?;
        }
    }
    Ok(())
}

fn sync_spec_cache(options: &CodegenSpecOptions, spec_data_dir: &Path) -> Result<()> {
    if options.fetch {
        let remote_versions = git::list_remote_versions(&options.repo)?;
        if !options.json {
            println!("Upstream Matter Specification Versions ({}):", options.repo);
            for (family, tag) in &remote_versions {
                println!("  • {family:<6} -> {tag}");
            }
            println!();
        }

        for (family, tag) in &remote_versions {
            let cache_dir = spec_data_dir.join(tag);
            git::fetch_csa_spec_sparse(
                &options.repo,
                tag,
                &options.path,
                &cache_dir,
                options.force,
                options.json,
            )?;
            git::remove_other_cached_family_versions(spec_data_dir, family, tag, options.json)?;
        }
    } else if !options.pin.is_empty() {
        for pin_str in &options.pin {
            let (family, normalized_tag) = git::validate_and_normalize_pin_version(pin_str)?;
            let cache_dir = spec_data_dir.join(&normalized_tag);
            git::fetch_csa_spec_sparse(
                &options.repo,
                &normalized_tag,
                &options.path,
                &cache_dir,
                options.force,
                options.json,
            )?;
            git::remove_other_cached_family_versions(spec_data_dir, &family, &normalized_tag, options.json)?;
        }
    } else {
        // Mode B: Default run from cached XML files
    }
    Ok(())
}

struct GenerationContext<'a> {
    spec_crate_dir: &'a Path,
    spec_data_dir: &'a Path,
    options: &'a CodegenSpecOptions,
    all_domain_categories: &'a mut BTreeSet<Category>,
    family_to_clusters: &'a mut BTreeMap<String, BTreeSet<String>>,
    family_to_endpoints: &'a mut BTreeMap<String, BTreeSet<String>>,
    endpoint_to_clusters: &'a mut BTreeMap<String, BTreeSet<String>>,
    all_cluster_cfgs: &'a mut BTreeSet<String>,
    shared_plan: &'a SharedPlan,
    cluster_hashes: &'a BTreeMap<(String, String), String>,
}

fn emit_version_dispatch_dir(
    version_src_dir: &Path,
    dispatch_clusters: &[layout::DispatchClusterEntry],
    dispatch_endpoints: &[layout::DispatchEndpointEntry],
    version: &str,
) -> Result<()> {
    let legacy_dispatch_rs = version_src_dir.join("dispatch.rs");
    if legacy_dispatch_rs.exists() {
        fs::remove_file(&legacy_dispatch_rs)
            .with_context(|| format!("Failed to remove stale file at {}", legacy_dispatch_rs.display()))?;
    }

    let dispatch_dir = version_src_dir.join("dispatch");
    fs::create_dir_all(&dispatch_dir)?;

    let dispatch_files = [
        ("mod.rs", layout::generate_version_dispatch_mod_rs(version)?),
        (
            "command.rs",
            layout::generate_version_dispatch_command_rs(dispatch_clusters, version)?,
        ),
        (
            "attribute.rs",
            layout::generate_version_dispatch_attribute_rs(dispatch_clusters, version)?,
        ),
        (
            "event.rs",
            layout::generate_version_dispatch_event_rs(dispatch_clusters, version)?,
        ),
        (
            "cluster.rs",
            layout::generate_version_dispatch_cluster_rs(dispatch_clusters, version)?,
        ),
        (
            "endpoint.rs",
            layout::generate_version_dispatch_endpoint_rs(dispatch_endpoints, version)?,
        ),
    ];
    let mut expected_dispatch_files = BTreeSet::new();
    for (file_name, code) in dispatch_files {
        write_if_changed(&dispatch_dir.join(file_name), &code)?;
        expected_dispatch_files.insert(file_name.to_owned());
    }
    prune_stale_rs_files(&dispatch_dir, &expected_dispatch_files)
}

fn emit_shared_modules(
    shared_dir: &Path,
    spec: &Spec,
    shared_plan: &SharedPlan,
) -> Result<BTreeMap<(String, String), String>> {
    let clusters_dir = shared_dir.join("clusters");
    let globals_dir = shared_dir.join("globals");
    fs::create_dir_all(&clusters_dir)?;
    fs::create_dir_all(&globals_dir)?;

    let mut expected_root_files = BTreeSet::new();
    expected_root_files.insert("mod.rs".to_owned());
    let mut expected_cluster_files = BTreeSet::new();
    expected_cluster_files.insert("mod.rs".to_owned());
    let mut expected_global_files = BTreeSet::new();
    expected_global_files.insert("mod.rs".to_owned());

    let mut cluster_idents = BTreeMap::new();
    let shared_mod_rs = String::from(concat!(
        "// AUTO-GENERATED BY cargo xtask codegen-spec - DO NOT EDIT MANUALLY\n",
        "#![allow(\n",
        "    clippy::absolute_paths,\n",
        "    clippy::allow_attributes,\n",
        "    clippy::arithmetic_side_effects,\n",
        "    clippy::decimal_literal_representation,\n",
        "    clippy::doc_markdown,\n",
        "    clippy::excessive_nesting,\n",
        "    clippy::indexing_slicing,\n",
        "    clippy::large_enum_variant,\n",
        "    clippy::too_many_lines,\n",
        "    clippy::unnecessary_wraps,\n",
        "    clippy::unreadable_literal,\n",
        "    reason = \"Auto-generated CSA Matter specification shared modules\",\n",
        ")]\n\n",
        "pub(crate) mod clusters;\n",
        "pub(crate) mod globals;\n",
    ));

    let mut clusters_mod_entries: BTreeMap<String, String> = BTreeMap::new();
    let mut globals_mod_entries: BTreeMap<String, String> = BTreeMap::new();

    for bucket in shared_plan.buckets.values() {
        match bucket.kind {
            SharedKind::Global => {
                let Some(rep_ver) = bucket.versions.iter().next() else {
                    continue;
                };
                let ver =
                    spec.versions.iter().find(|v| v.version == *rep_ver).with_context(|| {
                        format!("Missing version {rep_ver} for global shared bucket {}", bucket.ident)
                    })?;
                let code = ver.globals.render_single_global(&bucket.module_name, shared_plan)?;
                write_if_changed(&globals_dir.join(format!("{}.rs", bucket.ident)), &code)?;
                expected_global_files.insert(format!("{}.rs", bucket.ident));

                let cfg_expr = if bucket.version_cluster_pairs.is_empty() {
                    let ver_cfgs: Vec<String> = bucket
                        .versions
                        .iter()
                        .map(|v| format!("feature = \"{}\"", version_to_feature_name(v)))
                        .collect();
                    format_any_cfg(&ver_cfgs)
                } else {
                    let mut by_version: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
                    for (v, c) in &bucket.version_cluster_pairs {
                        by_version.entry(v.as_str()).or_default().insert(c.as_str());
                    }
                    let per_ver: Vec<String> = by_version
                        .into_iter()
                        .map(|(v, clusters)| {
                            let v_feat = version_to_feature_name(v);
                            let c_cfgs: Vec<String> = clusters.iter().map(|&c| c.to_owned()).collect();
                            let c_expr = format_any_cfg(&c_cfgs);
                            format!("all(feature = \"{v_feat}\", {c_expr})")
                        })
                        .collect();
                    format_any_cfg(&per_ver)
                };
                globals_mod_entries.insert(
                    bucket.ident.clone(),
                    format!("#[cfg({cfg_expr})]\npub(crate) mod {};\n", bucket.ident),
                );
            }
            SharedKind::Cluster => {
                let Some((rep_ver, rep_mod)) = bucket.version_cluster_pairs.iter().next() else {
                    continue;
                };

                let ver = spec
                    .versions
                    .iter()
                    .find(|v| v.version == *rep_ver)
                    .with_context(|| format!("Missing version {rep_ver} for shared bucket {}", bucket.ident))?;
                let cluster = ver
                    .clusters
                    .iter()
                    .find(|c| c.module_name() == *rep_mod)
                    .with_context(|| {
                        format!(
                            "Missing cluster {rep_mod} in {rep_ver} for shared bucket {}",
                            bucket.ident
                        )
                    })?;

                let code = cluster.render_shared(shared_plan)?;
                write_if_changed(&clusters_dir.join(format!("{}.rs", bucket.ident)), &code)?;
                expected_cluster_files.insert(format!("{}.rs", bucket.ident));

                let cluster_cfgs: Vec<String> = bucket.cluster_cfgs.iter().cloned().collect();
                let ver_cfgs: Vec<String> = bucket
                    .versions
                    .iter()
                    .map(|v| format!("feature = \"{}\"", version_to_feature_name(v)))
                    .collect();
                let cluster_expr = format_any_cfg(&cluster_cfgs);
                let ver_expr = format_any_cfg(&ver_cfgs);
                clusters_mod_entries.insert(
                    bucket.ident.clone(),
                    format!(
                        "#[cfg(all({cluster_expr}, {ver_expr}))]\npub(crate) mod {};\n",
                        bucket.ident
                    ),
                );

                for (v_ver, c_mod) in &bucket.version_cluster_pairs {
                    cluster_idents.insert((version_to_module_name(v_ver), c_mod.clone()), bucket.ident.clone());
                }
            }
        }
    }

    let mut clusters_mod_rs = String::from("// AUTO-GENERATED BY cargo xtask codegen-spec - DO NOT EDIT MANUALLY\n\n");
    for entry in clusters_mod_entries.values() {
        clusters_mod_rs.push_str(entry);
    }

    let mut globals_mod_rs = String::from("// AUTO-GENERATED BY cargo xtask codegen-spec - DO NOT EDIT MANUALLY\n\n");
    for entry in globals_mod_entries.values() {
        globals_mod_rs.push_str(entry);
    }

    write_if_changed(&shared_dir.join("mod.rs"), &shared_mod_rs)?;
    write_if_changed(&clusters_dir.join("mod.rs"), &clusters_mod_rs)?;
    write_if_changed(&globals_dir.join("mod.rs"), &globals_mod_rs)?;
    prune_stale_rs_files(shared_dir, &expected_root_files)?;
    prune_stale_rs_files(&clusters_dir, &expected_cluster_files)?;
    prune_stale_rs_files(&globals_dir, &expected_global_files)?;
    Ok(cluster_idents)
}

fn render_single_version_artifacts(
    ctx: &mut GenerationContext<'_>,
    ver: &matter_spec_gen::VersionSpec,
) -> Result<(String, String, String)> {
    let version = &ver.version;
    let tag = &ver.tag;
    let mod_name = version_to_module_name(version);
    let feat_name = version_to_feature_name(version);

    let cache_dir = ctx.spec_data_dir.join(tag);

    // 3. Emit Rust code in spec/matter-spec/src/<mod_name>/
    let version_src_dir = ctx.spec_crate_dir.join("src").join(&mod_name);
    let clusters_dir = version_src_dir.join("clusters");
    fs::create_dir_all(&clusters_dir)?;

    // Write src/<mod_name>/globals.rs
    let globals_code = ver.globals.render_with_shared(ctx.shared_plan)?;
    write_if_changed(&version_src_dir.join("globals.rs"), &globals_code)?;

    let mut cluster_modules = Vec::new();
    let mut expected_cluster_files = BTreeSet::new();
    expected_cluster_files.insert("mod.rs".to_owned());

    let mut dispatch_clusters = Vec::new();
    let mut dispatch_endpoints = Vec::new();

    for cluster in &ver.clusters {
        let family_str = cluster.category().as_str();
        let snake_name = cluster.module_name();
        let cluster_cfg = cluster.cluster_cfg();

        ctx.all_domain_categories.insert(cluster.category().clone());
        ctx.all_cluster_cfgs.insert(cluster_cfg.to_owned());
        ctx.family_to_clusters
            .entry(family_str.to_owned())
            .or_default()
            .insert(cluster_cfg.to_owned());

        let mod_file = clusters_dir.join(format!("{snake_name}.rs"));
        let code = if let Some(ident) = ctx.cluster_hashes.get(&(mod_name.clone(), snake_name.to_owned())) {
            format!(
                "// AUTO-GENERATED BY cargo xtask codegen-spec - DO NOT EDIT MANUALLY\n\npub use \
                 crate::__shared::clusters::{ident}::*;\n"
            )
        } else {
            cluster.render()?
        };
        write_if_changed(&mod_file, &code)?;

        expected_cluster_files.insert(format!("{snake_name}.rs"));
        cluster_modules.push((snake_name.to_owned(), cluster_cfg.to_owned(), cluster.name().to_owned()));

        dispatch_clusters.push(layout::DispatchClusterEntry {
            cluster_id: cluster.cluster_id(),
            module_name: snake_name.to_owned(),
            name: cluster.name().to_owned(),
            struct_name: format!("{}Cluster", cluster.name()),
            cfg: cluster_cfg.to_owned(),
            has_attributes: cluster.has_attributes(),
            has_client_commands: cluster.has_client_commands(),
            has_server_commands: cluster.has_server_commands(),
            has_events: cluster.has_events(),
            is_large_state: cluster.is_large_state(),
            is_large_patch: cluster.is_large_patch(),
        });
    }

    // Generate src/<mod_name>/clusters/mod.rs
    layout::generate_version_clusters_mod(&clusters_dir.join("mod.rs"), &cluster_modules, version)?;
    prune_stale_rs_files(&clusters_dir, &expected_cluster_files)?;

    // Generate src/<mod_name>/endpoints/
    let endpoints_dir = version_src_dir.join("endpoints");
    fs::create_dir_all(&endpoints_dir)?;

    // Remove legacy device_types or devices directory if present
    let old_device_types_dir = version_src_dir.join("device_types");
    if old_device_types_dir.exists() {
        fs::remove_dir_all(&old_device_types_dir)?;
    }
    let old_devices_dir = version_src_dir.join("devices");
    if old_devices_dir.exists() {
        fs::remove_dir_all(&old_devices_dir)?;
    }

    let mut endpoint_modules = Vec::new();
    let mut expected_endpoint_files = BTreeSet::new();
    expected_endpoint_files.insert("mod.rs".to_owned());

    for dt in &ver.device_types {
        let family_str = dt.category().as_str();
        let snake_name = dt.module_name();
        let dt_feat = dt.feature_name();

        ctx.all_domain_categories.insert(dt.category().clone());
        ctx.family_to_endpoints
            .entry(family_str.to_owned())
            .or_default()
            .insert(dt_feat.to_owned());

        let dt_deps = ctx.endpoint_to_clusters.entry(dt_feat.to_owned()).or_default();
        for dep in dt.cluster_dependencies() {
            dt_deps.insert(dep.clone());
        }

        let dt_file = endpoints_dir.join(format!("{snake_name}.rs"));
        let code = dt.render()?;
        write_if_changed(&dt_file, &code)?;

        expected_endpoint_files.insert(format!("{snake_name}.rs"));
        endpoint_modules.push((dt_feat.to_owned(), snake_name.to_owned(), dt.name().to_owned()));

        dispatch_endpoints.push(layout::DispatchEndpointEntry {
            device_id: dt.device_id(),
            module_name: snake_name.to_owned(),
            name: dt.name().to_owned(),
            feature: dt_feat.to_owned(),
            is_empty: dt.is_empty(),
            is_large: dt.is_large(),
        });
    }

    let endpoint_mod_code = layout::generate_endpoints_mod_rs(&endpoint_modules, version)?;
    write_if_changed(&endpoints_dir.join("mod.rs"), &endpoint_mod_code)?;
    prune_stale_rs_files(&endpoints_dir, &expected_endpoint_files)?;

    // Generate src/<mod_name>/registry.rs
    let registry_code = ver.render_registry()?;
    write_if_changed(&version_src_dir.join("registry.rs"), &registry_code)?;

    // Generate src/<mod_name>/dispatch/
    emit_version_dispatch_dir(&version_src_dir, &dispatch_clusters, &dispatch_endpoints, version)?;

    // Generate src/<mod_name>/mod.rs
    let (full_sha, short_sha) = git::resolve_version_ref(tag, &cache_dir);
    let repo_clean = ctx.options.repo.trim_end_matches(".git");
    let repo_folder_url = format!("{repo_clean}/tree/{full_sha}/{}", ctx.options.path);
    layout::generate_version_mod(
        &version_src_dir.join("mod.rs"),
        &cluster_modules,
        &endpoint_modules,
        version,
        tag,
        &short_sha,
        repo_clean,
        &repo_folder_url,
    )?;

    if !ctx.options.json {
        println!(
            "  -> Generated {} clusters and {} endpoints for {version} ({tag}) in {}",
            ver.clusters.len(),
            ver.device_types.len(),
            version_src_dir.display()
        );
    }

    Ok((tag.to_owned(), mod_name, feat_name))
}

#[cfg(test)]
mod tests {
    use std::{env, process};

    use super::*;

    #[test]
    fn prunes_stale_rs_files() -> Result<()> {
        let temp_dir = env::temp_dir().join(format!("test_prune_{}", process::id()));
        fs::create_dir_all(&temp_dir)?;

        let keep_rs = temp_dir.join("keep.rs");
        let stale_rs = temp_dir.join("stale.rs");
        let other_file = temp_dir.join("other.txt");

        fs::write(&keep_rs, "// keep")?;
        fs::write(&stale_rs, "// stale")?;
        fs::write(&other_file, "text")?;

        let mut expected = BTreeSet::new();
        expected.insert("keep.rs".to_owned());

        prune_stale_rs_files(&temp_dir, &expected)?;

        assert!(keep_rs.exists());
        assert!(!stale_rs.exists());
        assert!(other_file.exists());

        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }

    #[test]
    fn prunes_stale_version_directories() -> Result<()> {
        let temp_dir = env::temp_dir().join(format!("test_prune_ver_{}", process::id()));
        fs::create_dir_all(&temp_dir)?;

        let keep_dir = temp_dir.join("v1_4");
        let stale_dir = temp_dir.join("v9_9");
        let other_dir = temp_dir.join("traits");

        fs::create_dir_all(&keep_dir)?;
        fs::create_dir_all(&stale_dir)?;
        fs::create_dir_all(&other_dir)?;

        let mut active = BTreeSet::new();
        active.insert("v1_4".to_owned());

        prune_stale_version_dirs(&temp_dir, &active)?;

        assert!(keep_dir.exists());
        assert!(!stale_dir.exists());
        assert!(other_dir.exists());

        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}
