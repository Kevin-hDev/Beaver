use crate::app_exit::AppWorkSupervisor;
use crate::services::agent_local::{app_handle_global, types_tools::SearchResult};
use crate::services::work_registry::{ServiceWorkAdmission, ServiceWorkCancellation};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;
use tauri::Manager;
use tokio::sync::Mutex;

use super::work_supervision::{SearxngWorkServices, SERVER_PROCESSES};

#[derive(Clone)]
pub struct SearxngSidecar {
    process: Arc<Mutex<Option<SearxngHandle>>>,
    start_gate: Arc<Mutex<()>>,
    publication_generation: Arc<AtomicU64>,
    work: SearxngWorkServices,
}

struct SearxngHandle {
    child: tokio::process::Child,
    port: u16,
    _admission: ServiceWorkAdmission<SERVER_PROCESSES>,
}

impl SearxngSidecar {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            start_gate: Arc::new(Mutex::new(())),
            publication_generation: Arc::new(AtomicU64::new(1)),
            work: SearxngWorkServices::new(app_work),
        }
    }

    async fn ensure_running(
        &self,
        app: &tauri::AppHandle,
        cancel: &ServiceWorkCancellation,
    ) -> Result<String, String> {
        let _start = tokio::select! {
            guard = self.start_gate.lock() => guard,
            _ = cancel.cancelled() => return Err(shutdown_error()),
        };
        let generation = self.publication_generation.load(Ordering::Acquire);
        {
            let mut guard = self.process.lock().await;
            if let Some(handle) = guard.as_mut() {
                match handle.child.try_wait() {
                    Ok(None) => return Ok(base_url(handle.port)),
                    Ok(Some(_)) => *guard = None,
                    Err(_) => return Err("SearXNG: état processus illisible".to_string()),
                }
            }
        }
        ensure_start_active(self, cancel, generation)?;
        if let Some(error) = super::startup_failure::recent() {
            return Err(error);
        }

        super::startup::run_blocking(super::process::kill_orphan_sidecar).await?;
        ensure_start_active(self, cancel, generation)?;
        let source = super::paths::source_dir(app)?;
        let python = super::runtime::ensure_runtime(&source, cancel).await?;
        ensure_start_active(self, cancel, generation)?;
        let port = super::settings::find_free_port()?;
        let settings = super::settings::write_settings(port)?;
        let admission = self.work.try_admit_server().map_err(|_| shutdown_error())?;
        let mut child = super::process::spawn(&python, &source, &settings, port).await?;
        let pid = child
            .id()
            .ok_or_else(|| "SearXNG: démarrage impossible".to_string())?;
        let url = base_url(port);
        if let Err(error) = super::startup::wait_until_ready(&url, &mut child, cancel).await {
            super::startup_failure::remember(&error);
            super::process::kill_child_process(child).await;
            return Err(error);
        }
        if ensure_start_active(self, cancel, generation).is_err() {
            super::process::kill_child_process(child).await;
            return Err(shutdown_error());
        }
        let mut guard = self.process.lock().await;
        if ensure_start_active(self, cancel, generation).is_err() || guard.is_some() {
            drop(guard);
            super::process::kill_child_process(child).await;
            return Err(shutdown_error());
        }
        super::process::save_pid(pid);
        ::log::info!("[searxng] sidecar démarré pid={pid} port={port}");
        super::startup_failure::clear();
        *guard = Some(SearxngHandle {
            child,
            port,
            _admission: admission,
        });
        Ok(url)
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.work.begin_closing();
        self.publication_generation.fetch_add(1, Ordering::AcqRel);
        let process_stopped = stop_published_process(self, deadline).await;
        self.work.stop_and_wait(deadline).await && process_stopped
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

    #[cfg(test)]
    pub(crate) async fn suspend_test_start_before_publication_for_test(
        &self,
        started: tokio::sync::oneshot::Sender<u32>,
        release: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), String> {
        let run_state = self.clone();
        self.work
            .run_start(move |_cancel| async move {
                let _start = run_state.start_gate.lock().await;
                let child = super::process::spawn_test_fixture().await?;
                let pid = child
                    .id()
                    .ok_or_else(|| "fixture SearXNG indisponible".to_string())?;
                let _ = started.send(pid);
                let _ = release.await;
                drop(child);
                Ok(())
            })
            .await
            .map_err(|_| "fixture SearXNG interrompue".to_string())?
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
            ::log::warn!(
                "[searxng] warmup failed: {}",
                super::startup_failure::safe_log_error(&error)
            );
        }
    });
}

fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

fn ensure_start_active(
    sidecar: &SearxngSidecar,
    cancel: &ServiceWorkCancellation,
    generation: u64,
) -> Result<(), String> {
    if cancel.is_cancelled() || sidecar.publication_generation.load(Ordering::Acquire) != generation
    {
        return Err(shutdown_error());
    }
    Ok(())
}

async fn stop_published_process(sidecar: &SearxngSidecar, deadline: Instant) -> bool {
    let Ok(mut process) = tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        sidecar.process.lock(),
    )
    .await
    else {
        return false;
    };
    let handle = process.take();
    drop(process);
    let Some(handle) = handle else { return true };
    tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        super::process::kill_child_process(handle.child),
    )
    .await
    .is_ok()
}

fn shutdown_error() -> String {
    "SearXNG: arrêt en cours".to_string()
}
