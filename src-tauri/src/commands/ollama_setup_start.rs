use std::time::{Duration, Instant};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::ollama_setup::OllamaSetupProgress;
use crate::services::ollama_lifecycle;

const OLLAMA_STARTUP_TIMEOUT: Duration = Duration::from_secs(45);
const OLLAMA_STARTUP_POLL: Duration = Duration::from_millis(500);

pub(crate) async fn start_sidecar_and_wait(
    app: &tauri::AppHandle,
    on_progress: &Channel<OllamaSetupProgress>,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err(super::ollama_setup_cancel::cancelled_error());
    }
    let _ = on_progress.send(OllamaSetupProgress {
        completed: 0,
        total: 0,
        status: "starting".into(),
    });
    ollama_lifecycle::start_sidecar(app).map_err(|e| {
        eprintln!("[ollama-setup] start: {e}");
        "ollama-start-error".to_string()
    })?;
    wait_until_ollama_ready(cancel).await
}

async fn wait_until_ollama_ready(cancel: &CancellationToken) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|_| "ollama-start-error".to_string())?;
    let started_at = Instant::now();

    while started_at.elapsed() < OLLAMA_STARTUP_TIMEOUT {
        if cancel.is_cancelled() {
            return Err(super::ollama_setup_cancel::cancelled_error());
        }
        let url = format!("{}/api/version", crate::services::ollama_port::base_url());
        if client
            .get(url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return Ok(());
        }
        tokio::select! {
            _ = cancel.cancelled() => return Err(super::ollama_setup_cancel::cancelled_error()),
            _ = tokio::time::sleep(OLLAMA_STARTUP_POLL) => {}
        }
    }

    Err("ollama-start-timeout".to_string())
}
