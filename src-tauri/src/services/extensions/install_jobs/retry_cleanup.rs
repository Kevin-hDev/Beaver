use super::{InstallJobStore, InstallJobView, InstallRequest, InstallStatus};

impl InstallJobStore {
    pub(crate) async fn start_reconciled(
        &self,
        request: InstallRequest,
    ) -> Result<InstallJobView, String> {
        let (_, key, _) = super::request::normalize(request.clone())?;
        let prior = {
            let state = self.lock()?;
            state
                .jobs
                .iter()
                .find(|job| {
                    job.key == key
                        && job.view.status.terminal()
                        && job.finished_revision.is_some()
                        && !job.clean
                })
                .map(|job| job.view.id.clone())
        };
        if let Some(id) = prior {
            self.dismiss_reconciled(&id).await?;
        }
        self.start(request)
    }

    pub(crate) async fn dismiss_reconciled(&self, id: &str) -> Result<(), String> {
        super::request::id(id)?;
        let receiver = {
            let mut state = self.lock()?;
            let index = state.index(id)?;
            let job = &state.jobs[index];
            if !job.view.status.terminal() {
                return Err(super::limits::INVALID.into());
            }
            if job.clean {
                drop(state);
                return self.dismiss(id);
            }
            if !matches!(
                job.view.status,
                InstallStatus::Interrupted | InstallStatus::Completed | InstallStatus::Failed
            ) {
                return Err(super::limits::UNAVAILABLE.into());
            }
            if job.finished_revision.is_none() {
                return Err(super::limits::BUSY.into());
            }
            let checkpoint = job.checkpoint.clone().ok_or(super::limits::UNAVAILABLE)?;
            let previous_status = job.view.status;
            state.jobs[index].claimed_cleanup = true;
            state.jobs[index].view.status = InstallStatus::Cancelling;
            state.jobs[index].view.can_resume = false;
            self.persist(&state)?;
            let store = self.clone();
            let id = id.to_owned();
            let receiver = super::owned_work::spawn(&self.work, move || {
                let mut checkpoint = checkpoint;
                let result = super::recovery::stop_recovered(&mut checkpoint)
                    .and_then(|()| super::cleanup::run(&checkpoint));
                let mut state = store.lock()?;
                let index = state.index(&id)?;
                state.jobs[index].claimed_cleanup = false;
                state.jobs[index].view.status = previous_status;
                state.jobs[index].clean = result.is_ok();
                checkpoint.cleanup_unconfirmed = result.is_err();
                state.jobs[index].checkpoint = Some(checkpoint);
                if result.is_ok() {
                    state.jobs.remove(index);
                }
                store.persist(&state)?;
                store.changed(&mut state);
                result
            });
            if receiver.is_err() {
                state.jobs[index].claimed_cleanup = false;
                state.jobs[index].view.status = previous_status;
                self.changed(&mut state);
            }
            drop(state);
            receiver
        };
        receiver?.await.map_err(|_| super::limits::UNAVAILABLE)?
    }
}
