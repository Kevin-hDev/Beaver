use super::blocking::run_ollama_blocking;
use super::durable_fs::OllamaDurableFs;
use super::error::OllamaErrorCode;
use crate::services::paths::OllamaPaths;
use std::path::Path;
use std::sync::Arc;

pub(super) async fn prepare_staging<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    staging: &Path,
) -> Result<(), OllamaErrorCode> {
    if staging.exists() {
        return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
    }
    let fs = Arc::clone(fs);
    let staging = staging.to_path_buf();
    run_ollama_blocking(move || {
        fs.create_directory_durable(&staging)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}

pub(super) async fn commit_staging<F: OllamaDurableFs + 'static>(
    fs: &Arc<F>,
    paths: &OllamaPaths,
) -> Result<(), OllamaErrorCode> {
    let fs = Arc::clone(fs);
    let source = paths.install_staging.clone();
    let active = paths.active.clone();
    run_ollama_blocking(move || {
        if active.exists() {
            return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
        }
        fs.rename_durable(&source, &active)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}
