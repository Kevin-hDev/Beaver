use super::download::DownloadLimits;
use super::error::OllamaErrorCode;
use super::progress::{self, OllamaProgressReporter, OllamaProgressUpdate};
use super::release_source::{allowlisted_redirect_policy, OllamaArchive};
use super::types::OllamaProgressStage;
use futures_util::StreamExt;
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(1_800);

#[derive(Clone, Copy)]
struct StreamProgress<'a> {
    reporter: Option<&'a OllamaProgressReporter>,
    completed_before: u64,
    total: u64,
}

pub(super) async fn download_archive_with_progress(
    archive: &OllamaArchive,
    destination: &Path,
    cancellation: &CancellationToken,
    reporter: Option<&OllamaProgressReporter>,
    completed_before: u64,
    total: u64,
) -> Result<(), OllamaErrorCode> {
    let limits = DownloadLimits::default();
    if !limits.accepts_declared_size(archive.expected_size)
        || destination.file_name().and_then(|name| name.to_str())
            != Some(archive.file_name.as_str())
    {
        return Err(OllamaErrorCode::OllamaDownloadFailed);
    }
    let client = client()?;
    download_response(
        &client,
        archive.url.as_url(),
        archive.expected_size,
        destination,
        cancellation,
        StreamProgress {
            reporter,
            completed_before,
            total,
        },
    )
    .await
}

fn client() -> Result<reqwest::Client, OllamaErrorCode> {
    reqwest::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .redirect(allowlisted_redirect_policy())
        .build()
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)
}

#[cfg(test)]
pub(super) async fn download_fixture_response(
    url: &url::Url,
    expected_size: u64,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), OllamaErrorCode> {
    download_response(
        &client()?,
        url,
        expected_size,
        destination,
        cancellation,
        StreamProgress {
            reporter: None,
            completed_before: 0,
            total: expected_size,
        },
    )
    .await
}

async fn download_response(
    client: &reqwest::Client,
    url: &url::Url,
    expected_size: u64,
    destination: &Path,
    cancellation: &CancellationToken,
    progress: StreamProgress<'_>,
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
    let result = receive_body(response, &mut file, expected_size, cancellation, progress).await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(destination).await;
    }
    result
}

async fn receive_body(
    response: reqwest::Response,
    file: &mut tokio::fs::File,
    expected_size: u64,
    cancellation: &CancellationToken,
    progress_state: StreamProgress<'_>,
) -> Result<(), OllamaErrorCode> {
    let limits = DownloadLimits::default();
    let mut received = 0_u64;
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
        progress::report(
            progress_state.reporter,
            OllamaProgressUpdate {
                stage: OllamaProgressStage::Downloading,
                completed: progress_state
                    .completed_before
                    .checked_add(received)
                    .ok_or(OllamaErrorCode::OllamaDownloadFailed)?,
                total: progress_state.total,
            },
        )?;
    }
    file.flush()
        .await
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    file.sync_all()
        .await
        .map_err(|_| OllamaErrorCode::OllamaDownloadFailed)?;
    limits.accepts_stream_size(received, expected_size)
}
