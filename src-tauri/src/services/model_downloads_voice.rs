use super::model_downloads::{emit_states, ModelDownloadManager};
use super::model_downloads_store::{DownloadEntry, DownloadStore};
use super::model_downloads_types::{
    ModelDownloadKind, ModelDownloadState, ModelDownloadStatus, MAX_PENDING_DOWNLOADS,
};
use tauri::AppHandle;
use tauri::{Emitter, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

impl ModelDownloadManager {
    pub fn restore_voice_checkpoints(
        &self,
        data_dir: &std::path::Path,
        resource_dir: &std::path::Path,
    ) {
        let Ok(catalog) = crate::services::voice::download::load_catalog(resource_dir) else {
            return;
        };
        let checkpoints = crate::services::voice::download::discover_checkpoints(data_dir);
        let mut store = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        for checkpoint in checkpoints {
            let resource_is_current = catalog
                .entries
                .iter()
                .any(|entry| checkpoint.matches_entry(entry));
            let request_is_asr = catalog.entries.iter().any(|entry| {
                entry.id == checkpoint.requested_model_id
                    && entry.role == crate::services::voice::download::VoiceModelRole::Asr
            });
            if !resource_is_current || !request_is_asr {
                continue;
            }
            if store.entries.len() >= MAX_PENDING_DOWNLOADS
                || has_voice_model(&store, &checkpoint.requested_model_id)
            {
                continue;
            }
            let id = Uuid::new_v4().to_string();
            let mut state = ModelDownloadState::new(
                ModelDownloadKind::Voice,
                checkpoint.requested_model_id,
                false,
                id.clone(),
                ModelDownloadStatus::Suspended,
            );
            state.downloaded = checkpoint.durable_bytes;
            state.total = checkpoint.expected_bytes;
            state.percent = percentage(checkpoint.durable_bytes, checkpoint.expected_bytes);
            store.order.push_back(id.clone());
            store.entries.insert(
                id,
                DownloadEntry {
                    state,
                    cancel: CancellationToken::new(),
                },
            );
        }
    }
}

fn has_voice_model(store: &DownloadStore, model_id: &str) -> bool {
    store.entries.values().any(|entry| {
        entry.state.kind == ModelDownloadKind::Voice && entry.state.model_id == model_id
    })
}

fn percentage(downloaded: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    ((downloaded.saturating_mul(100) / total).min(100)) as u8
}

pub async fn run_voice_download(
    app: AppHandle,
    manager: ModelDownloadManager,
    state: ModelDownloadState,
    cancel: CancellationToken,
) {
    let result = run_voice_download_inner(&app, &manager, &state, &cancel).await;
    let current = manager.state(&state.id).await;
    let (status, error) = match result {
        Ok(()) => (ModelDownloadStatus::Completed, None),
        Err(ref error)
            if error == "cancelled"
                && current
                    .as_ref()
                    .is_some_and(|item| item.status == ModelDownloadStatus::Suspended) =>
        {
            return;
        }
        Err(ref error) if error == "cancelled" => {
            let _ = crate::services::voice::download::cleanup_request(
                &crate::services::paths::data_dir(),
                &state.model_id,
            );
            (ModelDownloadStatus::Cancelled, None)
        }
        Err(_) => (ModelDownloadStatus::Failed, Some("model-download-failed")),
    };
    if status == ModelDownloadStatus::Completed {
        let _ = app.emit("voice-models-changed", ());
    }
    emit_states(&app, manager.finish(&state.id, status, error).await);
}

async fn run_voice_download_inner(
    app: &AppHandle,
    manager: &ModelDownloadManager,
    state: &ModelDownloadState,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|_| "model-download-invalid-model".to_string())?;
    let catalog = crate::services::voice::download::load_catalog(&resource_dir)
        .map_err(|_| "model-download-invalid-model".to_string())?;
    let asr = catalog
        .entries
        .iter()
        .find(|entry| {
            entry.id == state.model_id
                && entry.role == crate::services::voice::download::VoiceModelRole::Asr
        })
        .ok_or_else(|| "model-download-invalid-model".to_string())?;
    let vad = catalog
        .entries
        .iter()
        .find(|entry| entry.role == crate::services::voice::download::VoiceModelRole::Vad)
        .ok_or_else(|| "model-download-invalid-model".to_string())?;
    install_entry(app, manager, state, asr, cancel).await?;
    install_entry(app, manager, state, vad, cancel).await
}

async fn install_entry(
    app: &AppHandle,
    manager: &ModelDownloadManager,
    state: &ModelDownloadState,
    entry: &crate::services::voice::download::VoiceCatalogEntry,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let data_dir = crate::services::paths::data_dir();
    if crate::services::voice::download::installed_receipt(entry, &data_dir)?.is_some() {
        return Ok(());
    }
    crate::services::voice::download::ensure_disk_available(&data_dir, entry)?;
    let id = state.id.clone();
    let progress_manager = manager.clone();
    let progress_app = app.clone();
    let archive = crate::services::voice::download::download_archive(
        entry,
        &state.model_id,
        &data_dir,
        cancel,
        move |downloaded, total| {
            if let Some(states) = progress_manager.try_progress(
                &id,
                super::model_downloads_store::ProgressUpdate {
                    phase: super::model_downloads_types::ModelDownloadPhase::Downloading,
                    downloaded,
                    total,
                    percent: percentage(downloaded, total),
                },
            ) {
                emit_states(&progress_app, states);
            }
        },
    )
    .await?;
    if cancel.is_cancelled() {
        return Err("cancelled".into());
    }
    if let Some(states) = manager.try_progress(
        &state.id,
        super::model_downloads_store::ProgressUpdate {
            phase: super::model_downloads_types::ModelDownloadPhase::Installing,
            downloaded: entry.archive.bytes,
            total: entry.archive.bytes,
            percent: 100,
        },
    ) {
        emit_states(app, states);
    }
    let entry = entry.clone();
    tokio::task::spawn_blocking(move || {
        crate::services::voice::download::install_archive(&entry, &archive, &data_dir)
    })
    .await
    .map_err(|_| "model-download-failed".to_string())??;
    Ok(())
}
