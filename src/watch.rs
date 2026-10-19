use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{DorkForgeError, Result};

/// Content signature for a flat template directory. The registry itself
/// intentionally loads only TOML files from that directory, so this watches
/// exactly the same input surface (including additions and removals).
pub fn template_snapshot(dir: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let entries = fs::read_dir(dir).map_err(|source| DorkForgeError::IoError {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut snapshot = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|source| DorkForgeError::IoError {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        let contents = fs::read(&path).map_err(|source| DorkForgeError::IoError {
            path: path.clone(),
            source,
        })?;
        let digest = Sha256::digest(contents);
        let signature = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        snapshot.insert(path, signature);
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::template_snapshot;
    use tempfile::tempdir;

    #[test]
    fn detects_same_length_content_change() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("templates.toml");
        std::fs::write(&file, "first").unwrap();
        let before = template_snapshot(dir.path()).unwrap();
        std::fs::write(&file, "other").unwrap();
        let after = template_snapshot(dir.path()).unwrap();
        assert_ne!(before, after);
    }
}
