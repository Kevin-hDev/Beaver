use super::{Project, PROJECT_STORE_UNAVAILABLE};
use std::path::Path;

pub(super) async fn backup_and_reset(
    path: &Path,
    corrupt_data: Vec<u8>,
) -> Result<Vec<Project>, String> {
    let backup = path.with_extension("json.corrupted");
    crate::services::private_store::atomic_write_async(backup, corrupt_data)
        .await
        .map_err(|_| PROJECT_STORE_UNAVAILABLE.to_string())?;
    crate::services::private_store::atomic_write_async(path.to_path_buf(), b"[]".to_vec())
        .await
        .map_err(|_| PROJECT_STORE_UNAVAILABLE.to_string())?;
    ::log::error!("[project-store] corrupt-document-backed-up-and-reset");
    Ok(Vec::new())
}
