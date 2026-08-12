use crate::app_exit::AppWorkSupervisor;
use crate::services::agent_local::{app_handle_global, types_tools::SearchResult};
use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkCancellation};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::sync::Mutex;

use super::work_supervision::{SearxngWorkServices, SERVER_PROCESSES};

const START_FAILURE_COOLDOWN: Duration = Duration::from_secs(30);
static LAST_START_FAILURE: StdMutex<Option<StartFailure>> = StdMutex::new(None);

#[derive(Clone)]
pub struct SearxngSidecar {
    process: Arc<Mutex<Option<SearxngHandle>>>,
    work: SearxngWorkServices,
}

struct SearxngHandle {
    child: tokio::process::Child,
    port: u16,
    _admission: ServiceWorkAdmission<SERVER_PROCESSES>,
}

struct StartFailure {
    at: Instant,
    message: String,
}

impl SearxngSidecar {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            work: SearxngWorkServices::new(app_work),
        }
    }

    async fn ensure_running(
        &self,
        app: &tauri::AppHandle,
        cancel: &ServiceWorkCancellation,
    ) -> Result<String, String> {
        let mut guard = self.process.lock().await;
        if let Some(handle) = guard.as_mut() {
            match handle.child.try_wait() {
                Ok(None) => return Ok(base_url(handle.port)),
                Ok(Some(_)) => *guard = None,
                Err(_) => return Err("SearXNG: état processus illisible".to_string()),
            }
        }
        if let Some(error) = recent_start_failure() {
            return Err(error);
        }

        run_blocking(super::process::kill_orphan_sidecar).await?;
        let source = super::paths::source_dir(app)?;
        let python = super::runtime::ensure_runtime(&source, cancel).await?;
        let port = super::settings::find_free_port()?;
        let settings = super::settings::write_settings(port)?;
        let admission = self
            .work
            .try_admit_server()
            .map_err(|_| "SearXNG: arrêt en cours".to_string())?;
        let mut child = super::process::spawn(&python, &source, &settings, port).await?;
        let pid = child
            .id()
            .ok_or_else(|| "SearXNG: démarrage impossible".to_string())?;
        super::process::save_pid(pid);
        let url = base_url(port);
        if let Err(error) = wait_until_ready(&url, &mut child, cancel).await {
            remember_start_failure(&error);
            super::process::kill_child_process(child).await;
            return Err(error);
        }
        ::log::info!("[searxng] sidecar démarré pid={pid} port={port}");
        clear_start_failure();
        *guard = Some(SearxngHandle {
            child,
            port,
            _admission: admission,
        });
        Ok(url)
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        if let Some(handle) = self.process.lock().await.take() {
            super::process::kill_child_process(handle.child).await;
        }
        self.work.stop_and_wait(deadline).await
    }

    #[cfg(test)]
    pub(crate) fn try_admit_start_for_test(&self) -> Result<(), ()> {
        let admission = self.work.try_admit_server().map_err(|_| ())?;
        drop(admission);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) async fn start_test_process_for_test(&self) -> Result<u32, String> {
        let admission = self
            .work
            .try_admit_server()
            .map_err(|_| "fixture SearXNG indisponible".to_string())?;
        let child = super::process::spawn_test_fixture().await?;
        let pid = child
            .id()
            .ok_or_else(|| "fixture SearXNG indisponible".to_string())?;
        *self.process.lock().await = Some(SearxngHandle {
            child,
            port: 0,
            _admission: admission,
        });
        Ok(pid)
    }
}

pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    let app = app_handle_global::get().ok_or_else(|| "SearXNG: app non initialisée".to_string())?;
    let state = app.state::<SearxngSidecar>().inner().clone();
    let run_state = state.clone();
    let run_app = app.clone();
    let base_url = state
        .work
        .run_start(move |cancel| async move { run_state.ensure_running(&run_app, &cancel).await })
        .await
        .map_err(|_| "SearXNG: arrêt en cours".to_string())??;
    super::client::search(&base_url, query).await
}

pub fn prepare_on_startup(app: tauri::AppHandle) {
    let Some(state) = app.try_state::<SearxngSidecar>() else {
        return;
    };
    let state = state.inner().clone();
    let run_state = state.clone();
    let _ = state.work.spawn_start(move |cancel| async move {
        if let Err(error) = run_state.ensure_running(&app, &cancel).await {
            ::log::warn!("[searxng] warmup failed: {}", safe_log_error(&error));
        }
    });
}

async fn wait_until_ready(
    base_url: &str,
    child: &mut tokio::process::Child,
    cancel: &ServiceWorkCancellation,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{base_url}/healthz");
    for _ in 0..40 {
        if cancel.is_cancelled() {
            return Err("SearXNG: arrêt en cours".to_string());
        }
        if let Ok(Some(status)) = child.try_wait() {
            let hint = super::process::startup_log_hint()
                .map(|hint| format!(" ({hint})"))
                .unwrap_or_default();
            return Err(format!("SearXNG: arrêt au démarrage {status}{hint}"));
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(250)) => {}
            _ = cancel.cancelled() => return Err("SearXNG: arrêt en cours".to_string()),
        }
        if let Ok(response) = client
            .get(&url)
            .timeout(Duration::from_millis(500))
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(());
            }
        }
    }
    Err("SearXNG: timeout au démarrage".to_string())
}

async fn run_blocking<Operation>(operation: Operation) -> Result<(), String>
where
    Operation: FnOnce() + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| "SearXNG: opération interrompue".to_string())
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

pub(super) fn recent_start_failure() -> Option<String> {
    let guard = LAST_START_FAILURE.lock().ok()?;
    let failure = guard.as_ref()?;
    (failure.at.elapsed() < START_FAILURE_COOLDOWN).then(|| failure.message.clone())
}

pub(super) fn remember_start_failure(error: &str) {
    if let Ok(mut guard) = LAST_START_FAILURE.lock() {
        *guard = Some(StartFailure {
            at: Instant::now(),
            message: error.to_string(),
        });
    }
}

pub(super) fn clear_start_failure() {
    if let Ok(mut guard) = LAST_START_FAILURE.lock() {
        *guard = None;
    }
}

pub(super) fn safe_log_error(error: &str) -> String {
    error
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .take(240)
        .collect::<String>()
        .trim()
        .to_string()
}
