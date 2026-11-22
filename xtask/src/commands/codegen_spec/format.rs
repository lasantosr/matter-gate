use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context as _, Result, bail};
use tempfile::TempDir;
use toml_edit::DocumentMut;

/// Formats the generated `matter-spec` crate using nightly `rustfmt` (bypassing `.rustfmt.toml`'s
/// `ignore` list) and `taplo fmt` for `spec/matter-spec/Cargo.toml`.
pub(super) fn format_spec_crate(root: &Path) -> Result<()> {
    println!("\nFormatting generated code with cargo +nightly fmt...");
    let (_tmp_dir, config_path) = prepare_rustfmt_config(root)?;

    let mut fmt_cmd = Command::new("cargo");
    fmt_cmd
        .arg("+nightly")
        .arg("fmt")
        .arg("-p")
        .arg("matter-spec")
        .arg("--")
        .arg("--config-path")
        .arg(&config_path)
        .current_dir(root);
    let status = fmt_cmd
        .status()
        .context("Failed to invoke cargo +nightly fmt on matter-spec")?;
    if !status.success() {
        bail!("cargo +nightly fmt failed with status: {:?}", status.code());
    }

    println!("Formatting spec Cargo.toml with taplo fmt...");
    let mut taplo_cmd = Command::new("taplo");
    taplo_cmd
        .arg("fmt")
        .arg("spec/matter-spec/Cargo.toml")
        .env("RUST_LOG", "warn")
        .current_dir(root);
    let taplo_status = taplo_cmd
        .status()
        .context("Failed to invoke taplo fmt on spec/matter-spec/Cargo.toml")?;
    if !taplo_status.success() {
        bail!("taplo fmt failed with status: {:?}", taplo_status.code());
    }

    Ok(())
}

/// Writes a temporary `.rustfmt.toml` without `ignore` inside an auto-cleaned `TempDir`.
fn prepare_rustfmt_config(root: &Path) -> Result<(TempDir, PathBuf)> {
    let src_path = root.join(".rustfmt.toml");
    let raw = fs::read_to_string(&src_path)
        .with_context(|| format!("Failed to read rustfmt config at {}", src_path.display()))?;
    let stripped = strip_rustfmt_ignore(&raw)?;

    let tmp_dir = tempfile::tempdir().context("Failed to create temporary directory for rustfmt config")?;
    let config_path = tmp_dir.path().join(".rustfmt.toml");
    fs::write(&config_path, stripped)
        .with_context(|| format!("Failed to write temporary rustfmt config at {}", config_path.display()))?;
    Ok((tmp_dir, config_path))
}

/// Removes the top-level `ignore` key from a `.rustfmt.toml` document so codegen
/// formats generated `matter-spec` files that are normally skipped by `cargo fmt`.
fn strip_rustfmt_ignore(content: &str) -> Result<String> {
    let mut doc: DocumentMut = content.parse().context("Failed to parse .rustfmt.toml")?;
    let _removed = doc.remove("ignore");
    Ok(doc.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ignore_from_rustfmt_config() -> Result<()> {
        let input = r#"max_width = 120
unstable_features = true
ignore = [
  "spec/matter-spec/src/__shared",
  "spec/matter-spec/src/v*",
]
wrap_comments = true
"#;
        let stripped = strip_rustfmt_ignore(input)?;
        assert!(
            stripped.contains("max_width = 120"),
            "expected max_width to be preserved"
        );
        assert!(
            stripped.contains("wrap_comments = true"),
            "expected wrap_comments to be preserved"
        );
        assert!(!stripped.contains("ignore"), "expected ignore key to be removed");
        Ok(())
    }

    #[test]
    fn prepares_and_cleans_up_temp_rustfmt_config() -> Result<()> {
        let src_dir = tempfile::tempdir()?;
        fs::write(
            src_dir.path().join(".rustfmt.toml"),
            "max_width = 120\nignore = [\"spec/matter-spec/src/v*\"]\n",
        )?;

        let config_path = {
            let (_tmp_dir, path) = prepare_rustfmt_config(src_dir.path())?;
            let content = fs::read_to_string(&path)?;
            assert!(content.contains("max_width = 120"), "expected config option preserved");
            assert!(!content.contains("ignore"), "expected ignore stripped");
            path
        };

        assert!(!config_path.exists(), "temp config should be cleaned up on drop");
        Ok(())
    }
}
