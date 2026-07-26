pub use super::model_downloads_store::{ModelDownloadManager, ProgressUpdate};
pub use super::model_downloads_types::{
    ModelDownloadKind, ModelDownloadPhase, ModelDownloadState, ModelDownloadStatus, EVENT_NAME,
};

use crate::services::agent_local::{
    model_customizations, ollama_client::OllamaClient, ollama_registry, types_ollama::PullProgress,
};
use crate::services::forecast::model_manager;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

pub async fn run_ollama_download(
    app: AppHandle,
    manager: ModelDownloadManager,
    state: ModelDownloadState,
    cancel: CancellationToken,
) {
    let id = state.id.clone();
    let model = state.model_id.clone();
    let ollama = OllamaClient::new();
    let saved = if state.is_update {
        model_customizations::save_for_update(&ollama, &model).await
    } else {
        None
    };
    let mut digests = Vec::new();
    let progress = |event: PullProgress| {
        let percent = progress_percent(event.completed, event.total);
        let manager = manager.clone();
        let app = app.clone();
        let id = id.clone();
        tauri::async_runtime::spawn(async move {
            emit_states(
                &app,
                manager
                    .progress(
                        &id,
                        ProgressUpdate {
                            phase: ModelDownloadPhase::Downloading,
                            downloaded: event.completed.unwrap_or_default(),
                            total: event.total.unwrap_or_default(),
                            percent,
                        },
                    )
                    .await,
            );
        });
        let _ = event.status;
    };

    let result =
        ollama_registry::pull_model_with_callback(&model, progress, &cancel, &mut digests).await;
    finish_ollama(app, manager, state, result, saved, digests).await;
}

pub async fn run_download_queue(
    app: AppHandle,
    manager: ModelDownloadManager,
    mut state: ModelDownloadState,
    mut cancel: CancellationToken,
) {
    loop {
        match state.kind {
            ModelDownloadKind::Ollama => {
                run_ollama_download(app.clone(), manager.clone(), state, cancel).await;
            }
            ModelDownloadKind::Forecast => {
                run_forecast_download(app.clone(), manager.clone(), state, cancel).await;
            }
        }
        let Some((next_state, next_cancel)) = manager.activate_next().await else {
            break;
        };
        state = next_state;
        cancel = next_cancel;
        emit_states(&app, manager.list().await);
    }
}

async fn finish_ollama(
    app: AppHandle,
    manager: ModelDownloadManager,
    state: ModelDownloadState,
    result: Result<(), String>,
    saved: Option<String>,
    digests: Vec<String>,
) {
    let id = state.id.clone();
    match result {
        Ok(()) => {
            if let Some(perso) = saved {
                model_customizations::restore_after_update(
                    &OllamaClient::new(),
                    &state.model_id,
                    &perso,
                )
                .await;
            }
            let _ = app.emit("ollama-models-changed", ());
            emit_states(
                &app,
                manager
                    .finish(&id, ModelDownloadStatus::Completed, None)
                    .await,
            );
        }
        Err(e) if e == "cancelled" => {
            if !state.is_update {
                let _ = ollama_registry::cleanup_partial_blobs(&digests);
                let _ = ollama_registry::delete_model(&state.model_id).await;
            }
            emit_states(
                &app,
                manager
                    .finish(&id, ModelDownloadStatus::Cancelled, None)
                    .await,
            );
        }
        Err(_) => emit_states(
            &app,
            manager
                .finish(
                    &id,
                    ModelDownloadStatus::Failed,
                    Some("model-download-failed"),
                )
                .await,
        ),
    }
}

pub async fn run_forecast_download(
    app: AppHandle,
    manager: ModelDownloadManager,
    state: ModelDownloadState,
    cancel: CancellationToken,
) {
    let id = state.id.clone();
    let resources = crate::storage_migration_files::install_forecast_sidecar(
        &app,
        &crate::services::paths::data_dir(),
    );
    let result = match resources {
        Ok(()) => {
            model_manager::install_with_callback(&state.model_id, &cancel, |progress| {
                let manager = manager.clone();
                let app = app.clone();
                let id = id.clone();
                tauri::async_runtime::spawn(async move {
                    emit_states(&app, manager.progress(&id, progress).await);
                });
            })
            .await
        }
        Err(error) => Err(error),
    };
    finish_forecast(app, manager, state, result).await;
}

async fn finish_forecast(
    app: AppHandle,
    manager: ModelDownloadManager,
    state: ModelDownloadState,
    result: Result<(), String>,
) {
    let status = match &result {
        Ok(()) => ModelDownloadStatus::Completed,
        Err(e) if e == "cancelled" => ModelDownloadStatus::Cancelled,
        Err(_) => ModelDownloadStatus::Failed,
    };
    let _ = app.emit("forecast-models-changed", ());
    let error = (status == ModelDownloadStatus::Failed).then_some("model-download-failed");
    emit_states(&app, manager.finish(&state.id, status, error).await);
}

fn progress_percent(completed: Option<u64>, total: Option<u64>) -> u8 {
    match (completed, total) {
        (Some(done), Some(total)) if total > 0 => ((done as f64 / total as f64) * 100.0) as u8,
        _ => 0,
    }
    .min(100)
}

pub fn emit_states(app: &AppHandle, states: Vec<ModelDownloadState>) {
    let _ = app.emit(EVENT_NAME, states);
}
