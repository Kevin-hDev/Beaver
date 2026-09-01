use super::tab_store::{available, serialize_document, unavailable, TerminalTabsDocument};
use crate::services::private_store;
use std::path::{Path, PathBuf};

pub(super) const RECOVERED: &str = "terminal-tabs-recovered";

pub(super) async fn recover_oversized(path: PathBuf) -> Result<TerminalTabsDocument, String> {
    let backup = path.with_extension("json.corrupted");
    let pending = path.with_extension("json.recovery-pending");
    let source = path.with_extension("json.recovery-source");
    available(private_store::atomic_write_async(pending.clone(), b"pending".to_vec()).await)?;
    available(tokio::fs::rename(&path, &source).await)?;
    replace_backup(&source, &backup).await?;
    reset(path).await?;
    available(tokio::fs::remove_file(pending).await)?;
    log::error!("[terminal-tabs] oversized-document-backed-up-and-reset");
    Err(RECOVERED.to_string())
}

pub(super) async fn resume_oversized_recovery(path: &Path, pending: &Path) -> Result<bool, String> {
    let source = path.with_extension("json.recovery-source");
    let backup = path.with_extension("json.corrupted");
    let path_exists = available(tokio::fs::try_exists(path).await)?;
    let source_exists = available(tokio::fs::try_exists(&source).await)?;

    if path_exists && !source_exists {
        available(tokio::fs::remove_file(pending).await)?;
        return Ok(false);
    }
    if !path_exists && source_exists {
        replace_backup(&source, &backup).await?;
    } else if path_exists || !available(tokio::fs::try_exists(&backup).await)? {
        return Err(unavailable());
    }

    // Le marqueur rend chaque étape rejouable après un arrêt brutal.
    reset(path.to_path_buf()).await?;
    available(tokio::fs::remove_file(pending).await)?;
    log::error!("[terminal-tabs] interrupted-oversized-recovery-resumed");
    Ok(true)
}

pub(super) async fn recover_invalid(
    path: PathBuf,
    corrupt_data: Vec<u8>,
) -> Result<TerminalTabsDocument, String> {
    let backup = path.with_extension("json.corrupted");
    available(private_store::atomic_write_async(backup, corrupt_data).await)?;
    reset(path).await?;
    log::error!("[terminal-tabs] corrupt-document-backed-up-and-reset");
    Err(RECOVERED.to_string())
}

async fn replace_backup(source: &Path, backup: &Path) -> Result<(), String> {
    if available(tokio::fs::try_exists(backup).await)? {
        available(tokio::fs::remove_file(backup).await)?;
    }
    available(tokio::fs::rename(source, backup).await)
}

async fn reset(path: PathBuf) -> Result<(), String> {
    let empty = serialize_document(&TerminalTabsDocument::empty())?;
    available(private_store::atomic_write_async(path, empty).await)
}
