use super::notes_validation;
use crate::services::{paths::data_dir, private_store};
use std::path::{Path, PathBuf};

const NOTES_DIRECTORY: &str = "forecast-notes";

pub(super) async fn root_for_write() -> Result<PathBuf, String> {
    notes_root(true).await?.ok_or_else(path_error)
}

pub(super) async fn directory_for_write(analysis_id: &str) -> Result<PathBuf, String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    let root = root_for_write().await?;
    let requested = root.join(analysis_id);
    ensure_private_child(&root, &requested).await
}

pub(super) async fn directory_if_exists(analysis_id: &str) -> Result<Option<PathBuf>, String> {
    notes_validation::id(analysis_id, "Identifiant d'analyse invalide")?;
    let Some(root) = notes_root(false).await? else {
        return Ok(None);
    };
    let requested = root.join(analysis_id);
    if !tokio::fs::try_exists(&requested)
        .await
        .map_err(|_| path_error())?
    {
        return Ok(None);
    }
    verified_directory(&root, &requested).await.map(Some)
}

pub(super) async fn file_for_write(analysis_id: &str, note_id: &str) -> Result<PathBuf, String> {
    notes_validation::id(note_id, "Identifiant de note invalide")?;
    let directory = directory_for_write(analysis_id).await?;
    let requested = directory.join(format!("{note_id}.md"));
    if tokio::fs::try_exists(&requested)
        .await
        .map_err(|_| path_error())?
    {
        verify_regular_file(&directory, &requested).await
    } else {
        Ok(requested)
    }
}

pub(super) async fn file_if_exists(
    analysis_id: &str,
    note_id: &str,
) -> Result<Option<PathBuf>, String> {
    notes_validation::id(note_id, "Identifiant de note invalide")?;
    let Some(directory) = directory_if_exists(analysis_id).await? else {
        return Ok(None);
    };
    let requested = directory.join(format!("{note_id}.md"));
    if !tokio::fs::try_exists(&requested)
        .await
        .map_err(|_| path_error())?
    {
        return Ok(None);
    }
    verify_regular_file(&directory, &requested).await.map(Some)
}

pub(super) async fn verify_directory_entry(
    directory: &Path,
    path: &Path,
) -> Result<PathBuf, String> {
    verify_regular_file(directory, path).await
}

pub(super) async fn verify_child_directory(parent: &Path, path: &Path) -> Result<PathBuf, String> {
    verified_directory(parent, path).await
}

async fn notes_root(create: bool) -> Result<Option<PathBuf>, String> {
    let app_root = tokio::fs::canonicalize(data_dir())
        .await
        .map_err(|_| path_error())?;
    let requested = app_root.join(NOTES_DIRECTORY);
    if !tokio::fs::try_exists(&requested)
        .await
        .map_err(|_| path_error())?
    {
        if !create {
            return Ok(None);
        }
        private_store::ensure_private_dir_async(requested.clone()).await?;
    }
    verified_directory(&app_root, &requested).await.map(Some)
}

async fn ensure_private_child(parent: &Path, requested: &Path) -> Result<PathBuf, String> {
    if tokio::fs::try_exists(requested)
        .await
        .map_err(|_| path_error())?
    {
        return verified_directory(parent, requested).await;
    }
    private_store::ensure_private_dir_async(requested.to_path_buf()).await?;
    verified_directory(parent, requested).await
}

async fn verified_directory(parent: &Path, requested: &Path) -> Result<PathBuf, String> {
    let metadata = tokio::fs::symlink_metadata(requested)
        .await
        .map_err(|_| path_error())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(path_error());
    }
    let canonical = tokio::fs::canonicalize(requested)
        .await
        .map_err(|_| path_error())?;
    if !canonical.starts_with(parent) {
        return Err(path_error());
    }
    private_store::repair_path(&canonical)?;
    Ok(canonical)
}

async fn verify_regular_file(directory: &Path, requested: &Path) -> Result<PathBuf, String> {
    let metadata = tokio::fs::symlink_metadata(requested)
        .await
        .map_err(|_| path_error())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(path_error());
    }
    let canonical = tokio::fs::canonicalize(requested)
        .await
        .map_err(|_| path_error())?;
    if !canonical.starts_with(directory) {
        return Err(path_error());
    }
    private_store::repair_path(&canonical)?;
    Ok(canonical)
}

fn path_error() -> String {
    "Chemin de note invalide".into()
}
