use super::{limits::*, types::*, worker::InstallExecutor};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub(super) struct Job {
    pub view: InstallJobView,
    pub request: InstallRequest,
    pub key: String,
    pub cancel: CancellationToken,
    // Only the executor may attest that owned artifacts are gone.
    pub clean: bool,
    pub finished_revision: Option<u64>,
}
pub(super) struct State {
    pub revision: u64,
    pub jobs: Vec<Job>,
    pub worker: bool,
}

#[derive(Clone)]
pub(crate) struct InstallJobStore {
    pub(super) state: Arc<Mutex<State>>,
    pub(super) notify: Arc<Notify>,
    pub(super) work: super::super::work_supervision::ExtensionWorkServices,
    pub(super) executor: Option<Arc<dyn InstallExecutor>>,
    pub(super) app: Option<tauri::AppHandle>,
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
            })),
            notify: Arc::new(Notify::new()),
            work,
            executor,
            app,
        }
    }
    pub(super) fn lock(&self) -> Result<MutexGuard<'_, State>, String> {
        self.state.lock().map_err(|_| UNAVAILABLE.into())
    }
    pub(crate) fn snapshot(&self) -> Result<InstallJobsSnapshot, String> {
        Ok(self.lock()?.snapshot())
    }
    pub(super) fn changed(&self, state: &mut State) {
        state.revision += 1;
        for job in &mut state.jobs {
            job.view.revision = state.revision;
        }
        self.notify.notify_one();
        if let Some(app) = &self.app {
            use tauri::Emitter;
            if let Err(error) = app.emit(CHANGED_EVENT, state.snapshot()) {
                log::warn!("extension install snapshot delivery failed: {error}");
            }
        }
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
    pub(crate) fn resume(&self, id: &str) -> Result<InstallJobView, String> {
        super::request::id(id)?;
        // Recovery must revalidate durable ownership before advertising resumability.
        let state = self.lock()?;
        state.index(id)?;
        Err(UNAVAILABLE.into())
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
