use super::{limits::*, InstallJobStore, InstallJobView, InstallStatus};
use tokio_util::sync::CancellationToken;

impl InstallJobStore {
    pub(crate) async fn resume_reconciled(&self, id: String) -> Result<InstallJobView, String> {
        super::request::id(&id)?;
        let store = self.clone();
        // Fingerprinting may read a large tree; keep it off the window thread
        // while retaining supervised ownership until that read actually ends.
        super::owned_work::spawn(&self.work, move || store.resume(&id))?
            .await
            .map_err(|_| UNAVAILABLE)?
    }
    pub(crate) fn resume(&self, id: &str) -> Result<InstallJobView, String> {
        super::request::id(id)?;
        let candidate = {
            let state = self.lock()?;
            let job = &state.jobs[state.index(id)?];
            if job.view.status != InstallStatus::Interrupted {
                return Err(INVALID.into());
            }
            job.checkpoint.clone().ok_or(UNAVAILABLE)?
        };
        if !super::recovery::resumable(&candidate)
            || self.available_disk_space()? <= self.disk_policy.reserve_bytes
        {
            return Err(UNAVAILABLE.into());
        }
        let mut state = self.lock()?;
        if !self.work.is_open() || state.durable_error {
            return Err(UNAVAILABLE.into());
        }
        let index = state.index(id)?;
        if state.jobs[index].view.status != InstallStatus::Interrupted {
            return Err(INVALID.into());
        }
        if state
            .jobs
            .iter()
            .any(|job| job.view.id != id && job.view.status.terminal() && !job.clean)
        {
            return Err(BUSY.into());
        }
        if state
            .jobs
            .iter()
            .filter(|job| !job.view.status.terminal())
            .count()
            >= MAX_ACTIVE
        {
            return Err(BUSY.into());
        }
        let executor = self.executor.clone().ok_or(UNAVAILABLE)?;
        let admission = if state.worker {
            None
        } else {
            Some(self.work.try_admit_operation().map_err(|_| UNAVAILABLE)?)
        };
        let job = &mut state.jobs[index];
        job.cancel = CancellationToken::new();
        job.view.status = InstallStatus::Queued;
        job.view.can_cancel = true;
        job.view.can_resume = false;
        job.view.confirmation_id = None;
        job.finished_revision = None;
        // A new explicit resume invalidates all historical volume consent.
        if let Some(checkpoint) = &mut job.checkpoint {
            checkpoint.allowance = super::disk_policy::StorageAllowance::new(self.disk_policy);
        }
        job.monitor = Default::default();
        self.persist(&state)?;
        if let Some(admission) = admission {
            state.worker = true;
            let store = self.clone();
            if admission
                .spawn(move |cancel| super::worker::run(store, executor, cancel))
                .is_err()
            {
                state.worker = false;
                state.jobs[index].view.status = InstallStatus::Interrupted;
                self.changed(&mut state);
                return Err(UNAVAILABLE.into());
            }
        }
        self.changed(&mut state);
        Ok(state.jobs[index].view.clone())
    }
}
