use super::{limits::*, types::*, worker::InstallExecutor};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Job {
    pub view: InstallJobView,
    pub request: InstallRequest,
    pub key: String,
    #[serde(skip)]
    pub cancel: CancellationToken,
    #[serde(default)]
    pub checkpoint: Option<super::checkpoint::InstallCheckpoint>,
    #[serde(skip)]
    pub claimed_cleanup: bool,
    #[serde(skip)]
    pub monitor: super::disk_control::DiskMonitor,
    // Only the executor may attest that owned artifacts are gone.
    pub clean: bool,
    pub finished_revision: Option<u64>,
}
pub(super) struct State {
    pub revision: u64,
    pub jobs: Vec<Job>,
    pub worker: bool,
    pub durable_error: bool,
    pub recovery_error: bool,
}

#[derive(Clone)]
pub(crate) struct InstallJobStore {
    pub(super) state: Arc<Mutex<State>>,
    pub(super) notify: Arc<Notify>,
    pub(super) work: super::super::work_supervision::ExtensionWorkServices,
    pub(super) executor: Option<Arc<dyn InstallExecutor>>,
    pub(super) app: Option<tauri::AppHandle>,
    pub(super) journal: Option<std::path::PathBuf>,
    pub(super) disk_policy: super::disk_policy::DiskPolicy,
    #[cfg(test)]
    pub(super) free_bytes_for_test: Option<Arc<std::sync::atomic::AtomicU64>>,
}
impl InstallJobStore {
    pub(in crate::services::extensions) fn new(
        work: super::super::work_supervision::ExtensionWorkServices,
        executor: Option<Arc<dyn InstallExecutor>>,
        app: Option<tauri::AppHandle>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                revision: 0,
                jobs: Vec::new(),
                worker: false,
                durable_error: false,
                recovery_error: false,
            })),
            notify: Arc::new(Notify::new()),
            work,
            executor,
            app,
            journal: None,
            disk_policy: Default::default(),
            #[cfg(test)]
            free_bytes_for_test: None,
        }
    }
    pub(super) fn lock(&self) -> Result<MutexGuard<'_, State>, String> {
        self.state.lock().map_err(|_| UNAVAILABLE.into())
    }
    pub(crate) fn snapshot(&self) -> Result<InstallJobsSnapshot, String> {
        let state = self.lock()?;
        if state.recovery_error {
            return Err(RECOVERY_UNAVAILABLE.into());
        }
        Ok(state.snapshot())
    }
    pub(super) fn changed(&self, state: &mut State) {
        state.revision += 1;
        for job in &mut state.jobs {
            job.view.revision = state.revision;
        }
        if self.persist(state).is_err() {
            state.durable_error = true;
            for job in &state.jobs {
                job.cancel.cancel();
            }
            log::warn!("extension install journal unavailable; work cancelled");
        }
        // Consent and legacy result waiters must all observe a state transition.
        self.notify.notify_waiters();
        if let Some(app) = &self.app {
            use tauri::Emitter;
            if let Err(error) = app.emit(CHANGED_EVENT, state.snapshot()) {
                log::warn!("extension install snapshot delivery failed: {error}");
            }
        }
    }
    pub(in crate::services::extensions) fn stop_confirmed(&self) -> bool {
        self.lock().is_ok_and(|state| {
            !state.recovery_error
                && state.jobs.iter().all(|job| {
                    job.clean
                        || job.checkpoint.as_ref().is_some_and(|checkpoint| {
                            checkpoint.native_process.is_none() && !checkpoint.producer_active
                        })
                })
        })
    }

    pub(crate) fn dismiss(&self, id: &str) -> Result<(), String> {
        super::request::id(id)?;
        let mut state = self.lock()?;
        let index = state.index(id)?;
        let job = &state.jobs[index];
        if !job.view.status.terminal() || !job.clean {
            return Err(INVALID.into());
        }
        state.jobs.remove(index);
        self.changed(&mut state);
        Ok(())
    }
}
impl State {
    pub fn index(&self, id: &str) -> Result<usize, String> {
        self.jobs
            .iter()
            .position(|job| job.view.id == id)
            .ok_or_else(|| INVALID.into())
    }
    pub fn snapshot(&self) -> InstallJobsSnapshot {
        let blocker = self
            .jobs
            .iter()
            .find(|job| job.view.status == InstallStatus::AwaitingConfirmation)
            .map(|job| QueueBlocker::Confirmation {
                job_id: job.view.id.clone(),
            });
        InstallJobsSnapshot {
            revision: self.revision,
            jobs: self
                .jobs
                .iter()
                .map(|job| {
                    let mut view = job.view.clone();
                    view.queue_blocker = (view.status == InstallStatus::Queued)
                        .then(|| blocker.clone())
                        .flatten();
                    view
                })
                .collect(),
        }
    }
    pub fn evict(&mut self, maximum: usize) -> Result<(), String> {
        while self
            .jobs
            .iter()
            .filter(|job| job.view.status.terminal())
            .count()
            > maximum
        {
            let index = self
                .jobs
                .iter()
                .enumerate()
                .filter(|(_, job)| job.view.status.terminal() && job.clean)
                .min_by_key(|(_, job)| job.finished_revision.unwrap_or(u64::MAX))
                .map(|(index, _)| index)
                .ok_or(BUSY)?;
            self.jobs.remove(index);
        }
        Ok(())
    }
}
