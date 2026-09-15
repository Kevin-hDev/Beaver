use std::{path::Path, time::Duration};

use futures_util::StreamExt;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

use super::{
    checkpoint::should_checkpoint, checkpoint_path, partial_path, save_checkpoint, Checkpoint,
    VoiceCatalogEntry,
};

const MAX_ATTEMPTS: u8 = 3;
const MAX_REDIRECTS: u8 = 5;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) async fn download_archive(
    entry: &VoiceCatalogEntry,
    requested_model_id: &str,
    data_dir: &Path,
    cancel: &CancellationToken,
    mut progress: impl FnMut(u64, u64),
) -> Result<std::path::PathBuf, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()
        .map_err(|_| "model-download-network-failed".to_string())?;
    download_with_client(
        entry,
        requested_model_id,
        data_dir,
        cancel,
        &client,
        false,
        &mut progress,
    )
    .await
}

pub(super) async fn download_with_client(
    entry: &VoiceCatalogEntry,
    requested_model_id: &str,
    data_dir: &Path,
    cancel: &CancellationToken,
    client: &reqwest::Client,
    allow_loopback: bool,
    progress: &mut impl FnMut(u64, u64),
) -> Result<std::path::PathBuf, String> {
    let checkpoint_file = checkpoint_path(data_dir, &entry.id);
    let partial_file = partial_path(data_dir, &entry.id);
    let mut checkpoint = super::transfer_prepare::prepare_checkpoint(
        entry,
        requested_model_id,
        &checkpoint_file,
        &partial_file,
    )?;
    progress(checkpoint.durable_bytes, entry.archive.bytes);

    for attempt in 0..MAX_ATTEMPTS {
        if cancel.is_cancelled() {
            return Err("cancelled".into());
        }
        match one_attempt(
            entry,
            (&partial_file, &checkpoint_file),
            &mut checkpoint,
            cancel,
            client,
            allow_loopback,
            progress,
        )
        .await
        {
            Ok(()) => return Ok(partial_file),
            Err(error) if error == "cancelled" => return Err(error),
            Err(_) if attempt + 1 < MAX_ATTEMPTS => {
                let delay = Duration::from_millis(250 * (u64::from(attempt) + 1));
                tokio::select! {
                    _ = cancel.cancelled() => return Err("cancelled".into()),
                    _ = tokio::time::sleep(delay) => {}
                }
            }
            Err(error) => return Err(error),
        }
    }
    Err("model-download-network-failed".into())
}

async fn one_attempt(
    entry: &VoiceCatalogEntry,
    files: (&Path, &Path),
    checkpoint: &mut Checkpoint,
    cancel: &CancellationToken,
    client: &reqwest::Client,
    allow_loopback: bool,
    progress: &mut impl FnMut(u64, u64),
) -> Result<(), String> {
    let (partial_file, checkpoint_file) = files;
    let response = super::transfer_http::send(
        client,
        &entry.archive.url,
        checkpoint,
        allow_loopback,
        MAX_REDIRECTS,
    )
    .await?;
    let response = super::transfer_http::normalize_response(
        response,
        checkpoint,
        partial_file,
        checkpoint_file,
    )?;
    let validator = super::transfer_http::response_validator(response.headers());
    if checkpoint.validator.is_none() {
        checkpoint.validator = validator;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(partial_file)
        .await
        .map_err(|_| "model-download-storage-failed".to_string())?;
    file.seek(std::io::SeekFrom::Start(checkpoint.durable_bytes))
        .await
        .map_err(|_| "model-download-storage-failed".to_string())?;
    let mut written = checkpoint.durable_bytes;
    let mut confirmed = written;
    let mut last_checkpoint = tokio::time::Instant::now();
    let mut body = response.bytes_stream();
    while let Some(chunk) = tokio::select! {
        _ = cancel.cancelled() => {
            confirm(&file, checkpoint_file, checkpoint, written).await?;
            progress(written, entry.archive.bytes);
            return Err("cancelled".into());
        },
        chunk = body.next() => chunk,
    } {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(_) => {
                confirm(&file, checkpoint_file, checkpoint, written).await?;
                progress(written, entry.archive.bytes);
                return Err("model-download-network-failed".into());
            }
        };
        written = written
            .checked_add(chunk.len() as u64)
            .filter(|bytes| *bytes <= entry.archive.bytes)
            .ok_or_else(|| "model-download-body-too-large".to_string())?;
        file.write_all(&chunk)
            .await
            .map_err(|_| "model-download-storage-failed".to_string())?;
        if should_checkpoint(written, confirmed, last_checkpoint.elapsed()) {
            confirm(&file, checkpoint_file, checkpoint, written).await?;
            confirmed = written;
            progress(written, entry.archive.bytes);
            last_checkpoint = tokio::time::Instant::now();
        }
    }
    if written != entry.archive.bytes {
        confirm(&file, checkpoint_file, checkpoint, written).await?;
        progress(written, entry.archive.bytes);
        return Err("model-download-network-failed".into());
    }
    confirm(&file, checkpoint_file, checkpoint, written).await?;
    progress(written, entry.archive.bytes);
    Ok(())
}

async fn confirm(
    file: &tokio::fs::File,
    checkpoint_file: &Path,
    checkpoint: &mut Checkpoint,
    written: u64,
) -> Result<(), String> {
    file.sync_data()
        .await
        .map_err(|_| "model-download-storage-failed".to_string())?;
    checkpoint.durable_bytes = written;
    save_checkpoint(checkpoint_file, checkpoint)
}
