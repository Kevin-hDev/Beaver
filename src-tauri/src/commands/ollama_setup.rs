use crate::services::ollama_manager::{
    BundleState, InstallOutcome, InstallRequest, OllamaManager, OllamaProgressReporter,
    OllamaProgressStage, OllamaProgressUpdate, OllamaRuntimeStatus, OllamaStartOutcome,
    OllamaVersion, OperationState,
};
use serde::Serialize;
use std::ffi::OsString;
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaSetupProgress {
    pub completed: u64,
    pub total: u64,
    pub status: String,
}

#[tauri::command]
pub async fn is_ollama_installed(manager: tauri::State<'_, OllamaManager>) -> Result<bool, String> {
    Ok(matches!(manager.status().await.bundle, BundleState::Ready))
}

#[tauri::command]
pub async fn get_ollama_runtime_status(
    manager: tauri::State<'_, OllamaManager>,
) -> Result<OllamaRuntimeStatus, String> {
    Ok(manager.status().await)
}

#[tauri::command]
pub fn retry_ollama_recovery(manager: tauri::State<'_, OllamaManager>) -> Result<(), String> {
    manager
        .request_recovery_retry()
        .map_err(|code| code.as_str().to_string())
}

#[tauri::command]
pub async fn download_ollama(
    on_progress: Channel<OllamaSetupProgress>,
    manager: tauri::State<'_, OllamaManager>,
) -> Result<(), String> {
    let cancel = CancellationToken::new();
    let result = run_download_ollama(manager.inner(), &on_progress, &cancel).await;
    result
}

pub(crate) async fn run_download_ollama(
    manager: &OllamaManager,
    on_progress: &Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if matches!(manager.status().await.bundle, BundleState::Ready) {
        return start_manager_and_wait(manager, on_progress, cancel).await;
    }
    let paths = crate::services::paths::ollama_paths(&crate::services::paths::data_dir());
    let version = resolve_install_version().await;
    let manifest = crate::services::ollama_manager::release_source::fetch_manifest(
        version.clone(),
        &crate::services::ollama_manager::release_source::archive_names_for_platform(),
    )
    .await
    .map_err(|code| code.as_str().to_string())?;
    let request = InstallRequest {
        paths,
        version: Some(version),
        manifest: Some(manifest),
        inherited_environment: std::env::vars_os().collect::<Vec<(OsString, OsString)>>(),
        inherited_cwd: std::env::current_dir().map_err(|_| "ollama-storage-unavailable")?,
        cancellation: cancel.clone(),
        deadline: None,
        progress: Some(channel_progress_reporter(on_progress)),
        #[cfg(test)]
        local_archives: None,
    };
    match manager
        .install(request)
        .await
        .map_err(|code| code.as_str().to_string())?
    {
        InstallOutcome::Installed { .. } => {}
        InstallOutcome::Preparing => return Err("ollama-install-incomplete".into()),
    }
    start_manager_and_wait(manager, on_progress, cancel).await
}

pub(super) fn channel_progress_reporter(
    on_progress: &Channel<OllamaSetupProgress>,
) -> OllamaProgressReporter {
    let channel = on_progress.clone();
    std::sync::Arc::new(move |update: OllamaProgressUpdate| {
        let _ = channel.send(OllamaSetupProgress {
            completed: update.completed,
            total: update.total,
            status: progress_status(update.stage).into(),
        });
    })
}

pub(super) fn progress_status(stage: OllamaProgressStage) -> &'static str {
    match stage {
        OllamaProgressStage::Preparing => "preparing",
        OllamaProgressStage::Downloading => "downloading",
        OllamaProgressStage::Verifying => "verifying",
        OllamaProgressStage::Extracting => "extracting",
        OllamaProgressStage::Validating => "validating",
        OllamaProgressStage::Committing => "committing",
        OllamaProgressStage::Starting => "starting",
        OllamaProgressStage::Recovering => "recovering",
        OllamaProgressStage::RollingBack => "rolling_back",
        OllamaProgressStage::Cleaning => "cleaning",
    }
}

#[tauri::command]
pub async fn cancel_ollama_setup(manager: tauri::State<'_, OllamaManager>) -> Result<(), String> {
    match manager.cancel_operation().await {
        crate::services::ollama_manager::CancelOutcome::RejectedDuringShutdown => {
            Err("ollama-closing".into())
        }
        crate::services::ollama_manager::CancelOutcome::Cancelled
        | crate::services::ollama_manager::CancelOutcome::AlreadyIdle => Ok(()),
    }
}

#[tauri::command]
pub async fn restart_ollama_sidecar(
    manager: tauri::State<'_, OllamaManager>,
) -> Result<bool, String> {
    Ok(matches!(
        manager.restart().await,
        OllamaStartOutcome::OwnedStarted { .. }
            | OllamaStartOutcome::OwnedAlreadyRunning { .. }
            | OllamaStartOutcome::ExternalAvailable { .. }
    ))
}

pub(crate) async fn start_manager_and_wait(
    manager: &OllamaManager,
    on_progress: &Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let _ = on_progress.send(OllamaSetupProgress {
        completed: 0,
        total: 0,
        status: "starting".into(),
    });
    match manager.start().await {
        OllamaStartOutcome::OwnedStarted { .. }
        | OllamaStartOutcome::OwnedAlreadyRunning { .. }
        | OllamaStartOutcome::ExternalAvailable { .. } => {}
        OllamaStartOutcome::BlockedByRecovery { code } | OllamaStartOutcome::Failed { code } => {
            return Err(code.as_str().into())
        }
        OllamaStartOutcome::RejectedDuringShutdown => return Err("ollama-closing".into()),
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|_| "ollama-start-error")?;
    let deadline = Instant::now() + Duration::from_secs(45);
    while Instant::now() < deadline {
        if cancel.is_cancelled()
            || matches!(manager.status().await.operation, OperationState::Cancelling)
        {
            return Err("ollama-operation-cancelled".into());
        }
        if let Ok(endpoint) = manager.usable_endpoint().await {
            if client
                .get(format!("{}/api/version", endpoint.as_http_url()))
                .send()
                .await
                .map(|response| response.status().is_success())
                .unwrap_or(false)
            {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err("ollama-start-timeout".into())
}

async fn resolve_install_version() -> OllamaVersion {
    match crate::services::ollama_manager::release_source::fetch_latest_version().await {
        Ok(version) => version,
        Err(error) => {
            ::log::warn!(
                "[ollama-setup] latest version unavailable: {}",
                error.as_str()
            );
            crate::services::ollama_manager::release_source::fallback_version()
                .expect("bundled Ollama version must be valid")
        }
    }
}

#[tauri::command]
pub async fn check_model_fits_vram(size_bytes: u64) -> bool {
    let vram_mb = crate::services::gpu_detect::detect_vram_mb().unwrap_or(0);
    if vram_mb == 0 {
        return true;
    }
    size_bytes / 1_048_576 < vram_mb
}
