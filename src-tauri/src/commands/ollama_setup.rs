use crate::services::ollama_manager::{BundleState, OllamaManager, OllamaStartOutcome};
use serde::Serialize;
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
pub async fn is_ollama_installed(
    manager: tauri::State<'_, OllamaManager>,
) -> Result<bool, String> {
    Ok(matches!(manager.status().await.bundle, BundleState::Ready))
}

#[tauri::command]
pub async fn download_ollama(
    on_progress: Channel<OllamaSetupProgress>,
    manager: tauri::State<'_, OllamaManager>,
) -> Result<(), String> {
    let cancel = CancellationToken::new();
    super::ollama_setup_cancel::register(cancel.clone()).await;
    let result = run_download_ollama(manager.inner(), &on_progress, &cancel).await;
    if let Err(error) = &result {
        if super::ollama_setup_cancel::is_cancelled_error(error) {
            let paths = crate::services::paths::ollama_paths(&crate::services::paths::data_dir());
            let _ = manager.stop_and_wait(Instant::now() + Duration::from_secs(1)).await;
            let _ = std::fs::remove_dir_all(paths.active);
        }
    }
    super::ollama_setup_cancel::clear().await;
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
    super::ollama_setup_install::install_ollama_to(
        manager,
        &paths.active,
        &version,
        on_progress,
        cancel,
    )
    .await?;
    start_manager_and_wait(manager, on_progress, cancel).await
}

#[tauri::command]
pub async fn cancel_ollama_setup() -> Result<(), String> {
    super::ollama_setup_cancel::cancel_active().await;
    Ok(())
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
        if cancel.is_cancelled() {
            return Err(super::ollama_setup_cancel::cancelled_error());
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

async fn resolve_install_version() -> String {
    match crate::services::ollama_manager::release_source::fetch_latest_version().await {
        Ok(version) => version.to_string(),
        Err(error) => {
            ::log::warn!("[ollama-setup] latest version unavailable: {}", error.as_str());
            crate::services::ollama_manager::release_source::fallback_version()
                .expect("bundled Ollama version must be valid")
                .to_string()
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
