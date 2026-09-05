use super::{limits::INVALID, InstallJobStore, InstallJobView, InstallStatus};

impl InstallJobStore {
    pub(crate) fn request_cancel(&self, id: &str) -> Result<InstallJobView, String> {
        super::request::id(id)?;
        let mut state = self.lock()?;
        let index = state.index(id)?;
        let revision = state.revision + 1;
        let job = &mut state.jobs[index];
        if job.view.status.terminal() || job.view.status == InstallStatus::Cancelling {
            return Err(INVALID.into());
        }
        job.cancel.cancel();
        job.view.confirmation_id = None;
        // Queued jobs never owned a producer or files and need no asynchronous cleanup.
        job.view.status = if job.view.status == InstallStatus::Queued {
            job.finished_revision = Some(revision);
            InstallStatus::Cancelled
        } else {
            InstallStatus::Cancelling
        };
        job.view.can_cancel = false;
        state.evict(super::limits::MAX_RECENT)?;
        self.changed(&mut state);
        Ok(state.jobs[state.index(id)?].view.clone())
    }
    pub(crate) fn confirm(
        &self,
        id: &str,
        confirmation_id: &str,
    ) -> Result<InstallJobView, String> {
        super::request::id(id)?;
        super::request::id(confirmation_id)?;
        if !self.work.is_open() {
            return Err(super::limits::UNAVAILABLE.into());
        }
        let mut state = self.lock()?;
        let index = state.index(id)?;
        let job = &mut state.jobs[index];
        if job.view.status != InstallStatus::AwaitingConfirmation
            || job.view.confirmation_id.as_deref() != Some(confirmation_id)
        {
            return Err(INVALID.into());
        }
        self.executor
            .as_ref()
            .ok_or(super::limits::UNAVAILABLE)?
            .revalidate_confirmation(id)
            .map_err(|_| super::limits::UNAVAILABLE)?;
        // The current consent and its capacity check share the cancellation lock.
        job.view.confirmation_id = None;
        job.view.status = InstallStatus::Running;
        self.changed(&mut state);
        Ok(state.jobs[index].view.clone())
    }
}
