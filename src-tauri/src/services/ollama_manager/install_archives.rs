#![allow(dead_code)]

use super::blocking::run_ollama_blocking;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) fn archive_staging_path(staging: &Path) -> PathBuf {
    let name = staging
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ollama-bundle-install-staging");
    staging.with_file_name(format!("{name}-archives"))
}

pub(crate) async fn remove_archives<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    archive_staging: &Path,
    archives: &[PathBuf],
) -> Result<(), OllamaErrorCode> {
    for path in archives {
        let fs = Arc::clone(fs);
        let path = path.clone();
        run_ollama_blocking(move || match fs.remove_file_durable(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == super::durable_fs::OllamaFsErrorKind::NotFound => Ok(()),
            Err(_) => Err(OllamaErrorCode::OllamaStorageUnavailable),
        })
        .await?;
    }
    let fs = Arc::clone(fs);
    let archive_staging = archive_staging.to_path_buf();
    run_ollama_blocking(move || {
        fs.remove_tree(&archive_staging)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}
