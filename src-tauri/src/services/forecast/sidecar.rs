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
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use zeroize::Zeroizing;

pub use super::sidecar_idle::schedule_idle_stop;
pub use super::sidecar_stop::{stop, stop_model};

pub(super) struct SidecarHandle {
    pub(super) child: tokio::process::Child,
    pub(super) pid: u32,
    pub(super) model_id: String,
    pub(super) family_id: String,
    pub(super) auth_token: Zeroizing<String>,
    pub(super) launch: LaunchSettings,
    pub(super) generation: u64,
    pub(super) publication_generation: u64,
    pub(super) _admission: SidecarAdmission,
}

#[derive(Clone)]
pub struct ChronosSidecar {
    pub(super) process: Arc<Mutex<Option<SidecarHandle>>>,
    prediction: Arc<Mutex<()>>,
    pub(super) work: ForecastWorkServices,
    pub(super) idle_changed: Arc<Notify>,
    pub(super) idle_started: Arc<AtomicBool>,
    // Cette génération reste stable pendant une sonde, contrairement au compteur d'inactivité.
    pub(super) next_publication_generation: Arc<AtomicU64>,
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
            next_publication_generation: Arc::new(AtomicU64::new(1)),
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

pub async fn start(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
) -> Result<SidecarEndpoint, String> {
    let launch = sidecar_settings::current();
    if let Some(endpoint) =
        super::sidecar_reuse::reuse_running(sidecar, model_name, family_id, &launch).await
    {
        return Ok(endpoint);
    }

    if !stop(sidecar).await {
        return Err("Sidecar Forecast indisponible".to_string());
    }
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
    )
    .await?;

    let pid = child.pid();
    let mut process = sidecar.process.lock().await;
    *process = Some(SidecarHandle {
        child: child.publish(),
        pid,
        model_id: model_name.to_string(),
        family_id: family_id.to_string(),
        auth_token: auth_token.clone(),
        launch,
        generation: 1,
        publication_generation: sidecar
            .next_publication_generation
            .fetch_add(1, Ordering::Relaxed),
        _admission: admission,
    });
    drop(process);
    sidecar_process::save_pid(pid);
    sidecar_http::set_port(port);
    sidecar.idle_changed.notify_waiters();

    match sidecar_spawn::wait_until_ready(port, model_name, family_id, pid, auth_token).await {
        Ok(endpoint) => Ok(endpoint),
        Err(error) => {
            let _ = stop(sidecar).await;
            Err(error)
        }
    }
}
