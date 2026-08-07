use super::memory_paths::MemoryScope;
use std::path::Path;

pub async fn store(
    scope: &MemoryScope,
    source: &Path,
    content: &str,
) -> Result<Vec<String>, String> {
    let archive_dir = scope.archive_dir();
    reject_archive_symlink(&archive_dir)?;
    crate::services::private_store::ensure_private_dir_async(archive_dir.clone())
        .await
        .map_err(log_archive_error)?;
    reject_archive_symlink(&archive_dir)?;
    super::memory_paths::validate_in_scope(scope, &archive_dir)?;

    let file_name = source
        .file_name()
        .ok_or_else(|| "Sujet mémoire invalide.".to_string())?;
    let destination = archive_dir.join(file_name);
    crate::services::private_store::write_new_async(
        destination.clone(),
        content.as_bytes().to_vec(),
    )
    .await
    .map_err(log_archive_error)?;

    if source.exists() {
        if let Err(error) = tokio::fs::remove_file(source).await {
            let _ = tokio::fs::remove_file(&destination).await;
            return Err(super::memory_io::storage_error("archive source removal", error));
        }
    }

    let mut changed = vec![
        source.to_string_lossy().into_owned(),
        destination.to_string_lossy().into_owned(),
    ];
    changed.extend(super::memory_index::rebuild(scope).await?);
    Ok(changed)
}

fn reject_archive_symlink(path: &Path) -> Result<(), String> {
    if std::fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err("Lien symbolique mémoire interdit.".into());
    }
    Ok(())
}

fn log_archive_error(error: String) -> String {
    ::log::error!("[memory] archive write: {error}");
    "Mémoire indisponible.".to_string()
}
