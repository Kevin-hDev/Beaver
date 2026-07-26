use crate::services::ollama_lifecycle;
use serde::Serialize;
use std::sync::LazyLock;
use tauri::ipc::Channel;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub(crate) static OLLAMA_INSTALL_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub(crate) fn fallback_ollama_version() -> &'static str {
    include_str!("../../ollama-version.txt").trim()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaSetupProgress {
    pub completed: u64,
    pub total: u64,
    pub status: String,
}

#[tauri::command]
pub async fn is_ollama_installed() -> bool {
    ollama_lifecycle::is_ollama_installed_or_external()
}

#[tauri::command]
pub async fn download_ollama(
    app: tauri::AppHandle,
    on_progress: Channel<OllamaSetupProgress>,
) -> Result<(), String> {
    let _guard = OLLAMA_INSTALL_LOCK.lock().await;
    let had_existing_binary = ollama_lifecycle::ollama_binary_path().is_ok();
    let cancel = CancellationToken::new();
    super::ollama_setup_cancel::register(cancel.clone()).await;
    let result = run_download_ollama(app.clone(), on_progress, &cancel).await;
    if let Err(err) = &result {
        if !had_existing_binary && super::ollama_setup_cancel::is_cancelled_error(err) {
            ollama_lifecycle::stop_sidecar(&app);
            let _ = std::fs::remove_dir_all(ollama_lifecycle::ollama_bundle_dir());
        }
    }
    super::ollama_setup_cancel::clear().await;
    result
}

async fn run_download_ollama(
    app: tauri::AppHandle,
    on_progress: Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if ollama_lifecycle::ollama_binary_path().is_ok() {
        return super::ollama_setup_start::start_sidecar_and_wait(&app, &on_progress, cancel).await;
    }
    let dest = ollama_lifecycle::ollama_bundle_dir();
    let version = resolve_install_version().await;
    super::ollama_setup_install::install_ollama_to(&dest, &version, &on_progress, cancel).await?;
    super::ollama_setup_start::start_sidecar_and_wait(&app, &on_progress, cancel).await
}

#[tauri::command]
pub async fn cancel_ollama_setup() -> Result<(), String> {
    super::ollama_setup_cancel::cancel_active().await;
    Ok(())
}

#[tauri::command]
pub async fn restart_ollama_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    ollama_lifecycle::stop_sidecar(&app);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    ollama_lifecycle::start_sidecar(&app)
}

#[tauri::command]
pub async fn check_model_fits_vram(size_bytes: u64) -> bool {
    let vram_mb = crate::services::gpu_detect::detect_vram_mb().unwrap_or(0);
    if vram_mb == 0 {
        return true;
    }
    let model_mb = size_bytes / 1_048_576;
    model_mb < vram_mb
}

async fn resolve_install_version() -> String {
    use super::ollama_bundle_utils::is_valid_semver;

    match super::ollama_version::fetch_latest_github_version().await {
        Ok((version, _)) if is_valid_semver(&version) => version,
        Ok((version, _)) => {
            eprintln!("[ollama-setup] latest version invalid: {version}");
            fallback_ollama_version().to_string()
        }
        Err(e) => {
            eprintln!("[ollama-setup] latest version unavailable: {e}");
            fallback_ollama_version().to_string()
        }
    }
}
