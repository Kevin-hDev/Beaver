use crate::services::agent_local::{app_handle_global, types_tools::SearchResult};
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::Mutex;

const START_FAILURE_COOLDOWN: Duration = Duration::from_secs(30);

static LAST_START_FAILURE: StdMutex<Option<StartFailure>> = StdMutex::new(None);

/// Best-effort flag set when a SearXNG sidecar has successfully started and not
/// yet been detected as dead. Read synchronously by the prompt assembler.
static SEARXNG_READY: AtomicBool = AtomicBool::new(false);

/// Synchronous, non-blocking read of the last known SearXNG runtime state.
/// True means a sidecar started successfully and has not been observed dead.
pub fn is_ready() -> bool {
    SEARXNG_READY.load(Ordering::Relaxed)
}

fn set_ready(value: bool) {
    SEARXNG_READY.store(value, Ordering::Relaxed);
}

pub struct SearxngSidecar(pub Mutex<Option<SearxngHandle>>);

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
        Self(Mutex::new(None))
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
    let mut guard = state.0.lock().await;
    if let Some(handle) = guard.as_mut() {
        match handle.child.try_wait() {
            Ok(None) => return Ok(base_url(handle.port)),
            Ok(Some(_)) => {
                set_ready(false);
                *guard = None;
            }
            Err(_) => return Err("SearXNG: état processus illisible".to_string()),
        }
    }

    if let Some(error) = recent_start_failure() {
        return Err(error);
    }

    super::process::kill_orphan_sidecar();
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
        super::process::kill_child_process(child);
        return Err(e);
    }
    eprintln!("[searxng] sidecar démarré pid={pid} port={port}");
    clear_start_failure();
    set_ready(true);
    *guard = Some(SearxngHandle { child, port });
    Ok(url)
}

pub async fn stop(sidecar: &SearxngSidecar) {
    let mut guard = sidecar.0.lock().await;
    if let Some(handle) = guard.take() {
        super::process::kill_child_process(handle.child);
    }
    set_ready(false);
    super::process::clear_pid_file();
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
    set_ready(false);
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

    /// Mutex serializing tests that mutate the process-global
    /// `LAST_START_FAILURE` and `SEARXNG_READY` flags. Without it, the
    /// default parallel test runner races on these globals.
    static GLOBAL_STATE_LOCK: StdMutex<()> = StdMutex::new(());

    #[test]
    fn safe_log_error_removes_control_chars_and_truncates() {
        let input = format!("SearXNG: timeout\n{}", "x".repeat(400));
        let output = safe_log_error(&input);
        assert!(!output.contains('\n'));
        assert!(output.chars().count() <= 240);
    }

    #[test]
    fn start_failure_cache_expires() {
        let _guard = GLOBAL_STATE_LOCK.lock().unwrap();
        clear_start_failure();
        remember_start_failure("SearXNG: arrêt au démarrage");
        assert_eq!(
            recent_start_failure(),
            Some("SearXNG: arrêt au démarrage".to_string())
        );
        clear_start_failure();
        assert_eq!(recent_start_failure(), None);
    }

    /// `is_ready` reads the global flag and `remember_start_failure` clears
    /// it. Serialized against `start_failure_cache_expires` to avoid races.
    #[test]
    fn is_ready_reads_flag_and_failure_clears_it() {
        let _guard = GLOBAL_STATE_LOCK.lock().unwrap();
        clear_start_failure();
        set_ready(false);

        set_ready(true);
        assert!(is_ready());

        remember_start_failure("SearXNG: timeout au démarrage");
        assert!(!is_ready());

        // Restore baseline for any test reading the flag afterwards.
        clear_start_failure();
        set_ready(false);
    }
}
