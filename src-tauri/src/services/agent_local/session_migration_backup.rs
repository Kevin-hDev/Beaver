use std::path::{Path, PathBuf};

use zeroize::Zeroizing;

use super::session_limits::{save_failed, MAX_SESSION_FILE_BYTES};

pub(super) fn backup_path(path: &Path) -> Result<PathBuf, String> {
    backup_path_for(path, "v1")
}

pub(super) fn v2_backup_path(path: &Path) -> Result<PathBuf, String> {
    backup_path_for(path, "v2")
}

pub(super) fn v3_backup_path(path: &Path) -> Result<PathBuf, String> {
    backup_path_for(path, "v3")
}

pub(super) fn v4_backup_path(path: &Path) -> Result<PathBuf, String> {
    backup_path_for(path, "v4")
}

fn backup_path_for(path: &Path, version: &str) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .ok_or_else(save_failed)?;
    Ok(path.with_file_name(format!("{name}.{version}.bak")))
}

pub(super) fn corrupt_backup_path(path: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .ok_or_else(save_failed)?;
    Ok(path.with_file_name(format!("{name}.corrupt.{}.bak", uuid::Uuid::new_v4())))
}

pub(super) async fn publish(
    path: &Path,
    backup: PathBuf,
    original: &[u8],
    current_bytes: Vec<u8>,
) -> Result<(), String> {
    ensure_exact_backup(&backup, original).await?;
    crate::services::private_store::atomic_write_async(path.to_path_buf(), current_bytes)
        .await
        .map_err(|_| save_failed())
}

pub(super) async fn acknowledge_path(backup: PathBuf, can_remove: bool) -> Result<(), String> {
    let Some(file) = crate::services::private_store::open_regular_single_link(&backup)
        .map_err(|_| save_failed())?
    else {
        return Ok(());
    };
    drop(file);
    if !can_remove {
        // Keep the exact backup when the visible session is empty: automatic
        // cleanup must not destroy the only potentially recoverable messages.
        log::warn!("session_migration_backup_retained_empty");
        return Ok(());
    }
    tokio::fs::remove_file(backup)
        .await
        .map_err(|_| save_failed())
}

pub(super) async fn ensure_exact_backup(path: &Path, original: &[u8]) -> Result<(), String> {
    match crate::services::private_store::read_bounded_regular_async(
        path.to_path_buf(),
        MAX_SESSION_FILE_BYTES,
    )
    .await
    .map_err(|_| save_failed())?
    {
        crate::services::private_store::BoundedFile::Missing => {
            crate::services::private_store::atomic_write_async(
                path.to_path_buf(),
                original.to_vec(),
            )
            .await
            .map_err(|_| save_failed())
        }
        crate::services::private_store::BoundedFile::Content(bytes) => {
            let _bytes = Zeroizing::new(bytes);
            // Une génération v1 antérieure est déjà une sauvegarde valable. La remplacer
            // détruirait précisément l'état de reprise que ce fichier doit préserver.
            log::warn!("session_migration_backup_already_exists");
            Ok(())
        }
    }
}
