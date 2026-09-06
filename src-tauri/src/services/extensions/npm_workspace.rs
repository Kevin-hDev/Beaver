//! One private npm cache survives a stopped phase; the job owns its final cleanup.
use std::path::{Path, PathBuf};

pub(super) fn prepare(root: &Path) -> Result<PathBuf, String> {
    let workspace = root.join(".npm-cache");
    for name in ["cache", "tmp"] {
        crate::services::private_store::ensure_private_dir(&workspace.join(name))
            .map_err(|_| "Cache npm indisponible.".to_string())?;
    }
    for name in ["userconfig", "globalconfig"] {
        crate::services::private_store::atomic_write(&workspace.join(name), b"")
            .map_err(|_| "Configuration npm indisponible.".to_string())?;
    }
    Ok(workspace)
}

pub(super) fn cleanup(workspace: &Path) -> Result<(), String> {
    std::fs::remove_dir_all(workspace).map_err(|_| "Cache npm impossible à nettoyer.".to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn repeated_preparation_preserves_verified_cache_and_refuses_a_symlink() {
        let root = tempfile::tempdir().unwrap();
        let workspace = super::prepare(root.path()).unwrap();
        let cached = workspace.join("cache/entry");
        std::fs::write(&cached, "retained").unwrap();
        super::prepare(root.path()).unwrap();
        assert_eq!(std::fs::read_to_string(&cached).unwrap(), "retained");
        super::cleanup(&workspace).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/tmp", &workspace).unwrap();
            assert!(super::prepare(root.path()).is_err());
        }
    }
}
