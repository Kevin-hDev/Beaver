use crate::app_exit::AppWorkSupervisor;
use crate::services::forecast::{
    sidecar_auth, sidecar_http, sidecar_process,
    sidecar_settings::{self, LaunchSettings},
    sidecar_spawn,
    work_supervision::{ForecastWorkServices, SidecarAdmission},
};
use crate::services::paths::data_dir;
use crate::services::work_registry::ServiceWorkCancellation;
use std::future::Future;
use std::process::Child;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

pub use super::sidecar_idle::schedule_idle_stop;
pub use super::sidecar_stop::{stop, stop_model};

pub(super) struct SidecarHandle {
    pub(super) child: Child,
    pub(super) model_id: String,
    pub(super) family_id: String,
    pub(super) auth_token: Zeroizing<String>,
    pub(super) launch: LaunchSettings,
    pub(super) generation: u64,
    pub(super) _admission: SidecarAdmission,
}

#[derive(Clone)]
pub struct ChronosSidecar {
    pub(super) process: Arc<Mutex<Option<SidecarHandle>>>,
    prediction: Arc<Mutex<()>>,
    pub(super) work: ForecastWorkServices,
    pub(super) idle_changed: Arc<Notify>,
    pub(super) idle_started: Arc<AtomicBool>,
}

pub struct SidecarEndpoint {
    pub base_url: String,
    pub auth_token: Zeroizing<String>,
    pub pid: u32,
}

impl ChronosSidecar {
    pub fn new(app_work: AppWorkSupervisor) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            prediction: Arc::new(Mutex::new(())),
            work: ForecastWorkServices::new(app_work),
            idle_changed: Arc::new(Notify::new()),
            idle_started: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn lock_prediction(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.prediction.lock().await
    }

    pub async fn run_operation<Factory, Task, Output>(
        &self,
        work: Factory,
    ) -> Result<Output, String>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: Future<Output = Result<Output, String>> + Send + 'static,
        Output: Send + 'static,
    {
        self.work.run_operation(work).await
    }

    pub async fn run_cancellable<Factory, Task, Output>(
        &self,
        work: Factory,
    ) -> Result<Output, String>
    where
        Factory: FnOnce() -> Task + Send + 'static,
        Task: Future<Output = Result<Output, String>> + Send + 'static,
        Output: Send + 'static,
    {
        self.run_operation(move |cancel| async move {
            let task = work();
            tokio::pin!(task);
            tokio::select! {
                result = &mut task => result,
                _ = cancel.cancelled() => Err("app-shutting-down".to_string()),
            }
        })
        .await
    }

    pub async fn run_with_cancel<Factory, Task, Output>(
        &self,
        cancel: CancellationToken,
        work: Factory,
    ) -> Result<Output, String>
    where
        Factory: FnOnce(CancellationToken) -> Task + Send + 'static,
        Task: Future<Output = Result<Output, String>> + Send + 'static,
        Output: Send + 'static,
    {
        self.run_operation(move |shutdown| async move {
            let shutdown_cancel = cancel.clone();
            let task = work(cancel);
            tokio::pin!(task);
            tokio::select! {
                result = &mut task => result,
                _ = shutdown.cancelled() => {
                    shutdown_cancel.cancel();
                    task.await
                }
            }
        })
        .await
    }
}

pub fn get_port() -> u16 {
    sidecar_http::get_port()
}

pub fn base_url() -> String {
    sidecar_http::base_url()
}

pub async fn start(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
) -> Result<SidecarEndpoint, String> {
    let launch = sidecar_settings::current();
    if let Some(endpoint) = reuse_running(sidecar, model_name, family_id, &launch).await {
        return Ok(endpoint);
    }

    stop(sidecar).await;
    sidecar_process::kill_orphan_sidecar();
    let port = sidecar_http::find_free_port();
    let script = sidecar_spawn::sidecar_dir().join("server.py");
    if !script.exists() {
        return Err("Sidecar Python non installé".into());
    }
    let runtime_python = sidecar_spawn::ready_runtime(family_id)?;
    let models_dir = data_dir().join("forecast-models");
    let auth_token = sidecar_auth::generate_auth_token();
    let admission = sidecar
        .work
        .try_admit_sidecar()
        .map_err(|error| error.public_code().to_string())?;
    let child = sidecar_spawn::spawn_process(
        runtime_python,
        &script,
        port,
        model_name,
        family_id,
        &models_dir,
        &auth_token,
        &launch,
    )?;

    let pid = child.id();
    sidecar_process::save_pid(pid);
    sidecar_http::set_port(port);
    *sidecar.process.lock().await = Some(SidecarHandle {
        child,
        model_id: model_name.to_string(),
        family_id: family_id.to_string(),
        auth_token: auth_token.clone(),
        launch,
        generation: 1,
        _admission: admission,
    });
    sidecar.idle_changed.notify_waiters();

    match sidecar_spawn::wait_until_ready(port, model_name, family_id, pid, auth_token).await {
        Ok(endpoint) => Ok(endpoint),
        Err(error) => {
            stop(sidecar).await;
            Err(error)
        }
    }
}

async fn reuse_running(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
    launch: &LaunchSettings,
) -> Option<SidecarEndpoint> {
    let mut guard = sidecar.process.lock().await;
    let handle = guard.as_mut()?;
    if handle.model_id != model_name || handle.family_id != family_id || &handle.launch != launch {
        return None;
    }
    let (_, model, family) = sidecar_http::health_info(get_port(), handle.auth_token.as_str())?;
    if model != model_name || family != family_id {
        return None;
    }
    handle.generation = handle.generation.saturating_add(1);
    Some(SidecarEndpoint {
        base_url: base_url(),
        auth_token: handle.auth_token.clone(),
        pid: handle.child.id(),
    })
}
