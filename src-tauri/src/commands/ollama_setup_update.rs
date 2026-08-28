use super::ollama_setup::{channel_progress_reporter, start_manager_and_wait, OllamaSetupProgress};
use crate::services::ollama_manager::{
    OllamaManager, OllamaVersion, UpdateOutcome, UpdateRequest, UpdateSidecar,
};
use std::ffi::OsString;
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn update_ollama_binary(
    version: String,
    on_progress: Channel<OllamaSetupProgress>,
    manager: tauri::State<'_, OllamaManager>,
) -> Result<(), String> {
    let version = OllamaVersion::parse(version.trim_start_matches('v'))
        .map_err(|_| "ollama-version-invalid")?;
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
        progress: Some(channel_progress_reporter(&on_progress)),
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
                start_manager_and_wait(manager.inner(), &on_progress, &cancellation).await
            }
            UpdateOutcome::CleanupPending { code } | UpdateOutcome::Deferred { code } => {
                Err(code.as_str().to_string())
            }
        },
        Err(code) => Err(code.as_str().to_string()),
    };
    manager.clear_operation_cancellation();
    result
}
