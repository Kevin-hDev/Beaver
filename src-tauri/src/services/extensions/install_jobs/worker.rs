use super::{
    limits::FAILED, InstallControl, InstallInterruption, InstallJobStore, InstallRequest,
    InstallStatus,
};
use crate::services::work_registry::ServiceWorkCancellation;
use std::{future::Future, pin::Pin, sync::Arc};

pub(crate) type InstallFuture = Pin<Box<dyn Future<Output = InstallOutcome> + Send>>;
pub(crate) struct InstallOutcome {
    pub result: Result<String, InstallInterruption>,
    pub cleanup_confirmed: bool,
}
/// The executor owns real effects, observes cancellation, and returns only after producers
/// stop and owned artifacts are cleaned or durably retained. Never abandon its future.
pub(crate) trait InstallExecutor: Send + Sync {
    fn execute(&self, request: InstallRequest, control: InstallControl) -> InstallFuture;
    /// Recheck owned workspace capacity while the current consent is serialized with cancel.
    fn revalidate_confirmation(&self, _job_id: &str) -> Result<(), InstallInterruption> {
        Err(InstallInterruption::InsufficientSpace)
    }
}

pub(super) async fn run(
    store: InstallJobStore,
    executor: Arc<dyn InstallExecutor>,
    app_cancel: ServiceWorkCancellation,
) {
    loop {
        let next = {
            let Ok(mut state) = store.lock() else {
                return;
            };
            if app_cancel.is_cancelled() {
                for job in &mut state.jobs {
                    if job.view.status == InstallStatus::Queued {
                        job.view.status = InstallStatus::Interrupted;
                        job.view.can_cancel = false;
                    }
                }
            }
            let index = state.jobs.iter().position(|job| {
                !job.claimed_cleanup
                    && matches!(
                        job.view.status,
                        InstallStatus::Queued | InstallStatus::Cancelling
                    )
            });
            let Some(index) = index else {
                if state.evict(super::limits::MAX_RECENT).is_err() {
                    log::warn!("extension install cleanup still required");
                }
                state.worker = false;
                store.changed(&mut state);
                return;
            };
            let job = &mut state.jobs[index];
            if job.view.status == InstallStatus::Queued {
                job.view.status = InstallStatus::Running;
            }
            job.clean = false;
            let next = (job.view.id.clone(), job.request.clone(), job.cancel.clone());
            store.changed(&mut state);
            next
        };
        let (id, request, cancel) = next;
        let control = InstallControl {
            store: store.clone(),
            id: id.clone(),
            cancel,
            app_cancel: app_cancel.clone(),
        };
        let outcome = executor.execute(request, control).await;
        let Ok(mut state) = store.lock() else {
            return;
        };
        let Ok(index) = state.index(&id) else {
            return;
        };
        let revision = state.revision + 1;
        let job = &mut state.jobs[index];
        job.finished_revision = Some(revision);
        job.clean = outcome.cleanup_confirmed;
        job.view.can_cancel = false;
        job.view.confirmation_id = None;
        match outcome.result {
            // The executor returns success only when atomic publication already won.
            Ok(extension_id) => {
                if super::super::validation::identifier(&extension_id).is_ok() {
                    job.view.extension_id = Some(extension_id);
                    job.view.status = InstallStatus::Completed;
                } else {
                    job.view.status = InstallStatus::Failed;
                    job.view.error_code = Some(FAILED.into());
                }
            }
            Err(InstallInterruption::AppClosing) if job.clean => {
                job.view.status = InstallStatus::Interrupted;
            }
            Err(InstallInterruption::Cancelled) if job.clean => {
                job.view.status = InstallStatus::Cancelled;
            }
            Err(_) => {
                job.view.status = InstallStatus::Failed;
                job.view.error_code = Some(FAILED.into());
            }
        }
        let cleanup_confirmed = job.clean;
        if !cleanup_confirmed || state.evict(super::limits::MAX_RECENT).is_err() {
            // No consumer remains: stop never-started jobs explicitly while retaining
            // the failed producer's ownership evidence and leaving all sources intact.
            for queued in &mut state.jobs {
                if queued.view.status == InstallStatus::Queued {
                    queued.view.status = InstallStatus::Failed;
                    queued.view.error_code = Some(FAILED.into());
                    queued.view.can_cancel = false;
                    queued.finished_revision = Some(revision);
                }
            }
            if state.evict(super::limits::MAX_RECENT).is_err() {
                log::warn!("extension install cleanup still required");
            }
            state.worker = false;
            store.changed(&mut state);
            return;
        }
        store.changed(&mut state);
    }
}
