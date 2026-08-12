use super::app_update_install::DownloadProgress;
use super::app_update_install_temp::{create_unique_temp_file, TemporaryUpdate};
use super::app_update_manifest::{checked_download_size, sha256_matches, ManifestAsset};
use crate::services::work_registry::ServiceWorkCancellation;
use futures_util::{Stream, StreamExt};
use sha2::{Digest, Sha256};
use std::future::Future;

pub(crate) async fn await_or_cancel<Work, Output>(
    cancellation: &ServiceWorkCancellation,
    work: Work,
) -> Result<Output, String>
where
    Work: Future<Output = Output>,
{
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => Err(download_error()),
        output = work => Ok(output),
    }
}

pub(crate) async fn write_response_to_temporary<Progress>(
    response: reqwest::Response,
    expected: &ManifestAsset,
    suffix: &str,
    cancellation: &ServiceWorkCancellation,
    progress: Progress,
) -> Result<TemporaryUpdate, String>
where
    Progress: FnMut(DownloadProgress),
{
    let (temporary, file) = create_unique_temp_file(
        crate::updater_worker::UPDATE_TEMP_PREFIX,
        &format!(".{suffix}"),
    )?;
    write_verified_stream(
        temporary,
        file,
        response.bytes_stream(),
        expected.size,
        &expected.sha256,
        cancellation,
        progress,
    )
    .await
}

pub(crate) async fn write_verified_stream<DownloadStream, Chunk, StreamError, Progress>(
    temporary: TemporaryUpdate,
    file: std::fs::File,
    stream: DownloadStream,
    expected_size: u64,
    expected_sha256: &str,
    cancellation: &ServiceWorkCancellation,
    mut progress: Progress,
) -> Result<TemporaryUpdate, String>
where
    DownloadStream: Stream<Item = Result<Chunk, StreamError>>,
    Chunk: AsRef<[u8]>,
    Progress: FnMut(DownloadProgress),
{
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::from_std(file);
    let mut stream = std::pin::pin!(stream);
    let mut downloaded = 0_u64;
    let mut hasher = Sha256::new();

    loop {
        let next = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(download_error()),
            next = stream.next() => next,
        };
        let Some(chunk) = next else {
            break;
        };
        let chunk = chunk.map_err(|_| download_error())?;
        let bytes = chunk.as_ref();
        let next = checked_download_size(downloaded, bytes.len(), expected_size)
            .ok_or_else(download_error)?;
        hasher.update(bytes);
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(download_error()),
            result = file.write_all(bytes) => result.map_err(|_| write_error())?,
        }
        downloaded = next;
        progress(DownloadProgress {
            completed: downloaded,
            total: expected_size,
        });
    }

    check_cancelled(cancellation)?;
    let actual: [u8; 32] = hasher.finalize().into();
    if downloaded != expected_size || !sha256_matches(&actual, expected_sha256) {
        return Err(download_error());
    }
    check_cancelled(cancellation)?;
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(download_error()),
        result = file.flush() => result.map_err(|_| write_error())?,
    }
    tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(download_error()),
        result = file.sync_all() => result.map_err(|_| write_error())?,
    }
    drop(file);
    check_cancelled(cancellation)?;
    Ok(temporary)
}

fn check_cancelled(cancellation: &ServiceWorkCancellation) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err(download_error())
    } else {
        Ok(())
    }
}

fn download_error() -> String {
    "update-download-error".to_string()
}

fn write_error() -> String {
    "update-write-error".to_string()
}

#[cfg(test)]
#[path = "app_update_download_tests.rs"]
mod tests;
