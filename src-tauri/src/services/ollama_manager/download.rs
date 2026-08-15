#![allow(dead_code)]

use super::error::OllamaErrorCode;
use super::release_source::{
    allowlisted_redirect_policy, AllowlistedArchiveName, OllamaArchive, OllamaReleaseManifest,
};
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const MAX_BINARY_BYTES: u64 = 3 * 1024 * 1024 * 1024;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(1_800);
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
    if !DownloadLimits::default().accepts_archive_count(manifest.archives().len()) {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let mut paths = Vec::with_capacity(MAX_ARCHIVE_TEMPORARIES);
    for archive in manifest.archives() {
        if cancellation.is_cancelled() {
            return Err(OllamaErrorCode::OllamaOperationCancelled);
        }
        let destination = staging.join(archive.file_name.as_str());
        download_archive(archive, &destination, cancellation).await?;
        paths.push(destination);
    }
    Ok(paths)
}

pub async fn download_archive(
    archive: &OllamaArchive,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let limits = DownloadLimits::default();
    if !limits.accepts_declared_size(archive.expected_size)
        || destination.file_name().and_then(|name| name.to_str())
            != Some(archive.file_name.as_str())
    {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .redirect(allowlisted_redirect_policy())
        .build()
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    download_response(
        &client,
        archive.url.as_url(),
        archive.expected_size,
        destination,
        cancellation,
    )
    .await
}

async fn download_response(
    client: &reqwest::Client,
    url: &url::Url,
    expected_size: u64,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let limits = DownloadLimits::default();
    if !limits.accepts_declared_size(expected_size) {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let response = tokio::select! {
        _ = cancellation.cancelled() => return Err(OllamaErrorCode::OllamaOperationCancelled),
        result = client.get(url.clone()).send() =>
            result.map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?,
    };
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length != expected_size)
    {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination)
        .await
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    let mut received = 0_u64;
    let result = async {
        let mut stream = response.bytes_stream();
        while let Some(chunk) = tokio::select! {
            _ = cancellation.cancelled() => return Err(OllamaErrorCode::OllamaOperationCancelled),
            chunk = stream.next() => chunk,
        } {
            let chunk = chunk.map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
            received = received
                .checked_add(chunk.len() as u64)
                .ok_or(OllamaErrorCode::OllamaDownloadFailed)?;
            if received > expected_size || received > limits.max_bytes {
                return Err(OllamaErrorCode::OllamaDownloadFailed);
            }
            file.write_all(&chunk)
                .await
                .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
        }
        file.flush()
            .await
            .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
        file.sync_all()
            .await
            .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
        limits.accepts_stream_size(received, expected_size)
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(destination).await;
    }
    result
}

#[cfg(test)]
pub(crate) async fn download_fixture(
    url: &url::Url,
    expected_size: u64,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    let client = reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .redirect(allowlisted_redirect_policy())
        .build()
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    download_response(&client, url, expected_size, destination, cancellation).await
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
