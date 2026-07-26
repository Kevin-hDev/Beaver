use crate::services::agent_local::ollama_base_url;
use crate::services::ollama_ps;
use std::time::Duration;
use tauri::Emitter;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);
const FAILURES_BEFORE_RESTART: u32 = 3;
const MAX_RESTART_ATTEMPTS: u32 = 4;

fn backoff_secs(attempt: u32) -> u64 {
    match attempt {
        0 => 2,
        1 => 4,
        2 => 8,
        _ => 16,
    }
}

pub fn start(handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(HEALTH_TIMEOUT)
            .build()
            .unwrap_or_default();

        let mut last_running = false;
        let mut consecutive_failures: u32 = 0;
        let mut restart_attempts: u32 = 0;

        loop {
            let base = ollama_base_url();
            let running = client
                .get(format!("{base}/api/version"))
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            emit_gpu_status(&handle, &client, &base, running).await;

            if running {
                consecutive_failures = 0;
                restart_attempts = 0;
            } else {
                if crate::services::ollama_lifecycle::ollama_binary_path().is_err() {
                    tokio::time::sleep(POLL_INTERVAL).await;
                    continue;
                }

                consecutive_failures += 1;
                if consecutive_failures >= FAILURES_BEFORE_RESTART
                    && restart_attempts < MAX_RESTART_ATTEMPTS
                {
                    let delay = backoff_secs(restart_attempts);
                    eprintln!(
                        "[ollama] {} échecs, tentative {} dans {delay}s",
                        consecutive_failures,
                        restart_attempts + 1
                    );
                    tokio::time::sleep(Duration::from_secs(delay)).await;

                    match crate::services::ollama_lifecycle::start_sidecar(&handle) {
                        Ok(true) => eprintln!("[ollama] sidecar redémarré"),
                        Ok(false) => eprintln!("[ollama] daemon externe détecté"),
                        Err(e) => eprintln!("[ollama] restart échoué : {e}"),
                    }

                    restart_attempts += 1;
                    consecutive_failures = 0;
                }
            }

            if running != last_running {
                let _ = handle.emit("ollama-status", running);
                last_running = running;
            }

            tokio::time::sleep(POLL_INTERVAL).await;
        }
    });
}

async fn emit_gpu_status(
    handle: &tauri::AppHandle,
    client: &reqwest::Client,
    base: &str,
    ollama_running: bool,
) {
    let (vram_total, vram_used) = tokio::task::spawn_blocking(|| {
        use crate::services::gpu_vram;
        (
            gpu_vram::detect_vram_mb().unwrap_or(0),
            gpu_vram::detect_vram_used_mb().unwrap_or(0),
        )
    })
    .await
    .unwrap_or((0, 0));

    let empty = ollama_ps::PsResponse { models: vec![] };
    let ps = if ollama_running {
        let url = format!("{base}/api/ps");
        match client.get(&url).send().await {
            Ok(r) => r.json::<ollama_ps::PsResponse>().await.unwrap_or(empty),
            Err(_) => empty,
        }
    } else {
        empty
    };
    let status = ollama_ps::build_gpu_status(&ps, vram_total, vram_used);
    let _ = handle.emit("ollama-gpu-status", &status);
}
