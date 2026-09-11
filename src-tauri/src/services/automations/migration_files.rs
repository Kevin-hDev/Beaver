use super::migration::AutomationMigrationStatus;
use std::path::{Path, PathBuf};

pub(super) async fn finish_interrupted_publication(root: &Path) -> Result<(), String> {
    if root.join("config.pre-automations-v1.json").exists()
        && root.join("automations.json").exists()
        && root.join("automation-runtime.json").exists()
        && !root.join(".automations-v1-migrated").exists()
    {
        super::store::read_all_unlocked_at(root).await?;
        if super::runtime_store::read_unlocked_at(root)
            .await?
            .is_none()
        {
            return Err(super::migration::migration_error());
        }
        write(root.join(".automations-v1-migrated"), b"pending".to_vec()).await?;
    }
    Ok(())
}

pub(super) fn stop(boundary: u8, requested: Option<u8>) -> Result<(), String> {
    if requested == Some(boundary) {
        Err(super::migration::migration_error())
    } else {
        Ok(())
    }
}

pub(super) async fn write(path: PathBuf, bytes: Vec<u8>) -> Result<(), String> {
    crate::services::private_store::atomic_write_async(path, bytes)
        .await
        .map_err(|_| super::migration::migration_error())
}

pub(super) fn cleanup(root: &Path) -> AutomationMigrationStatus {
    let marker = root.join(".automations-v1-migrated");
    if std::fs::read(&marker)
        .map(|value| value == b"cleanup_on_next_start")
        .unwrap_or(false)
        && crate::services::private_store::atomic_write(&marker, b"cleanup_after_success").is_err()
    {
        ::log::warn!("[automations] cleanup marker unavailable");
    }
    AutomationMigrationStatus::Ready
}

pub(super) fn acknowledge_successful_startup(root: &Path) -> Result<(), String> {
    let marker = root.join(".automations-v1-migrated");
    if !marker.exists() {
        return Ok(());
    }
    match std::fs::read(&marker).as_deref() {
        Ok(b"pending") => {
            crate::services::private_store::atomic_write(&marker, b"cleanup_on_next_start")
                .map_err(|_| super::migration::migration_error())
        }
        Ok(b"cleanup_after_success") => {
            remove_legacy_files(root)?;
            crate::services::private_store::atomic_write(&marker, b"complete")
                .map_err(|_| super::migration::migration_error())
        }
        Ok(b"cleanup_on_next_start") | Ok(b"complete") => Ok(()),
        _ => Err(super::migration::migration_error()),
    }
}

fn remove_legacy_files(root: &Path) -> Result<(), String> {
    for path in [
        root.join("config.pre-automations-v1.json"),
        root.join("heartbeat-runtime.json"),
    ] {
        if let Err(error) = std::fs::remove_file(path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(super::migration::migration_error());
            }
        }
    }
    Ok(())
}
