use std::path::Path;

pub async fn write_if_missing(path: &Path, content: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    write_atomic(path, content.as_bytes()).await
}

pub async fn write_atomic(path: &Path, content: &[u8]) -> Result<(), String> {
    crate::services::private_store::atomic_write_async(path.to_path_buf(), content.to_vec())
        .await
        .map_err(|error| {
            ::log::error!("[memory] atomic write: {error}");
            "Mémoire indisponible.".to_string()
        })
}

pub async fn read_bounded(path: &Path, max_bytes: u64) -> Result<String, String> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| storage_error("file metadata", error))?;
    if metadata.file_type().is_symlink() || metadata.len() > max_bytes {
        return Err("Fichier mémoire inaccessible.".into());
    }
    tokio::fs::read_to_string(path)
        .await
        .map_err(|error| storage_error("file read", error))
}

pub fn storage_error(operation: &str, error: std::io::Error) -> String {
    ::log::error!("[memory] {operation}: {error}");
    "Mémoire indisponible.".to_string()
}
