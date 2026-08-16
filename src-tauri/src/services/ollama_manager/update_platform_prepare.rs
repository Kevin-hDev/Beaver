use super::{preflight, PlatformUpdateBackend};
use crate::services::ollama_manager::blocking::run_ollama_blocking;
use crate::services::ollama_manager::bundle_install::{prepare_bundle, write_metadata};
use crate::services::ollama_manager::download::{download_archives_with_progress, verify_sha256};
use crate::services::ollama_manager::durable_fs::OllamaDurableFs;
use crate::services::ollama_manager::error::OllamaErrorCode;
use crate::services::ollama_manager::extract::{extract_archive, extract_archive_overlay};
use crate::services::ollama_manager::install_archives::remove_archives;
use crate::services::ollama_manager::probe::PreparedBundle;
use crate::services::ollama_manager::progress;
use crate::services::ollama_manager::types::OllamaProgressStage;
use crate::services::ollama_manager::update::UpdateRequest;
use std::sync::Arc;

pub(super) async fn prepare(
    backend: &PlatformUpdateBackend,
    request: &UpdateRequest,
) -> Result<PreparedBundle, OllamaErrorCode> {
    let manifest = request
        .manifest
        .clone()
        .ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
    if manifest.version != request.version {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    preflight::ensure_absent(&request.paths.update_staging)?;
    preflight::ensure_absent(&request.paths.archive_staging)?;
    create_staging(backend, request).await?;
    let archives = download_archives_with_progress(
        &manifest,
        &request.paths.archive_staging,
        &request.cancellation,
        request.progress.as_ref(),
    )
    .await?;
    if archives.len() != manifest.archives().len() {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    for (index, (archive, path)) in manifest.archives().iter().zip(&archives).enumerate() {
        progress::report_stage(request.progress.as_ref(), OllamaProgressStage::Verifying)?;
        verify_sha256(path, &archive.sha256)?;
        progress::report_stage(request.progress.as_ref(), OllamaProgressStage::Extracting)?;
        let extract = if index == 0 {
            extract_archive
        } else {
            extract_archive_overlay
        };
        extract(
            path,
            &request.paths.update_staging,
            archive.file_name.as_str(),
            &request.cancellation,
        )?;
    }
    remove_archives(&backend.fs, &request.paths.archive_staging, &archives).await?;
    let mut paths = request.paths.clone();
    paths.install_staging = request.paths.update_staging.clone();
    let prepared = prepare_bundle(&paths, &request.version).await?;
    write_metadata(&backend.fs, &paths, &prepared).await?;
    Ok(prepared)
}

async fn create_staging(
    backend: &PlatformUpdateBackend,
    request: &UpdateRequest,
) -> Result<(), OllamaErrorCode> {
    let fs = Arc::clone(&backend.fs);
    let update_staging = request.paths.update_staging.clone();
    let archive_staging = request.paths.archive_staging.clone();
    run_ollama_blocking(move || {
        fs.create_directory_durable(&update_staging)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
        fs.create_directory_durable(&archive_staging)
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)
    })
    .await
}
