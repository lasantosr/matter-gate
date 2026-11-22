use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result};

pub(crate) fn find_workspace_root() -> Result<PathBuf> {
    let mut current = env::current_dir()?;
    loop {
        if current.join("Cargo.toml").exists() && current.join("spec").is_dir() {
            return Ok(current);
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    // Fallback: CARGO_MANIFEST_DIR parent
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        if let Some(parent) = manifest_path.parent()
            && parent.join("Cargo.toml").exists()
            && parent.join("spec").is_dir()
        {
            return Ok(parent.to_path_buf());
        }
    }
    anyhow::bail!("Could not locate workspace root (expected Cargo.toml and spec/ directory)")
}

pub(crate) fn write_if_changed(path: &Path, new_content: &str) -> Result<bool> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == new_content => return Ok(false),
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| format!("Failed to read {}", path.display()));
        }
    }
    fs::write(path, new_content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::process;

    use super::*;

    #[test]
    fn write_if_changed_is_idempotent() {
        let dir = env::temp_dir().join(format!("xtask_util_test_{}", process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("test.txt");

        assert!(write_if_changed(&file, "hello").unwrap());
        assert!(!write_if_changed(&file, "hello").unwrap());
        assert!(write_if_changed(&file, "world").unwrap());
        assert!(!write_if_changed(&file, "world").unwrap());

        drop(fs::remove_dir_all(&dir));
    }
}
