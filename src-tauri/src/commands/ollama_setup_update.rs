use super::ollama_setup::{start_manager_and_wait, OllamaSetupProgress};
use super::ollama_setup_progress::{
    begin as begin_progress, composite_reporter, finish as finish_progress,
};
use crate::services::ollama_manager::{
    OllamaErrorCode, OllamaManager, OllamaVersion, UpdateOutcome, UpdateRequest, UpdateSidecar,
};
use crate::services::update_progress::{UpdateOperationStatus, UpdateProgressRuntime};
use std::ffi::OsString;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn update_ollama_binary(
    app: tauri::AppHandle,
    version: String,
    on_progress: Channel<OllamaSetupProgress>,
    manager: tauri::State<'_, OllamaManager>,
    progress: tauri::State<'_, UpdateProgressRuntime>,
) -> Result<(), String> {
    let version = OllamaVersion::parse(version.trim_start_matches('v'))
        .map_err(|_| "ollama-version-invalid")?;
    let id = uuid::Uuid::new_v4().to_string();
    let label = format!("Ollama {version}");
    begin_progress(&app, &progress, &id, &label)?;
    let reporter = composite_reporter(&on_progress, &app, &progress, &id, &label);
    let cancellation = CancellationToken::new();
    let request = UpdateRequest {
        paths: crate::services::paths::ollama_paths(&crate::services::paths::data_dir()),
        version,
        manifest: None,
        inherited_environment: std::env::vars_os().collect::<Vec<(OsString, OsString)>>(),
        inherited_cwd: std::env::current_dir().map_err(|_| "ollama-storage-unavailable")?,
        cancellation: cancellation.clone(),
        deadline: None,
        sidecar: UpdateSidecar::Absent,
        progress: Some(reporter.clone()),
    };
    manager.set_operation_cancellation(cancellation.clone());
    let result = match manager.update_from_release(request).await {
        Ok(outcome) => match outcome {
            UpdateOutcome::Updated { .. } | UpdateOutcome::AlreadyCurrent => {
                let _ = on_progress.send(OllamaSetupProgress {
                    completed: 0,
                    total: 0,
                    status: "restarting".into(),
                });
                start_manager_and_wait(manager.inner(), &reporter, &cancellation).await
            }
            UpdateOutcome::CleanupPending { code } | UpdateOutcome::Deferred { code } => {
                Err(code.as_str().to_string())
            }
        },
        Err(code) => Err(code.as_str().to_string()),
    };
    manager.clear_operation_cancellation();
    let cancelled = result
        .as_ref()
        .err()
        .is_some_and(|error| error == OllamaErrorCode::OllamaOperationCancelled.as_str());
    if finish_progress(
        &app,
        &progress,
        &id,
        match &result {
            Ok(()) => UpdateOperationStatus::Completed,
            Err(_) if cancelled => UpdateOperationStatus::Cancelled,
            Err(_) => UpdateOperationStatus::Failed,
        },
        result
            .as_ref()
            .err()
            .filter(|_| !cancelled)
            .map(String::as_str),
    )
    .is_err()
    {
        log::warn!("update_progress_finalization_failed");
    }
    result
}
