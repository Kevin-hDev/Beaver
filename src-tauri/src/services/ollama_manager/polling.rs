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
    pub vram_used_mb: Option<u64>,
    pub vram_total_mb: u64,
    pub model_loaded: Option<String>,
}

pub(crate) fn build_gpu_status(
    ps: &PsResponse,
    snapshot: Option<crate::services::gpu_vram::GpuMemorySnapshot>,
    active_compute_mode: Option<super::compute_mode::OllamaComputeMode>,
    system_ram_total_mb: u64,
) -> GpuStatusPayload {
    let cpu_model_bytes = ps.models.iter().fold(0_u64, |total, model| {
        total.saturating_add(model.size.saturating_sub(model.size_vram))
    });
    let model_uses_gpu = ps.models.iter().any(|model| model.size_vram > 0);
    let cpu_runtime = !ps.models.is_empty() && !model_uses_gpu
        || ps.models.is_empty()
            && matches!(
                active_compute_mode,
                Some(super::compute_mode::OllamaComputeMode::Cpu)
            );
    let (accelerator, vram_total_mb, vram_used_mb) = if cpu_runtime {
        (
            "CPU · RAM",
            system_ram_total_mb,
            Some(cpu_model_bytes / 1_048_576),
        )
    } else {
        match snapshot {
            Some(snapshot) => (
                match snapshot.kind {
                    crate::services::gpu_vram::GpuMemoryKind::Dedicated => "VRAM",
                    crate::services::gpu_vram::GpuMemoryKind::Unified => "RAM",
                    crate::services::gpu_vram::GpuMemoryKind::Unknown => "",
                },
                snapshot.total_mb,
                snapshot.used_mb,
            ),
            None if model_uses_gpu => ("VRAM", 0, None),
            None => ("", 0, None),
        }
    };
    GpuStatusPayload {
        accelerator: accelerator.into(),
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
        let snapshot = crate::services::gpu_vram::current_snapshot();
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
        let _ = app.emit(
            "ollama-gpu-status",
            &build_gpu_status(
                &ps,
                snapshot,
                self.active_compute_mode(),
                system_ram_total_mb(),
            ),
        );
    }
}

fn system_ram_total_mb() -> u64 {
    static TOTAL_MB: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *TOTAL_MB.get_or_init(|| {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        system.total_memory() / 1_048_576
    })
}
