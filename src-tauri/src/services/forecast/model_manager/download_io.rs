use crate::services::model_downloads::{ModelDownloadPhase, ProgressUpdate};
use futures_util::StreamExt;
use reqwest::Response;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use super::super::fs_safety;
use super::model_artifacts::{sha256_matches, ModelArtifact};

const CHUNK_TIMEOUT: Duration = Duration::from_secs(60);

#[allow(clippy::too_many_arguments)]
pub(super) async fn write_model_response(
    response: Response,
    destination: &Path,
    artifact: &ModelArtifact,
    downloaded: &mut u64,
    total_size: u64,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
    minimum_progress_step: f64,
) -> Result<(), String> {
    let temporary = destination.with_extension("part");
    let result = write_stream(
        response,
        &temporary,
        artifact,
        downloaded,
        total_size,
        cancel,
        on_progress,
        last_percent_sent,
        minimum_progress_step,
    )
    .await;
    if let Err(error) = result {
        let _ = fs_safety::remove_path(&temporary).await;
        return Err(error);
    }
    fs_safety::remove_path(destination)
        .await
        .map_err(|_| "Finalisation du téléchargement impossible".to_string())?;
    tokio::fs::rename(&temporary, destination)
        .await
        .map_err(|_| "Finalisation du téléchargement impossible".to_string())
}

#[allow(clippy::too_many_arguments)]
async fn write_stream(
    response: Response,
    temporary: &Path,
    artifact: &ModelArtifact,
    downloaded: &mut u64,
    total_size: u64,
    cancel: &CancellationToken,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
    minimum_progress_step: f64,
) -> Result<(), String> {
    fs_safety::remove_path(temporary)
        .await
        .map_err(|_| "Création du téléchargement impossible".to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .await
        .map_err(|_| "Création du téléchargement impossible".to_string())?;
    let mut file_size = 0u64;
    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    loop {
        if cancel.is_cancelled() {
            return Err("cancelled".to_string());
        }
        let next = tokio::time::timeout(CHUNK_TIMEOUT, stream.next())
            .await
            .map_err(|_| "Téléchargement expiré".to_string())?;
        let Some(chunk) = next else { break };
        let chunk = chunk.map_err(|_| "Téléchargement interrompu".to_string())?;
        file_size = file_size
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| "Téléchargement invalide".to_string())?;
        *downloaded = downloaded
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| "Téléchargement invalide".to_string())?;
        if file_size > artifact.size || *downloaded > total_size {
            return Err("Téléchargement invalide".to_string());
        }
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|_| "Écriture du téléchargement impossible".to_string())?;
        report_progress(
            *downloaded,
            total_size,
            on_progress,
            last_percent_sent,
            minimum_progress_step,
        );
    }
    let digest = hasher.finalize();
    if file_size != artifact.size || !sha256_matches(&digest, &artifact.sha256) {
        return Err("Téléchargement du modèle invalide".to_string());
    }
    file.flush()
        .await
        .map_err(|_| "Validation du téléchargement impossible".to_string())?;
    file.sync_all()
        .await
        .map_err(|_| "Validation du téléchargement impossible".to_string())
}

fn report_progress(
    downloaded: u64,
    total_size: u64,
    on_progress: &(dyn Fn(ProgressUpdate) + Send + Sync),
    last_percent_sent: &mut f64,
    minimum_progress_step: f64,
) {
    let percent = (downloaded as f64 / total_size as f64) * 100.0;
    if percent >= *last_percent_sent + minimum_progress_step || percent >= 99.9 {
        *last_percent_sent = percent;
        on_progress(ProgressUpdate {
            phase: ModelDownloadPhase::Downloading,
            downloaded,
            total: total_size,
            percent: percent.min(100.0).round() as u8,
        });
    }
}
