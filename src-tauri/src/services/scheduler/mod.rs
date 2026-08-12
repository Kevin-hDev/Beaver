mod agentic;
pub mod due;
pub mod fire;
mod fire_once;
pub mod log;
pub mod next_fire;
#[cfg(test)]
mod next_fire_tests;
mod runtime;
mod runtime_decisions;
#[cfg(test)]
mod runtime_tests;
pub mod state;
#[cfg(test)]
#[path = "task_tests.rs"]
mod task_tests;
mod work_supervision;

use crate::app_exit::AppWorkSupervisor;
#[cfg(test)]
use crate::services::work_registry::{ServiceWorkAdmissionError, ServiceWorkCancellation};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tauri::AppHandle;
use tokio::sync::watch;
pub use work_supervision::SchedulerDiagnostics;
use work_supervision::SchedulerWorkServices;

static RELOAD_SENDER: OnceLock<Mutex<Option<watch::Sender<u64>>>> = OnceLock::new();

pub struct Scheduler {
    reload_tx: watch::Sender<u64>,
    work: SchedulerWorkServices,
}

impl Scheduler {
    pub fn spawn(app: AppHandle, app_work: AppWorkSupervisor) -> Result<Self, String> {
        let (reload_tx, reload_rx) = watch::channel(0u64);
        let work = SchedulerWorkServices::new(app_work);
        let wakeups = work.wakeups();
        work.start_loop(move |cancel| runtime::run_loop(app, reload_rx, cancel, wakeups))
            .map_err(|error| error.public_code().to_string())?;
        let sender = RELOAD_SENDER.get_or_init(|| Mutex::new(None));
        *sender.lock().unwrap_or_else(|error| error.into_inner()) = Some(reload_tx.clone());
        Ok(Scheduler { reload_tx, work })
    }

    pub fn notify_config_changed(&self) {
        let next = self.reload_tx.borrow().wrapping_add(1);
        let _ = self.reload_tx.send(next);
    }

    pub fn diagnostics(&self) -> SchedulerDiagnostics {
        self.work.diagnostics()
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        let stopped = self.work.stop_and_wait(deadline).await;
        if !stopped {
            let diagnostics = self.diagnostics();
            ::log::warn!(
                "[scheduler] arrêt incomplet: boucle={}, réveils={}",
                diagnostics.loop_work.active,
                diagnostics.wakeups.active
            );
        }
        stopped
    }

    #[cfg(test)]
    fn for_test(app_work: AppWorkSupervisor) -> Self {
        let (reload_tx, _) = watch::channel(0u64);
        Self {
            reload_tx,
            work: SchedulerWorkServices::new(app_work),
        }
    }

    #[cfg(test)]
    fn spawn_wakeup_for_test<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: std::future::Future + Send + 'static,
    {
        self.work.spawn_wakeup(work)
    }

    #[cfg(test)]
    fn spawn_loop_for_test<Factory, Task>(
        &self,
        work: Factory,
    ) -> Result<(), ServiceWorkAdmissionError>
    where
        Factory: FnOnce(ServiceWorkCancellation) -> Task + Send + 'static,
        Task: std::future::Future + Send + 'static,
    {
        self.work.start_loop(work)
    }
}

pub fn notify_config_changed() {
    let Some(sender) = RELOAD_SENDER.get() else {
        return;
    };
    if let Some(sender) = sender
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
    {
        let next = sender.borrow().wrapping_add(1);
        let _ = sender.send(next);
    }
}
