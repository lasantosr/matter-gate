use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
};

use anyhow::{Context as _, Result, bail};

/// Tuple representing (major, minor, patch, build) segments of a Matter specification version.
type VersionParts = (u32, u32, u32, u32);

type BestRemoteMap = BTreeMap<(u32, u32), (VersionParts, String)>;

type BestLocalMap = BTreeMap<(u32, u32), (VersionParts, String, String)>;

/// Validate and normalize a pinned version argument into `(family, normalized_tag)`.
pub(super) fn validate_and_normalize_pin_version(pin_str: &str) -> Result<(String, String)> {
    let Some(parsed) = parse_4segment_version(pin_str) else {
        bail!(
            "Invalid pinned version '{pin_str}'. Pinned versions must specify all 4 numeric segments \
             major.minor.patch.build (e.g. '1.0.0.0' or 'v1.0.0.0')"
        );
    };
    Ok((parsed.family, parsed.normalized_tag))
}

/// Query remote release tags matching Matter specifications from the upstream repository.
pub(super) fn list_remote_versions(repo_url: &str) -> Result<Vec<(String, String)>> {
    let mut cmd = Command::new("git");
    cmd.args(["ls-remote", "--tags", repo_url, "refs/tags/v*"]);
    let output = cmd.output().context("Failed to execute git ls-remote")?;
    if !output.status.success() {
        bail!("git ls-remote failed with exit code: {:?}", output.status.code());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_remote_tags(&stdout))
}

/// Remove other cached directories in `spec_data_dir` for the same `major.minor` family except `keep_tag`.
pub(super) fn remove_other_cached_family_versions(
    spec_data_dir: &Path,
    family: &str,
    keep_tag: &str,
) -> Result<Vec<String>> {
    if !spec_data_dir.exists() {
        return Ok(Vec::new());
    }

    let mut removed = Vec::new();
    for entry in fs::read_dir(spec_data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if dir_name.eq_ignore_ascii_case(keep_tag) {
            continue;
        }

        let dir_family = if let Some(parsed) = parse_4segment_version(dir_name) {
            parsed.family
        } else {
            let (maj, min) = parse_semver_pair(dir_name);
            format!("{maj}.{min}")
        };

        if dir_family == family {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove cached directory at {}", path.display()))?;
            println!("Removed cached spec {dir_name} (replaced by {keep_tag} for family {family})");
            removed.push(dir_name.to_owned());
        }
    }

    removed.sort();
    Ok(removed)
}

/// Discover all locally cached Matter specification versions in `spec-data/` sorted semantically.
///
/// Returns a list of `(family, tag)` pairs for directories containing at least one XML definition.
pub(super) fn discover_local_versions(spec_data_dir: &Path) -> Result<Vec<(String, String)>> {
    if !spec_data_dir.exists() {
        return Ok(Vec::new());
    }
    let mut best_by_family: BestLocalMap = BTreeMap::new();
    for entry in fs::read_dir(spec_data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mut has_xml = false;
        for file_entry in fs::read_dir(&path)? {
            let file_entry = file_entry?;
            if file_entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
            {
                has_xml = true;
                break;
            }
        }
        if !has_xml {
            continue;
        }
        let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let (family, tuple, tag) = if let Some(parsed) = parse_4segment_version(dir_name) {
            (parsed.family, parsed.tuple, parsed.normalized_tag)
        } else {
            let (maj, min) = parse_semver_pair(dir_name);
            (format!("{maj}.{min}"), (maj, min, 0, 0), dir_name.to_owned())
        };
        let key = (tuple.0, tuple.1);
        match best_by_family.get(&key) {
            Some((best_tuple, _, _)) if *best_tuple >= tuple => {}
            _ => {
                best_by_family.insert(key, (tuple, family, tag));
            }
        }
    }
    let result = best_by_family
        .into_values()
        .map(|(_, family, tag)| (family, tag))
        .collect();
    Ok(result)
}

/// Resolve the git commit reference (full SHA and short SHA) for a given version tag.
#[must_use]
pub(super) fn resolve_version_ref(tag: &str, cache_dir: &Path) -> (String, String) {
    if let Ok(content) = fs::read_to_string(cache_dir.join(".ref")) {
        let trimmed = content.trim();
        if trimmed.len() >= 7 {
            let short = trimmed.get(..7).unwrap_or(trimmed);
            return (trimmed.to_owned(), short.to_owned());
        }
    }
    let (full, short) = match tag {
        "v1.0.0.2" => ("4088a77f557e8571a39338fad51a1d8eb0131d79", "4088a77"),
        "v1.1.0.2" => ("c49852a85368fe39e947ea4885cdead1d9352865", "c49852a"),
        "v1.2.0.1" => ("181b0cb14ff007ec912f2ba6627e05dfb066c008", "181b0cb"),
        "v1.3.0.0" => ("5bb5c9e23d532cea40476fc0bd1d3008522792ba", "5bb5c9e"),
        "v1.4.2.0" => ("06523c22640ceb8b89f9a11ff2325a4481a178a3", "06523c2"),
        "v1.5.1.0" => ("abcc720b48c5e59c0edcfe65c516f76ca9448aa3", "abcc720"),
        "v1.6.1.0" => ("3bcdd56ba54fb88b2afb4bfef575014671df7aa7", "3bcdd56"),
        _ => (
            tag,
            if tag.len() >= 7 {
                match tag.get(..7) {
                    Some(s) => s,
                    None => tag,
                }
            } else {
                tag
            },
        ),
    };
    (full.to_owned(), short.to_owned())
}

/// Convert a version string (e.g. `"1.3"`, `"v1.4.2.0"`) to a valid Rust module identifier (e.g. `"v1_3"`, `"v1_4"`).
#[must_use]
pub(super) fn version_to_module_name(version: &str) -> String {
    let trimmed = version.trim().trim_start_matches(['v', 'V']);
    let mut parts = trimmed.split('.');
    if let (Some(major), Some(minor)) = (parts.next(), parts.next()) {
        format!("v{major}_{minor}")
    } else {
        format!("v{}", trimmed.replace('.', "_"))
    }
}

/// Convert a version string to a Cargo feature flag name (e.g. `"v1_3"`, `"v1_4"`).
///
/// Version features in `matter-spec` now canonically align with module names (`vX_Y`).
#[must_use]
pub(super) fn version_to_feature_name(version: &str) -> String {
    version_to_module_name(version)
}

/// Fetch CSA Matter cluster definitions from the upstream Git repository using partial sparse-checkout.
pub(super) fn fetch_csa_spec_sparse(
    repo_url: &str,
    tag: &str,
    subfolder: &str,
    cache_dir: &Path,
    force: bool,
) -> Result<()> {
    if !force && cache_dir.exists() {
        let mut has_files = false;
        for entry in fs::read_dir(cache_dir)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|ext| ext == "xml") {
                has_files = true;
                break;
            }
        }
        if has_files {
            return Ok(());
        }
    }

    fs::create_dir_all(cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    let temp_dir = env::temp_dir().join(format!("chip_sparse_{}", process::id()));
    if temp_dir.exists() {
        drop(fs::remove_dir_all(&temp_dir));
    }
    fs::create_dir_all(&temp_dir)?;
    let _temp_guard = TempDirGuard(temp_dir.clone());

    println!("Cloning sparse definitions from {repo_url} (ref: {tag})...");

    let mut git_init = Command::new("git");
    git_init.args(["init"]).current_dir(&temp_dir);
    let init_status = git_init.status().context("Failed to run 'git init'")?;
    if !init_status.success() {
        bail!("'git init' failed with status: {init_status}");
    }

    let is_shallow_sparse_supported = {
        let check = Command::new("git")
            .args(["sparse-checkout", "init"])
            .current_dir(&temp_dir)
            .status();
        matches!(check, Ok(exit_status) if exit_status.success())
    };

    if !is_shallow_sparse_supported {
        bail!("Git sparse-checkout is required to fetch upstream CSA ZCL XMLs.");
    }

    let remote_status = Command::new("git")
        .args(["remote", "add", "origin", repo_url])
        .current_dir(&temp_dir)
        .status()?;
    if !remote_status.success() {
        bail!("Failed to add git remote: {remote_status}");
    }

    let config_status = Command::new("git")
        .args(["config", "core.sparseCheckout", "true"])
        .current_dir(&temp_dir)
        .status()?;
    if !config_status.success() {
        bail!("Failed to set core.sparseCheckout: {config_status}");
    }

    let sparse_status = Command::new("git")
        .args(["sparse-checkout", "set", subfolder])
        .current_dir(&temp_dir)
        .status()?;
    if !sparse_status.success() {
        bail!("Failed to set sparse-checkout pattern: {sparse_status}");
    }

    println!("Fetching partial tree for {tag}...");
    let fetch_status = Command::new("git")
        .args([
            "fetch",
            "--depth",
            "1",
            "--filter=blob:none",
            "origin",
            &format!("refs/tags/{tag}:refs/tags/{tag}"),
        ])
        .current_dir(&temp_dir)
        .status()?;

    let checkout_target = if fetch_status.success() {
        format!("refs/tags/{tag}")
    } else {
        println!("Tag fetch failed, attempting branch/commit fetch...");
        let branch_status = Command::new("git")
            .args(["fetch", "--depth", "1", "--filter=blob:none", "origin", tag])
            .current_dir(&temp_dir)
            .status()?;
        if !branch_status.success() {
            bail!("Failed to fetch {tag} from {repo_url}");
        }
        "FETCH_HEAD".to_owned()
    };

    let checkout_status = Command::new("git")
        .args(["checkout", &checkout_target])
        .current_dir(&temp_dir)
        .status()?;
    if !checkout_status.success() {
        bail!("Failed to checkout {checkout_target}: {checkout_status}");
    }

    let src_folder = temp_dir.join(subfolder);
    if !src_folder.exists() {
        bail!("Subfolder {subfolder} does not exist in repository {repo_url}");
    }

    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("xml")) {
            fs::remove_file(&path)?;
        }
    }

    let mut copied: usize = 0;
    for entry in fs::read_dir(&src_folder)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "xml") {
            let Some(file_name) = path.file_name() else {
                continue;
            };
            let dest = cache_dir.join(file_name);
            fs::copy(&path, &dest)?;
            copied = copied.saturating_add(1);
        }
    }

    let rev_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&temp_dir)
        .output();
    if let Ok(out) = rev_output
        && out.status.success()
    {
        let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        drop(fs::write(cache_dir.join(".ref"), sha));
    }

    drop(fs::remove_dir_all(&temp_dir));

    println!(
        "Successfully cached {copied} XML definitions in {}",
        cache_dir.display()
    );
    Ok(())
}

/// Print formatted table of local cached and remote available Matter specification versions.
pub(super) fn print_available_versions(repo_url: &str, spec_data_dir: &Path, fetch_remote: bool) -> Result<()> {
    println!("Matter Specification Versions:\n");

    let local_versions = list_local_versions(spec_data_dir)?;
    println!("  Local / Cached ({}):", spec_data_dir.display());
    if local_versions.is_empty() {
        println!("    (no cached versions found)");
    } else {
        for (tag_name, count) in &local_versions {
            println!("    • {tag_name:<14} ({count} XML cluster definitions)");
        }
    }
    println!();

    if fetch_remote {
        println!("  Remote Repository ({repo_url}):");
        match list_remote_versions(repo_url) {
            Ok(remote_versions) if !remote_versions.is_empty() => {
                let mut family_to_cached_tags: BTreeMap<String, Vec<String>> = BTreeMap::new();
                for (tag, _count) in &local_versions {
                    let (major, minor) = parse_semver_pair(tag);
                    family_to_cached_tags
                        .entry(format!("{major}.{minor}"))
                        .or_default()
                        .push(tag.clone());
                }

                for (ver, tag) in &remote_versions {
                    let status_str = format_remote_status(tag, family_to_cached_tags.get(ver));
                    println!("    • {ver:<6} -> {tag:<14} {status_str}");
                }
            }
            Ok(_) => {
                println!("    (no remote release tags found)");
            }
            Err(err) => {
                println!("    ⚠️  Unable to query remote tags: {err}");
                println!("       (check network connectivity or run with appropriate network permissions)");
            }
        }
        println!();
    }
    Ok(())
}

/// Parsed representation of a 4-segment Matter version.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedVersion {
    family: String,
    tuple: VersionParts,
    normalized_tag: String,
}

/// Parse major and minor version numbers for semantic sorting.
#[must_use]
fn parse_semver_pair(version_str: &str) -> (u32, u32) {
    let trimmed = version_str.trim().trim_start_matches(['v', 'V']);
    let mut parts = trimmed.split('.');
    let major = parts.next().and_then(|seg| seg.parse::<u32>().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|seg| seg.parse::<u32>().ok()).unwrap_or(0);
    (major, minor)
}

/// Parse a 4-segment version string (e.g. `"1.0.0.2"` or `"v1.4.2.0"`) into a `ParsedVersion`.
#[must_use]
fn parse_4segment_version(version_str: &str) -> Option<ParsedVersion> {
    let trimmed = version_str.trim().trim_start_matches(['v', 'V']);
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    let patch = parts.next()?.parse::<u32>().ok()?;
    let build = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let family = format!("{major}.{minor}");
    let normalized_tag = format!("v{major}.{minor}.{patch}.{build}");
    Some(ParsedVersion {
        family,
        tuple: (major, minor, patch, build),
        normalized_tag,
    })
}

/// Extract and parse remote git tags from `git ls-remote --tags` output,
/// picking the highest 4-segment numeric version per major.minor family.
#[must_use]
fn parse_remote_tags(stdout: &str) -> Vec<(String, String)> {
    let mut best_by_family: BestRemoteMap = BTreeMap::new();

    for line in stdout.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }
        let ref_part = if let Some((_hash, ref_tag)) = trimmed_line.split_once('\t') {
            ref_tag.trim()
        } else {
            trimmed_line
        };

        let tag = ref_part.strip_prefix("refs/tags/").unwrap_or(ref_part);

        if tag.ends_with("^{}") || tag.contains("test") || tag.contains("sve") {
            continue;
        }

        if !tag.starts_with(['v', 'V']) {
            continue;
        }

        let Some(parsed) = parse_4segment_version(tag) else {
            continue;
        };

        let key = (parsed.tuple.0, parsed.tuple.1);
        match best_by_family.get(&key) {
            Some((best_tuple, _)) if *best_tuple >= parsed.tuple => {}
            _ => {
                best_by_family.insert(key, (parsed.tuple, parsed.normalized_tag));
            }
        }
    }

    best_by_family
        .into_iter()
        .map(|((major, minor), (_, tag))| (format!("{major}.{minor}"), tag))
        .collect()
}

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
    }
}

/// List cached Matter specification versions present in the local `spec_data_dir`.
fn list_local_versions(spec_data_dir: &Path) -> Result<Vec<(String, usize)>> {
    if !spec_data_dir.exists() {
        return Ok(Vec::new());
    }
    let mut local_versions = Vec::new();
    for entry in fs::read_dir(spec_data_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let Some(dir_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let mut count = 0_usize;
            for file_entry in fs::read_dir(&path)? {
                let file_entry = file_entry?;
                if file_entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                {
                    count = count.saturating_add(1);
                }
            }
            if count > 0 {
                let tag_name = if dir_name.starts_with(['v', 'V']) {
                    dir_name.to_lowercase()
                } else {
                    format!("v{dir_name}")
                };
                local_versions.push((tag_name, count));
            }
        }
    }
    local_versions.sort_by_key(|(tag, _)| parse_semver_pair(tag));
    Ok(local_versions)
}

fn format_remote_status(tag: &str, cached_tags: Option<&Vec<String>>) -> String {
    let Some(cached_tags) = cached_tags else {
        return "[available to fetch]".to_owned();
    };
    if cached_tags.iter().any(|ct| ct.eq_ignore_ascii_case(tag)) {
        "[cached locally]".to_owned()
    } else {
        let old_tag = cached_tags.last().map(String::as_str).unwrap_or("");
        format!("[would replace {old_tag}]")
    }
}

#[cfg(test)]
mod tests {
    use std::{env, process};

    use super::*;

    #[test]
    fn parses_and_normalizes_4segment_version() {
        let parsed_1_0 = parse_4segment_version("1.0.0.0").expect("valid 4-segment version");
        assert_eq!(parsed_1_0.family, "1.0");
        assert_eq!(parsed_1_0.tuple, (1, 0, 0, 0));
        assert_eq!(parsed_1_0.normalized_tag, "v1.0.0.0");

        let parsed_1_4 = parse_4segment_version("v1.4.2.0").expect("valid 4-segment version with v");
        assert_eq!(parsed_1_4.family, "1.4");
        assert_eq!(parsed_1_4.tuple, (1, 4, 2, 0));
        assert_eq!(parsed_1_4.normalized_tag, "v1.4.2.0");

        let parsed_1_6 = parse_4segment_version("V1.6.1.0").expect("valid 4-segment version with uppercase V");
        assert_eq!(parsed_1_6.family, "1.6");
        assert_eq!(parsed_1_6.tuple, (1, 6, 1, 0));
        assert_eq!(parsed_1_6.normalized_tag, "v1.6.1.0");

        assert!(parse_4segment_version("1.0").is_none());
        assert!(parse_4segment_version("1.0.0").is_none());
        assert!(parse_4segment_version("invalid").is_none());
        assert!(parse_4segment_version("1.0.0.x").is_none());
    }

    #[test]
    fn validates_pin_versions() {
        let (fam1, tag1) = validate_and_normalize_pin_version("1.0.0.0").expect("valid pin");
        assert_eq!(fam1, "1.0");
        assert_eq!(tag1, "v1.0.0.0");

        let (fam2, tag2) = validate_and_normalize_pin_version("v1.4.2.0").expect("valid pin with v");
        assert_eq!(fam2, "1.4");
        assert_eq!(tag2, "v1.4.2.0");

        validate_and_normalize_pin_version("1.0").unwrap_err();
        validate_and_normalize_pin_version("1.0.0").unwrap_err();
        validate_and_normalize_pin_version("invalid").unwrap_err();
    }

    #[test]
    fn parses_remote_tags_semantic_comparison_and_filtering() {
        let stdout = r"
4088a77f557e8571a39338fad51a1d8eb0131d79	refs/tags/v1.0.0.2
06523c22640ceb8b89f9a11ff2325a4481a178a3	refs/tags/v1.4.2.0
beefbeefbeefbeefbeefbeefbeefbeefbeefbeef	refs/tags/v1.4.10.0
deaddeaddeaddeaddeaddeaddeaddeaddeaddead	refs/tags/v1.4.10.0^{}
1111111111111111111111111111111111111111	refs/tags/v1.5.0.0-test
2222222222222222222222222222222222222222	refs/tags/v1.5.0.0-sve
3bcdd56ba54fb88b2afb4bfef575014671df7aa7	refs/tags/v1.6.1.0
";
        let parsed = parse_remote_tags(stdout);
        assert_eq!(
            parsed,
            vec![
                ("1.0".to_owned(), "v1.0.0.2".to_owned()),
                ("1.4".to_owned(), "v1.4.10.0".to_owned()),
                ("1.6".to_owned(), "v1.6.1.0".to_owned()),
            ]
        );
    }

    #[test]
    fn parses_remote_tags_future_versions_v1_7_and_v2_0() {
        let stdout = r"
4088a77f557e8571a39338fad51a1d8eb0131d79	refs/tags/v1.6.1.0
06523c22640ceb8b89f9a11ff2325a4481a178a3	refs/tags/v1.7.0.0
beefbeefbeefbeefbeefbeefbeefbeefbeefbeef	refs/tags/v2.0.0.0
";
        let parsed = parse_remote_tags(stdout);
        assert_eq!(
            parsed,
            vec![
                ("1.6".to_owned(), "v1.6.1.0".to_owned()),
                ("1.7".to_owned(), "v1.7.0.0".to_owned()),
                ("2.0".to_owned(), "v2.0.0.0".to_owned()),
            ]
        );
    }

    #[test]
    fn prunes_other_cached_family_versions() -> Result<()> {
        let temp_dir = env::temp_dir().join(format!("test_prune_family_{}", process::id()));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        let v1_0_0_2 = temp_dir.join("v1.0.0.2");
        let v1_0_0_0 = temp_dir.join("v1.0.0.0");
        let v1_4_2_0 = temp_dir.join("v1.4.2.0");

        fs::create_dir_all(&v1_0_0_2)?;
        fs::create_dir_all(&v1_0_0_0)?;
        fs::create_dir_all(&v1_4_2_0)?;

        let removed = remove_other_cached_family_versions(&temp_dir, "1.0", "v1.0.0.0")?;
        assert_eq!(removed, vec!["v1.0.0.2".to_owned()]);

        assert!(!v1_0_0_2.exists());
        assert!(v1_0_0_0.exists());
        assert!(v1_4_2_0.exists());

        fs::remove_dir_all(&temp_dir)?;
        Ok(())
    }
}
