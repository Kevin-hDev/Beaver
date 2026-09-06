use super::{limits::INVALID, InstallJobStore, InstallJobView, InstallStatus};

impl InstallJobStore {
    pub(crate) async fn confirm_reconciled(
        &self,
        id: String,
        confirmation_id: String,
    ) -> Result<InstallJobView, String> {
        let store = self.clone();
        super::owned_work::spawn(&self.work, move || store.confirm(&id, &confirmation_id))?
            .await
            .map_err(|_| super::limits::UNAVAILABLE.to_string())?
    }
    pub(crate) fn request_cancel(&self, id: &str) -> Result<InstallJobView, String> {
        super::request::id(id)?;
        let mut state = self.lock()?;
        let index = state.index(id)?;
        let revision = state.revision + 1;
        let job = &mut state.jobs[index];
        if job.view.status == InstallStatus::Completed {
            return Ok(job.view.clone());
        }
        if job.view.status.terminal() || job.view.status == InstallStatus::Cancelling {
            return Err(INVALID.into());
        }
        job.cancel.cancel();
        job.view.confirmation_id = None;
        // Queued jobs never owned a producer or files and need no asynchronous cleanup.
        job.view.status = if job.view.status == InstallStatus::Queued && job.checkpoint.is_none() {
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
        let candidate = {
            let state = self.lock()?;
            let job = &state.jobs[state.index(id)?];
            if job.view.status != InstallStatus::AwaitingConfirmation
                || !super::super::fingerprint::same_encoded(
                    job.view.confirmation_id.as_deref(),
                    Some(confirmation_id),
                )
            {
                return Err(INVALID.into());
            }
            job.checkpoint.clone()
        };
        // Read-only capacity traversal must not hold up cancellation or the UI thread.
        let capacity = if let Some(checkpoint) = &candidate {
            if checkpoint.native_process.is_some() {
                return Err(super::limits::UNAVAILABLE.into());
            }
            let occupied =
                super::disk_usage::measure(checkpoint).map_err(|_| super::limits::UNAVAILABLE)?;
            let free = self.available_disk_space()?;
            let mut allowance = checkpoint.allowance.clone();
            allowance
                .approve(occupied, free, self.disk_policy)
                .map_err(|_| super::limits::INSUFFICIENT_SPACE)?;
            Some((occupied, free, allowance))
        } else {
            self.executor
                .as_ref()
                .ok_or(super::limits::UNAVAILABLE)?
                .revalidate_confirmation(id)
                .map_err(|_| super::limits::UNAVAILABLE)?;
            None
        };
        let mut state = self.lock()?;
        if !self.work.is_open() || state.durable_error {
            return Err(super::limits::UNAVAILABLE.into());
        }
        let index = state.index(id)?;
        let job = &mut state.jobs[index];
        if job.view.status != InstallStatus::AwaitingConfirmation
            || !super::super::fingerprint::same_encoded(
                job.view.confirmation_id.as_deref(),
                Some(confirmation_id),
            )
        {
            return Err(INVALID.into());
        }
        if let Some((occupied, free, allowance)) = capacity {
            job.checkpoint
                .as_mut()
                .ok_or(super::limits::UNAVAILABLE)?
                .allowance = allowance;
            job.view.occupied_bytes = occupied;
            job.view.free_bytes = Some(free);
            job.monitor = Default::default();
        }
        // Cancellation or another confirmation can win during the capacity traversal.
        job.view.confirmation_id = None;
        job.view.status = InstallStatus::Running;
        self.changed(&mut state);
        Ok(state.jobs[index].view.clone())
    }
}
