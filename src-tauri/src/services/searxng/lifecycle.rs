use crate::services::agent_local::{app_handle_global, types_tools::SearchResult};
use std::process::Child;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::Mutex;

const START_FAILURE_COOLDOWN: Duration = Duration::from_secs(30);
const PROCESS_TASK_ERROR: &str = "SearXNG: nettoyage processus interrompu";

static LAST_START_FAILURE: StdMutex<Option<StartFailure>> = StdMutex::new(None);

pub struct SearxngSidecar(pub Arc<Mutex<Option<SearxngHandle>>>);

pub struct SearxngHandle {
    child: Child,
    port: u16,
}

struct StartFailure {
    at: Instant,
    message: String,
}

impl SearxngSidecar {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let app = app_handle_global::get().ok_or_else(|| "SearXNG: app non initialisée".to_string())?;
    let base_url = ensure_running(app).await?;
    super::client::search(&base_url, query).await
}

pub fn prepare_on_startup(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = ensure_running(&app).await {
            eprintln!("[searxng] warmup failed: {}", safe_log_error(&e));
        }
    });
}

async fn ensure_running(app: &tauri::AppHandle) -> Result<String, String> {
    let state = app.state::<SearxngSidecar>();
    let mut guard = state.0.clone().lock_owned().await;
    if let Some(handle) = guard.as_mut() {
        match handle.child.try_wait() {
            Ok(None) => return Ok(base_url(handle.port)),
            Ok(Some(_)) => {
                *guard = None;
            }
            Err(_) => return Err("SearXNG: état processus illisible".to_string()),
        }
    }

    if let Some(error) = recent_start_failure() {
        return Err(error);
    }

    let mut guard = run_blocking_process_operation(move || {
        super::process::kill_orphan_sidecar();
        guard
    })
    .await?;
    let source = super::paths::source_dir(app)?;
    let python = super::runtime::ensure_runtime(&source).await?;
    let port = super::settings::find_free_port()?;
    let settings = super::settings::write_settings(port)?;
    let mut child = super::process::spawn(&python, &source, &settings, port)?;
    let pid = child.id();
    super::process::save_pid(pid);
    let url = base_url(port);
    if let Err(e) = wait_until_ready(&url, &mut child).await {
        remember_start_failure(&e);
        run_blocking_process_operation(move || {
            super::process::kill_child_process(child);
            drop(guard);
        })
        .await?;
        return Err(e);
    }
    eprintln!("[searxng] sidecar démarré pid={pid} port={port}");
    clear_start_failure();
    *guard = Some(SearxngHandle { child, port });
    Ok(url)
}

pub async fn stop(sidecar: &SearxngSidecar) {
    let mut guard = sidecar.0.clone().lock_owned().await;
    if let Some(handle) = guard.take() {
        // The task owns the guard so cancellation cannot expose a stale PID cleanup race.
        if let Err(error) = run_blocking_process_operation(move || {
            super::process::kill_child_process(handle.child);
            drop(guard);
        })
        .await
        {
            eprintln!("[searxng] {}", safe_log_error(&error));
        }
    }
}

async fn run_blocking_process_operation<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| PROCESS_TASK_ERROR.to_string())
}

async fn wait_until_ready(base_url: &str, child: &mut Child) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/healthz", base_url);
    for _ in 0..40 {
        if let Ok(Some(status)) = child.try_wait() {
            let hint = super::process::startup_log_hint()
                .map(|hint| format!(" ({hint})"))
                .unwrap_or_default();
            return Err(format!("SearXNG: arrêt au démarrage {status}{hint}"));
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        if let Ok(resp) = client
            .get(&url)
            .timeout(std::time::Duration::from_millis(500))
            .send()
            .await
        {
            if resp.status().is_success() {
                return Ok(());
            }
        }
    }
    Err("SearXNG: timeout au démarrage".to_string())
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

fn recent_start_failure() -> Option<String> {
    let guard = LAST_START_FAILURE.lock().ok()?;
    let failure = guard.as_ref()?;
    (failure.at.elapsed() < START_FAILURE_COOLDOWN).then(|| failure.message.clone())
}

fn remember_start_failure(error: &str) {
    if let Ok(mut guard) = LAST_START_FAILURE.lock() {
        *guard = Some(StartFailure {
            at: Instant::now(),
            message: error.to_string(),
        });
    }
}

fn clear_start_failure() {
    if let Ok(mut guard) = LAST_START_FAILURE.lock() {
        *guard = None;
    }
}

fn safe_log_error(error: &str) -> String {
    let cleaned: String = error
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .take(240)
        .collect();
    cleaned.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_log_error_removes_control_chars_and_truncates() {
        let input = format!("SearXNG: timeout\n{}", "x".repeat(400));
        let output = safe_log_error(&input);
        assert!(!output.contains('\n'));
        assert!(output.chars().count() <= 240);
    }

    #[test]
    fn start_failure_cache_expires() {
        clear_start_failure();
        remember_start_failure("SearXNG: arrêt au démarrage");
        assert_eq!(
            recent_start_failure(),
            Some("SearXNG: arrêt au démarrage".to_string())
        );
        clear_start_failure();
        assert_eq!(recent_start_failure(), None);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn process_operation_runs_off_async_thread() {
        let async_thread = std::thread::current().id();
        let blocking_thread = run_blocking_process_operation(|| std::thread::current().id())
            .await
            .unwrap();

        assert_ne!(blocking_thread, async_thread);
    }

    #[tokio::test]
    async fn stop_without_handle_preserves_tracked_pid() {
        let pid_path = crate::services::paths::data_dir().join("searxng-sidecar.pid");
        std::fs::write(&pid_path, "424242").unwrap();

        stop(&SearxngSidecar::new()).await;

        assert_eq!(std::fs::read_to_string(&pid_path).unwrap(), "424242");
        std::fs::remove_file(pid_path).unwrap();
    }
}
