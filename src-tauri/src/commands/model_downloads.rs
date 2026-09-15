use crate::services::forecast::validation;
use crate::services::model_downloads::{
    emit_states, run_download_queue, ModelDownloadKind, ModelDownloadManager, ModelDownloadState,
};
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

const MAX_OLLAMA_MODEL_ID_LEN: usize = 200;

#[tauri::command]
pub async fn start_model_download(
    app: AppHandle,
    kind: ModelDownloadKind,
    model_id: String,
    is_update: Option<bool>,
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<ModelDownloadState, String> {
    let voice_entry = validate_download_request(&app, kind, &model_id)?;
    let manager = downloads.inner_clone();
    let (state, runner) = manager
        .start(kind, model_id, is_update.unwrap_or(false))
        .await?;
    if let Some(entry) = voice_entry {
        if let Err(error) = crate::services::voice::download::prepare_download(
            &entry,
            &state.model_id,
            &crate::services::paths::data_dir(),
        ) {
            if runner.is_some() {
                manager.worker_start_failed(&state.id).await;
            } else {
                manager
                    .finish(
                        &state.id,
                        crate::services::model_downloads_types::ModelDownloadStatus::Failed,
                        Some("model-download-failed"),
                        None,
                    )
                    .await;
            }
            return Err(error);
        }
    }
    emit_states(&app, manager.list().await);
    spawn_runner(&app, &manager, &state, runner).await?;
    Ok(state)
}

#[tauri::command]
pub async fn list_model_downloads(
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<Vec<ModelDownloadState>, String> {
    Ok(downloads.list().await)
}

#[tauri::command]
pub async fn cancel_model_download(
    app: AppHandle,
    id: String,
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<(), String> {
    if id.len() > 64 || id.contains("..") {
        return Err("model-download-not-found".into());
    }
    let manager = downloads.inner_clone();
    let before = manager.state(&id).await;
    let states = manager.cancel(&id).await?;
    if let Some(state) = before.filter(|state| {
        state.kind == ModelDownloadKind::Voice
            && matches!(
                state.status,
                crate::services::model_downloads_types::ModelDownloadStatus::Queued
                    | crate::services::model_downloads_types::ModelDownloadStatus::Suspended
            )
    }) {
        crate::services::voice::download::cleanup_request(
            &crate::services::paths::data_dir(),
            &state.model_id,
        )?;
    }
    emit_states(&app, states);
    Ok(())
}

#[tauri::command]
pub async fn resume_model_download(
    app: AppHandle,
    id: String,
    downloads: tauri::State<'_, ModelDownloadManager>,
) -> Result<ModelDownloadState, String> {
    if id.len() > 64 || id.contains("..") {
        return Err("model-download-not-found".into());
    }
    let manager = downloads.inner_clone();
    let (state, runner) = manager.resume(&id).await?;
    emit_states(&app, manager.list().await);
    spawn_runner(&app, &manager, &state, runner).await?;
    Ok(state)
}

async fn spawn_runner(
    app: &AppHandle,
    manager: &ModelDownloadManager,
    state: &ModelDownloadState,
    runner: Option<(
        CancellationToken,
        crate::services::model_downloads_store::DownloadWorkAdmission,
    )>,
) -> Result<(), String> {
    if let Some((cancel, admission)) = runner {
        let task_app = app.clone();
        let task_manager = manager.clone();
        let task_state = state.clone();
        if let Err(error) = admission.spawn(move |shutdown| {
            run_download_queue(task_app, task_manager, task_state, cancel, shutdown)
        }) {
            manager.worker_start_failed(&state.id).await;
            return Err(error.public_code().to_string());
        }
    }
    Ok(())
}

fn validate_download_request(
    app: &AppHandle,
    kind: ModelDownloadKind,
    model_id: &str,
) -> Result<Option<crate::services::voice::download::VoiceCatalogEntry>, String> {
    match kind {
        ModelDownloadKind::Forecast => {
            validation::validate_model_id(model_id)?;
            Ok(None)
        }
        ModelDownloadKind::Ollama => {
            validate_ollama_model_id(model_id)?;
            Ok(None)
        }
        ModelDownloadKind::Voice => {
            #[cfg(target_os = "linux")]
            {
                let _ = app;
                return Err("model-download-invalid-model".into());
            }
            #[cfg(any(target_os = "macos", windows))]
            {
                let resource_dir = app
                    .path()
                    .resource_dir()
                    .map_err(|_| "model-download-invalid-model".to_string())?;
                crate::services::voice::download::find_catalog_entry(&resource_dir, model_id)
                    .map(Some)
                    .map_err(|_| "model-download-invalid-model".to_string())
            }
        }
    }
}

fn validate_ollama_model_id(model_id: &str) -> Result<(), String> {
    if model_id.is_empty() || model_id.len() > MAX_OLLAMA_MODEL_ID_LEN {
        return Err("model-download-invalid-model".into());
    }
    if model_id.contains("..")
        || !model_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/'))
    {
        return Err("model-download-invalid-model".into());
    }
    Ok(())
}
