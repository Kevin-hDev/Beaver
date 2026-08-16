#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::progress::{self, OllamaProgressReporter, OllamaProgressUpdate};
use super::release_source::{AllowlistedArchiveName, OllamaArchive, OllamaReleaseManifest};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio_util::sync::CancellationToken;

const MAX_BINARY_BYTES: u64 = 3 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_TEMPORARIES: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DownloadLimits {
    pub max_bytes: u64,
}

impl Default for DownloadLimits {
    fn default() -> Self {
        Self {
            max_bytes: MAX_BINARY_BYTES,
        }
    }
}

impl DownloadLimits {
    pub fn accepts_declared_size(&self, size: u64) -> bool {
        size > 0 && size <= self.max_bytes
    }

    pub fn accepts_stream_size(&self, received: u64, expected: u64) -> Result<(), OllamaErrorCode> {
        (received == expected && received <= self.max_bytes)
            .then_some(())
            .ok_or(OllamaErrorCode::OllamaDownloadFailed)
    }

    pub fn accepts_archive_count(&self, count: usize) -> bool {
        (1..=MAX_ARCHIVE_TEMPORARIES).contains(&count)
    }
}

pub fn bounded_archive_name(raw: &str) -> Result<AllowlistedArchiveName, OllamaErrorCode> {
    AllowlistedArchiveName::parse(raw)
}

pub async fn download_archives(
    manifest: &OllamaReleaseManifest,
    staging: &Path,
    cancellation: &CancellationToken,
) -> Result<Vec<PathBuf>, OllamaErrorCode> {
    download_archives_with_progress(manifest, staging, cancellation, None).await
}

pub async fn download_archives_with_progress(
    manifest: &OllamaReleaseManifest,
    staging: &Path,
    cancellation: &CancellationToken,
    reporter: Option<&OllamaProgressReporter>,
) -> Result<Vec<PathBuf>, OllamaErrorCode> {
    if !DownloadLimits::default().accepts_archive_count(manifest.archives().len()) {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let total = manifest.archives().iter().try_fold(0_u64, |sum, archive| {
        sum.checked_add(archive.expected_size)
            .ok_or(OllamaErrorCode::OllamaDownloadFailed)
    })?;
    progress::report(
        reporter,
        OllamaProgressUpdate {
            stage: super::types::OllamaProgressStage::Downloading,
            completed: 0,
            total,
        },
    )?;
    let mut paths = Vec::with_capacity(MAX_ARCHIVE_TEMPORARIES);
    let mut completed = 0_u64;
    for archive in manifest.archives() {
        if cancellation.is_cancelled() {
            return Err(OllamaErrorCode::OllamaOperationCancelled);
        }
        let destination = staging.join(archive.file_name.as_str());
        super::download_stream::download_archive_with_progress(
            archive,
            &destination,
            cancellation,
            reporter,
            completed,
            total,
        )
        .await?;
        completed = completed
            .checked_add(archive.expected_size)
            .ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
        paths.push(destination);
    }
    Ok(paths)
}

pub async fn download_archive(
    archive: &OllamaArchive,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    super::download_stream::download_archive_with_progress(
        archive,
        destination,
        cancellation,
        None,
        0,
        archive.expected_size,
    )
    .await
}

#[cfg(test)]
pub(crate) async fn download_fixture(
    url: &url::Url,
    expected_size: u64,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    super::download_stream::download_fixture_response(url, expected_size, destination, cancellation)
        .await
}

pub fn verify_sha256(
    path: &Path,
    expected: &super::fingerprint::Sha256Digest,
) -> Result<(), OllamaErrorCode> {
    let mut file =
        std::fs::File::open(path).map_err(|_| OllamaErrorCode::OllamaChecksumMismatch)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = std::io::Read::read(&mut file, &mut buffer)
            .map_err(|_| OllamaErrorCode::OllamaChecksumMismatch)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let actual = super::fingerprint::Sha256Digest::from_hex(&hex::encode(hasher.finalize()))
        .map_err(|_| OllamaErrorCode::OllamaChecksumMismatch)?;
    expected
        .constant_time_eq(&actual)
        .then_some(())
        .ok_or(OllamaErrorCode::OllamaChecksumMismatch)
}
