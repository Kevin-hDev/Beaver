use super::manager::OllamaManager;
use super::retry::RetryWait;
use super::types::{DaemonState, OllamaRuntimeStatus};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PsModel {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub size_vram: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PsResponse {
    #[serde(default)]
    pub models: Vec<PsModel>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GpuStatusPayload {
    pub accelerator: String,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub model_loaded: Option<String>,
}

pub(crate) fn build_gpu_status(
    ps: &PsResponse,
    vram_total_mb: u64,
    vram_used_mb: u64,
) -> GpuStatusPayload {
    let has_gpu_signal =
        ps.models.iter().any(|model| model.size_vram > 0) || vram_total_mb > 0 || vram_used_mb > 0;
    GpuStatusPayload {
        accelerator: if has_gpu_signal { "GPU" } else { "CPU" }.into(),
        vram_used_mb,
        vram_total_mb,
        model_loaded: ps.models.first().map(|model| model.name.clone()),
    }
}

impl OllamaManager {
    pub async fn poll_once(&self) -> OllamaRuntimeStatus {
        let barrier = self.wait_startup_decision().await;
        if !matches!(barrier, super::startup::StartupBarrierState::Ready) {
            return self.status().await;
        }
        let current = self.status().await;
        let endpoint = match &current.daemon {
            DaemonState::Owned { endpoint } | DaemonState::External { endpoint } => endpoint,
            DaemonState::Unavailable => return current,
        };
        let client = reqwest::Client::builder()
            .timeout(HEALTH_TIMEOUT)
            .build()
            .unwrap_or_default();
        let running = client
            .get(format!("{}/api/version", endpoint.as_http_url()))
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false);
        let daemon = if running {
            current.daemon
        } else {
            DaemonState::Unavailable
        };
        self.publish_daemon(daemon);
        self.status().await
    }

    pub async fn run_background_loop(&self, cancellation: CancellationToken) {
        let handle = crate::services::agent_local::app_handle_global::get().cloned();
        let retry = self.retry_handle();
        let mut last_running = false;
        loop {
            if cancellation.is_cancelled() || self.is_closing() {
                return;
            }
            match self.startup_state() {
                super::startup::StartupBarrierState::Pending => {
                    tokio::select! {
                        _ = cancellation.cancelled() => return,
                        _ = self.wait_startup_decision() => {}
                    }
                    continue;
                }
                super::startup::StartupBarrierState::Blocked { .. } => {
                    match retry.wait(&cancellation).await {
                        RetryWait::Due => {
                            let _ = self.run_startup_recovery().await;
                        }
                        RetryWait::Cancelled | RetryWait::Closing => return,
                    }
                    continue;
                }
                super::startup::StartupBarrierState::Ready => {}
            }
            let status = self.poll_once().await;
            let running = !matches!(status.daemon, DaemonState::Unavailable);
            if let Some(app) = &handle {
                if running != last_running {
                    let _ = app.emit("ollama-status", running);
                    last_running = running;
                }
                self.emit_gpu_status(app, &status, &cancellation).await;
            }
            tokio::select! {
                _ = cancellation.cancelled() => return,
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
            }
        }
    }

    async fn emit_gpu_status(
        &self,
        app: &tauri::AppHandle,
        status: &OllamaRuntimeStatus,
        cancellation: &CancellationToken,
    ) {
        let total = crate::services::gpu_vram::detect_vram_mb().unwrap_or(0);
        let used = crate::services::gpu_vram::detect_vram_used_mb().unwrap_or(0);
        let ps = if let DaemonState::Owned { endpoint } | DaemonState::External { endpoint } =
            &status.daemon
        {
            let client = reqwest::Client::builder()
                .timeout(HEALTH_TIMEOUT)
                .build()
                .unwrap_or_default();
            tokio::select! {
                _ = cancellation.cancelled() => return,
                response = client.get(format!(
                    "{}/api/ps",
                    endpoint.as_http_url()
                )).send() => match response {
                    Ok(response) => response.json().await.unwrap_or(PsResponse { models: vec![] }),
                    Err(_) => PsResponse { models: vec![] },
                }
            }
        } else {
            PsResponse { models: vec![] }
        };
        let _ = app.emit("ollama-gpu-status", &build_gpu_status(&ps, total, used));
    }
}
