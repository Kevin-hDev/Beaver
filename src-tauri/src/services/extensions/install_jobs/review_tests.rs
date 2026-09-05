use super::{request, stop, store, wait_status};
use crate::services::extensions::install_jobs::*;
use std::sync::{Arc, Mutex};

struct RacingCheckpoint {
    closing: bool,
    observed: Arc<Mutex<Option<InstallInterruption>>>,
}
impl InstallExecutor for RacingCheckpoint {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        let closing = self.closing;
        let observed = self.observed.clone();
        Box::pin(async move {
            let result = control.checkpoint_with(InstallPhase::Validating, || {
                // Exactly between the initial cancellation check and state acquisition.
                if closing {
                    control.app_cancel.cancel();
                } else {
                    control.store.request_cancel(&control.id).unwrap();
                }
            });
            *observed.lock().unwrap() = result.clone().err();
            InstallOutcome {
                result: result.map(|_| "unexpected-success".into()),
                cleanup_confirmed: true,
            }
        })
    }
}
async fn checkpoint_race(closing: bool, expected: InstallInterruption) {
    let observed = Arc::new(Mutex::new(None));
    let store = store(Arc::new(RacingCheckpoint {
        closing,
        observed: observed.clone(),
    }));
    let job = store.start(request("example")).unwrap();
    let terminal = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let view = store.snapshot().unwrap().jobs[0].clone();
            if view.status.terminal() {
                return view;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    stop(&store).await;
    assert_eq!(*observed.lock().unwrap(), Some(expected));
    assert_eq!(terminal.id, job.id);
    assert_eq!(
        terminal.status,
        if closing {
            InstallStatus::Interrupted
        } else {
            InstallStatus::Cancelled
        }
    );
}
#[tokio::test]
async fn individual_cancel_between_check_and_lock_keeps_its_classification() {
    checkpoint_race(false, InstallInterruption::Cancelled).await;
}
#[tokio::test]
async fn closing_between_check_and_lock_keeps_its_classification() {
    checkpoint_race(true, InstallInterruption::AppClosing).await;
}

struct GatedDirtyFailure(Arc<tokio::sync::Notify>);
impl InstallExecutor for GatedDirtyFailure {
    fn execute(&self, _: InstallRequest, _: InstallControl) -> InstallFuture {
        let release = self.0.clone();
        Box::pin(async move {
            release.notified().await;
            InstallOutcome {
                result: Err(InstallInterruption::Failed),
                cleanup_confirmed: false,
            }
        })
    }
}
#[tokio::test]
async fn dirty_failure_stops_queued_jobs_without_touching_sources_or_losing_ownership() {
    let release = Arc::new(tokio::sync::Notify::new());
    let store = store(Arc::new(GatedDirtyFailure(release.clone())));
    let active = store.start(request("active")).unwrap();
    wait_status(&store, &active.id, InstallStatus::Running).await;
    for index in 0..31 {
        let job = store.start(request(&format!("old-{index}"))).unwrap();
        store.request_cancel(&job.id).unwrap();
    }
    let source = tempfile::tempdir().unwrap();
    let path = source.path().join("extension.ts");
    std::fs::write(&path, "user-owned source").unwrap();
    let queued = store
        .start(InstallRequest::Local {
            path: path.to_str().unwrap().into(),
        })
        .unwrap();
    for index in 0..6 {
        store.start(request(&format!("pending-{index}"))).unwrap();
    }
    release.notify_one();
    wait_status(&store, &active.id, InstallStatus::Failed).await;
    stop(&store).await;
    let snapshot = store.snapshot().unwrap();
    assert!(snapshot.jobs.iter().all(|job| job.status.terminal()));
    assert!(snapshot.jobs.len() <= 32);
    let pending = snapshot
        .jobs
        .iter()
        .find(|job| job.id == queued.id)
        .unwrap();
    assert_eq!(pending.status, InstallStatus::Failed);
    assert_eq!(
        pending.error_code.as_deref(),
        Some(super::super::limits::FAILED)
    );
    assert!(!pending.can_cancel);
    assert!(pending.revision > queued.revision);
    {
        let state = store.lock().unwrap();
        assert!(!state.worker);
        let failed = &state.jobs[state.index(&active.id).unwrap()];
        assert!(!failed.clean);
        let pending = &state.jobs[state.index(&queued.id).unwrap()];
        assert!(pending.clean);
        assert_eq!(pending.finished_revision, Some(pending.view.revision));
    }
    assert!(store.dismiss(&active.id).is_err());
    store.dismiss(&queued.id).unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), "user-owned source");
}
