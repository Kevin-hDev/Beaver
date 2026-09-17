use super::{InstallJobStore, InstallPhase, InstallStatus};
use crate::services::work_registry::ServiceWorkCancellation;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum InstallInterruption {
    Cancelled,
    Confirmation,
    InsufficientSpace,
    AppClosing,
    Failed,
}
#[derive(Clone, Debug)]
pub(crate) struct InstallProgress {
    pub phase: InstallPhase,
    pub downloaded_bytes: Option<u64>,
    pub download_total_bytes: Option<u64>,
    pub occupied_bytes: u64,
    pub free_bytes: Option<u64>,
}
#[derive(Clone)]
pub(crate) struct InstallControl {
    pub(super) store: InstallJobStore,
    pub(super) id: String,
    pub(super) cancel: CancellationToken,
    pub(super) app_cancel: ServiceWorkCancellation,
}
impl InstallControl {
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled() || self.app_cancel.is_cancelled()
    }
    pub(crate) async fn cancelled(&self) {
        tokio::select! { _ = self.cancel.cancelled() => {}, _ = self.app_cancel.cancelled() => {} }
    }
    pub(crate) fn checkpoint(&self, phase: InstallPhase) -> Result<(), InstallInterruption> {
        self.checkpoint_with(phase, || {})
    }
    pub(super) fn checkpoint_with(
        &self,
        phase: InstallPhase,
        before_lock: impl FnOnce(),
    ) -> Result<(), InstallInterruption> {
        if self.app_cancel.is_cancelled() {
            return Err(InstallInterruption::AppClosing);
        }
        if self.cancel.is_cancelled() {
            return Err(InstallInterruption::Cancelled);
        }
        before_lock();
        let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        // Cancellation can win while this checkpoint waits for the state lock.
        // Preserve its reason before interpreting the resulting Cancelling status.
        if self.app_cancel.is_cancelled() {
            return Err(InstallInterruption::AppClosing);
        }
        if self.cancel.is_cancelled() {
            return Err(InstallInterruption::Cancelled);
        }
        let index = state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?;
        let job = &mut state.jobs[index];
        if job.view.status == InstallStatus::AwaitingConfirmation {
            return Err(InstallInterruption::Confirmation);
        }
        if job.view.status != InstallStatus::Running {
            return Err(InstallInterruption::Failed);
        }
        if job.view.phase != phase {
            // Network counters belong to one phase, never to subsequent npm/UI work.
            job.monitor.downloaded = None;
            job.view.downloaded_bytes = None;
            job.view.download_total_bytes = None;
        }
        job.view.phase = phase;
        self.store.changed(&mut state);
        Ok(())
    }
    pub(crate) fn progress(&self, progress: InstallProgress) -> Result<(), InstallInterruption> {
        if progress
            .download_total_bytes
            .zip(progress.downloaded_bytes)
            .is_some_and(|(total, received)| received > total)
        {
            return Err(InstallInterruption::Failed);
        }
        let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
        let index = state
            .index(&self.id)
            .map_err(|_| InstallInterruption::Failed)?;
        let view = &mut state.jobs[index].view;
        if view.status != InstallStatus::Running {
            return Err(InstallInterruption::Cancelled);
        }
        if self.is_cancelled() {
            return Err(InstallInterruption::Cancelled);
        }
        view.phase = progress.phase;
        view.downloaded_bytes = progress.downloaded_bytes;
        view.download_total_bytes = progress.download_total_bytes;
        view.occupied_bytes = progress.occupied_bytes;
        view.free_bytes = progress.free_bytes;
        self.store.changed(&mut state);
        Ok(())
    }
    /// Call only after every producer has stopped. On return, recheck disk before resuming.
    pub(crate) async fn await_confirmation(&self) -> Result<(), InstallInterruption> {
        {
            let mut state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
            let index = state
                .index(&self.id)
                .map_err(|_| InstallInterruption::Failed)?;
            let view = &mut state.jobs[index].view;
            if view.status != InstallStatus::Running || self.is_cancelled() {
                return Err(InstallInterruption::Cancelled);
            }
            view.status = InstallStatus::AwaitingConfirmation;
            view.confirmation_id = Some(uuid::Uuid::new_v4().to_string());
            self.store.changed(&mut state);
        }
        loop {
            let notified = self.store.notify.notified();
            {
                let state = self.store.lock().map_err(|_| InstallInterruption::Failed)?;
                let index = state
                    .index(&self.id)
                    .map_err(|_| InstallInterruption::Failed)?;
                if self.app_cancel.is_cancelled() {
                    return Err(InstallInterruption::AppClosing);
                }
                if self.cancel.is_cancelled() {
                    return Err(InstallInterruption::Cancelled);
                }
                if state.jobs[index].view.status == InstallStatus::Running {
                    return Ok(());
                }
            }
            tokio::select! { _ = notified => {}, _ = self.cancelled() => {} }
        }
    }
}
