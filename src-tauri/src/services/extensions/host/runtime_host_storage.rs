use std::path::Path;

pub(super) fn purge_orphaned_directories(root: &Path) -> Result<(), String> {
    if root.exists() {
        std::fs::remove_dir_all(root)
            .map_err(|_| super::error_codes::HOST_UNAVAILABLE.to_string())?;
    }
    std::fs::create_dir_all(root).map_err(|_| super::error_codes::HOST_UNAVAILABLE.to_string())
}
