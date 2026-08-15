use super::ollama_setup::{start_manager_and_wait, OllamaSetupProgress};
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
    let manifest = crate::services::ollama_manager::release_source::fetch_manifest(
        version.clone(),
        &crate::services::ollama_manager::release_source::archive_names_for_platform(),
    )
    .await
    .map_err(|code| code.as_str().to_string())?;
    let request = UpdateRequest {
        paths: crate::services::paths::ollama_paths(&crate::services::paths::data_dir()),
        version,
        manifest: Some(manifest),
        inherited_environment: std::env::vars_os().collect::<Vec<(OsString, OsString)>>(),
        inherited_cwd: std::env::current_dir().map_err(|_| "ollama-storage-unavailable")?,
        cancellation: CancellationToken::new(),
        deadline: None,
        sidecar: UpdateSidecar::Absent,
    };
    let _ = on_progress.send(OllamaSetupProgress {
        completed: 0,
        total: 0,
        status: "downloading".into(),
    });
    let outcome = manager
        .update(request)
        .await
        .map_err(|code| code.as_str().to_string())?;
    match outcome {
        UpdateOutcome::Updated { .. } | UpdateOutcome::AlreadyCurrent => {
            let _ = on_progress.send(OllamaSetupProgress {
                completed: 0,
                total: 0,
                status: "restarting".into(),
            });
            start_manager_and_wait(manager.inner(), &on_progress, &CancellationToken::new())
                .await
        }
        UpdateOutcome::CleanupPending { code } | UpdateOutcome::Deferred { code } => {
            Err(code.as_str().to_string())
        }
    }
}
