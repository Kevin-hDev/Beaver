use crate::app_exit::AppWorkSupervisor;
use crate::services::agent_local::{app_handle_global, types_tools::SearchResult};
use crate::services::work_registry::ServiceWorkAdmission;
use std::sync::{atomic::AtomicU64, Arc};
use tauri::Manager;
use tokio::sync::Mutex;

use super::work_supervision::{SearxngWorkServices, SERVER_PROCESSES};

#[derive(Clone)]
pub struct SearxngSidecar {
    pub(super) process: Arc<Mutex<Option<SearxngHandle>>>,
    pub(super) start_gate: Arc<Mutex<()>>,
    pub(super) publication_generation: Arc<AtomicU64>,
    pub(super) work: SearxngWorkServices,
}

pub(super) struct SearxngHandle {
    pub(super) child: tokio::process::Child,
    pub(super) port: u16,
    pub(super) _admission: ServiceWorkAdmission<SERVER_PROCESSES>,
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

pub(super) fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

pub(super) fn shutdown_error() -> String {
    "SearXNG: arrêt en cours".to_string()
}
