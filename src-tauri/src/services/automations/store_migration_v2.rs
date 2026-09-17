use super::store_wire::AutomationFile;
use std::path::{Path, PathBuf};

const BACKUP_NAME: &str = "automations.pre-v2.json";
const MARKER_NAME: &str = ".automations-v2-migrated";

pub(super) async fn load_or_migrate(
    root: &Path,
    path: PathBuf,
    bytes: Vec<u8>,
) -> Result<AutomationFile, String> {
    load_or_migrate_stopping_after(root, path, bytes, None).await
}

pub(super) async fn load_or_migrate_stopping_after(
    root: &Path,
    path: PathBuf,
    bytes: Vec<u8>,
    stop_after: Option<u8>,
) -> Result<AutomationFile, String> {
    let schema = serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|value| value.get("schema_version")?.as_u64())
        .ok_or_else(super::store::store_error)?;
    let mut file: AutomationFile =
        serde_json::from_slice(&bytes).map_err(|_| super::store::store_error())?;
    if schema != 1 {
        if schema == u64::from(super::store::AUTOMATIONS_SCHEMA_VERSION)
            && root.join(BACKUP_NAME).exists()
            && !root.join(MARKER_NAME).exists()
        {
            crate::services::private_store::atomic_write_async(
                root.join(MARKER_NAME),
                b"pending".to_vec(),
            )
            .await
            .map_err(|_| super::store::store_error())?;
        }
        return Ok(file);
    }
    preserve_exact_backup(root, &bytes).await?;
    stop(1, stop_after)?;
    file.schema_version = super::store::AUTOMATIONS_SCHEMA_VERSION;
    let migrated = serde_json::to_vec_pretty(&file).map_err(|_| super::store::store_error())?;
    crate::services::private_store::atomic_write_async(path, migrated)
        .await
        .map_err(|_| super::store::store_error())?;
    stop(2, stop_after)?;
    crate::services::private_store::atomic_write_async(root.join(MARKER_NAME), b"pending".to_vec())
        .await
        .map_err(|_| super::store::store_error())?;
    Ok(file)
}

pub(super) fn acknowledge_successful_startup(root: &Path) -> Result<(), String> {
    let marker = root.join(MARKER_NAME);
    match std::fs::read(&marker).as_deref() {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(b"pending") => crate::services::private_store::atomic_write(&marker, b"cleanup_next")
            .map_err(|_| super::store::store_error()),
        Ok(b"cleanup_next") => {
            remove_if_present(&root.join(BACKUP_NAME))?;
            remove_if_present(&marker)
        }
        _ => Err(super::store::store_error()),
    }
}

async fn preserve_exact_backup(root: &Path, bytes: &[u8]) -> Result<(), String> {
    let backup = root.join(BACKUP_NAME);
    match crate::services::private_store::read_bounded_regular_async(
        backup.clone(),
        super::store::MAX_STORE_BYTES,
    )
    .await
    .map_err(|_| super::store::store_error())?
    {
        crate::services::private_store::BoundedFile::Missing => {
            crate::services::private_store::atomic_write_async(backup, bytes.to_vec())
                .await
                .map_err(|_| super::store::store_error())
        }
        crate::services::private_store::BoundedFile::Content(existing) if existing == bytes => {
            Ok(())
        }
        _ => Err(super::store::store_error()),
    }
}

fn stop(boundary: u8, requested: Option<u8>) -> Result<(), String> {
    (requested != Some(boundary))
        .then_some(())
        .ok_or_else(super::store::store_error)
}

fn remove_if_present(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(super::store::store_error()),
    }
}
