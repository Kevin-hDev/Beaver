#![cfg(test)]

use super::download::download_archives_with_progress;
use super::error::OllamaErrorCode;
use super::install::InstallRequest;
use super::release_source::OllamaReleaseManifest;
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

pub(super) async fn archive_paths(
    request: &InstallRequest,
    manifest: &OllamaReleaseManifest,
    staging: &Path,
    cancellation: &CancellationToken,
) -> Result<Vec<PathBuf>, OllamaErrorCode> {
    if let Some(paths) = request.local_archives.clone() {
        return Ok(paths);
    }
    download_archives_with_progress(manifest, staging, cancellation, request.progress.as_ref())
        .await
}
