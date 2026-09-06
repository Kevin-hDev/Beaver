use super::*;
use std::sync::Arc;

struct Suspended;
impl InstallExecutor for Suspended {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        Box::pin(async move {
            control.cancelled().await;
            InstallOutcome {
                result: Err(InstallInterruption::Cancelled),
                cleanup_confirmed: true,
            }
        })
    }
}
fn store(executor: Arc<dyn InstallExecutor>) -> InstallJobStore {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    InstallJobStore::new(
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor()),
        Some(executor),
        None,
    )
}
fn request(name: &str) -> InstallRequest {
    InstallRequest::Npm {
        locator: name.into(),
    }
}
async fn wait_status(store: &InstallJobStore, id: &str, status: InstallStatus) -> InstallJobView {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            let view = store
                .snapshot()
                .unwrap()
                .jobs
                .into_iter()
                .find(|job| job.id == id)
                .unwrap();
            if view.status == status {
                return view;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap()
}
async fn stop(store: &InstallJobStore) {
    assert!(
        store
            .work
            .stop_and_wait(std::time::Instant::now() + std::time::Duration::from_secs(2))
            .await
    );
}
#[tokio::test]
async fn admission_returns_before_suspended_work_and_bounds_deduplicates() {
    let store = store(Arc::new(Suspended));
    let first = store.start(request("example")).unwrap();
    assert_eq!(first.id, store.start(request("example")).unwrap().id);
    wait_status(&store, &first.id, InstallStatus::Running).await;
    for index in 1..8 {
        store.start(request(&format!("example-{index}"))).unwrap();
    }
    assert!(store.start(request("overflow")).is_err());
    assert_eq!(store.snapshot().unwrap().jobs.len(), 8);
    assert_eq!(store.work.operation_diagnostics().active, 1);
    stop(&store).await;
}
#[tokio::test]
async fn closing_rejects_admission_without_phantom_job() {
    let store = store(Arc::new(Suspended));
    store.work.begin_closing();
    assert!(store.start(request("example")).is_err());
    assert!(store.snapshot().unwrap().jobs.is_empty());
    stop(&store).await;
}
#[test]
fn failed_spawn_releases_reservation_and_retains_only_failed_result() {
    let store = store(Arc::new(Suspended));
    assert!(store
        .start_with(request("example"), || store.work.begin_closing())
        .is_err());
    let snapshot = store.snapshot().unwrap();
    assert_eq!(snapshot.jobs.len(), 1);
    assert_eq!(snapshot.jobs[0].status, InstallStatus::Failed);
    assert!(!store.lock().unwrap().worker);
    assert_eq!(store.work.operation_diagnostics().active, 0);
}
struct Confirming;
impl InstallExecutor for Confirming {
    fn revalidate_confirmation(&self, _: &str) -> Result<(), InstallInterruption> {
        Ok(())
    }
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        Box::pin(async move {
            let result = control.await_confirmation().await;
            if result.is_ok() {
                control.cancelled().await;
            }
            InstallOutcome {
                result: Err(InstallInterruption::Cancelled),
                cleanup_confirmed: true,
            }
        })
    }
}
#[tokio::test]
async fn queue_blocker_consent_and_queued_cancellation_are_authoritative() {
    let store = store(Arc::new(Confirming));
    let first = store.start(request("first")).unwrap();
    let pending = wait_status(&store, &first.id, InstallStatus::AwaitingConfirmation).await;
    let second = store.start(request("second")).unwrap();
    assert_eq!(
        store.start(request("second")).unwrap().queue_blocker,
        second.queue_blocker
    );
    assert_eq!(
        second.queue_blocker,
        Some(QueueBlocker::Confirmation {
            job_id: first.id.clone()
        })
    );
    assert!(store
        .confirm("bad", pending.confirmation_id.as_ref().unwrap())
        .is_err());
    assert!(store
        .confirm(&first.id, &uuid::Uuid::new_v4().to_string())
        .is_err());
    assert!(store
        .confirm(&second.id, pending.confirmation_id.as_ref().unwrap())
        .is_err());
    let token = pending.confirmation_id.unwrap();
    store.confirm(&first.id, &token).unwrap();
    assert!(store.confirm(&first.id, &token).is_err());
    assert!(store.snapshot().unwrap().jobs[1].queue_blocker.is_none());
    store.request_cancel(&second.id).unwrap();
    assert!(store.request_cancel(&second.id).is_err());
    assert!(store.confirm(&second.id, &token).is_err());
    assert!(store.resume(&second.id).is_err());
    assert!(store.dismiss(&first.id).is_err());
    store.dismiss(&second.id).unwrap();
    stop(&store).await;
}
#[tokio::test]
async fn cancellation_unblocks_confirmation_and_does_not_cancel_service() {
    let store = store(Arc::new(Confirming));
    let first = store.start(request("first")).unwrap();
    wait_status(&store, &first.id, InstallStatus::AwaitingConfirmation).await;
    let second = store.start(request("second")).unwrap();
    store.request_cancel(&first.id).unwrap();
    assert!(store.snapshot().unwrap().jobs[1].queue_blocker.is_none());
    wait_status(&store, &first.id, InstallStatus::Cancelled).await;
    wait_status(&store, &second.id, InstallStatus::AwaitingConfirmation).await;
    assert!(store.work.is_open());
    stop(&store).await;
}
#[tokio::test]
async fn snapshots_are_bounded_revisions_monotone_and_only_terminals_evicted() {
    let store = store(Arc::new(Suspended));
    let first = store.start(request("first")).unwrap();
    wait_status(&store, &first.id, InstallStatus::Running).await;
    let mut revision = 0;
    for index in 0..70 {
        let job = store.start(request(&format!("queued-{index}"))).unwrap();
        store.request_cancel(&job.id).unwrap();
        let snapshot = store.snapshot().unwrap();
        assert!(snapshot.revision > revision);
        revision = snapshot.revision;
        assert!(snapshot.jobs.len() <= 33);
        assert!(snapshot.jobs.iter().any(|job| job.id == first.id));
    }
    // Completion order, not admission order, decides which result is oldest.
    store.request_cancel(&first.id).unwrap();
    wait_status(&store, &first.id, InstallStatus::Cancelled).await;
    store.start(request("newest")).unwrap();
    assert!(store
        .snapshot()
        .unwrap()
        .jobs
        .iter()
        .any(|job| job.id == first.id));
    stop(&store).await;
}
#[tokio::test]
async fn public_views_never_expose_source_paths_or_locator() {
    let store = store(Arc::new(Suspended));
    let job = store
        .start(InstallRequest::Git {
            locator: "https://private.example/repo.git".into(),
        })
        .unwrap();
    assert!(!serde_json::to_string(&job)
        .unwrap()
        .contains("private.example"));
    assert!(store.start(request("../invalid")).is_err());
    stop(&store).await;
}

struct InvalidProgress;
impl InstallExecutor for InvalidProgress {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        Box::pin(async move {
            let before = control.store.snapshot().unwrap().revision;
            assert!(control
                .progress(InstallProgress {
                    phase: InstallPhase::Downloading,
                    downloaded_bytes: Some(20),
                    download_total_bytes: Some(10),
                    occupied_bytes: 0,
                    free_bytes: None,
                })
                .is_err());
            let after = control.store.snapshot().unwrap().revision;
            InstallOutcome {
                result: if before == after {
                    Ok("test-extension".into())
                } else {
                    Err(InstallInterruption::Failed)
                },
                cleanup_confirmed: true,
            }
        })
    }
}
#[tokio::test]
async fn rejected_progress_cannot_mutate_the_snapshot() {
    let store = store(Arc::new(InvalidProgress));
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
    assert_eq!(terminal.id, job.id);
    assert_eq!(terminal.status, InstallStatus::Completed);
    stop(&store).await;
}

#[tokio::test]
async fn queued_cancel_while_confirmation_is_pending_and_racing_decisions() {
    let store = store(Arc::new(Confirming));
    let first = store.start(request("first")).unwrap();
    let pending = wait_status(&store, &first.id, InstallStatus::AwaitingConfirmation).await;
    let second = store.start(request("second")).unwrap();
    store.request_cancel(&second.id).unwrap();
    assert_eq!(
        store.snapshot().unwrap().jobs[0].status,
        InstallStatus::AwaitingConfirmation
    );
    let token = pending.confirmation_id.unwrap();
    std::thread::scope(|scope| {
        let confirm = scope.spawn(|| store.confirm(&first.id, &token));
        let cancel = scope.spawn(|| store.request_cancel(&first.id));
        assert!(cancel.join().unwrap().is_ok());
        let _ = confirm.join().unwrap();
    });
    wait_status(&store, &first.id, InstallStatus::Cancelled).await;
    assert!(store.confirm(&first.id, &token).is_err());
    stop(&store).await;
}

#[tokio::test]
async fn simultaneous_double_click_has_one_reservation() {
    let store = store(Arc::new(Suspended));
    std::thread::scope(|scope| {
        let first = scope.spawn(|| store.start(request("same")).unwrap());
        let second = scope.spawn(|| store.start(request("same")).unwrap());
        assert_eq!(first.join().unwrap().id, second.join().unwrap().id);
    });
    assert_eq!(store.work.operation_diagnostics().active, 1);
    assert_eq!(store.snapshot().unwrap().jobs.len(), 1);
    stop(&store).await;
}

struct DirtyFailure;
impl InstallExecutor for DirtyFailure {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        Box::pin(async move {
            control.checkpoint(InstallPhase::Cleaning).unwrap();
            InstallOutcome {
                result: Err(InstallInterruption::InsufficientSpace),
                cleanup_confirmed: false,
            }
        })
    }
}
#[tokio::test]
async fn unconfirmed_cleanup_retains_ownership_and_blocks_new_admission() {
    let store = store(Arc::new(DirtyFailure));
    let job = store.start(request("first")).unwrap();
    wait_status(&store, &job.id, InstallStatus::Failed).await;
    assert!(store.dismiss(&job.id).is_err());
    assert!(store.start(request("second")).is_err());
    assert_eq!(store.snapshot().unwrap().jobs.len(), 1);
    stop(&store).await;
}
#[tokio::test]
async fn missing_production_executor_never_admits_a_fake_success() {
    let mut store = store(Arc::new(Suspended));
    store.executor = None;
    assert!(store.start(request("example")).is_err());
    assert!(store.snapshot().unwrap().jobs.is_empty());
    stop(&store).await;
}
#[tokio::test]
async fn interrupted_without_revalidated_checkpoint_cannot_resume() {
    let store = store(Arc::new(Suspended));
    let first = store.start(request("first")).unwrap();
    wait_status(&store, &first.id, InstallStatus::Running).await;
    let second = store.start(request("second")).unwrap();
    {
        let mut state = store.lock().unwrap();
        let index = state.index(&second.id).unwrap();
        state.jobs[index].view.status = InstallStatus::Interrupted;
        state.jobs[index].view.can_cancel = false;
        store.changed(&mut state);
    }
    assert!(store.resume(&second.id).is_err());
    assert!(!store.snapshot().unwrap().jobs[1].can_resume);
    store.dismiss(&second.id).unwrap();
    stop(&store).await;
}

struct SpaceDenied;
impl InstallExecutor for SpaceDenied {
    fn execute(&self, request: InstallRequest, control: InstallControl) -> InstallFuture {
        Confirming.execute(request, control)
    }
}
#[tokio::test]
async fn consent_requires_capacity_revalidation_before_running() {
    let store = store(Arc::new(SpaceDenied));
    let job = store.start(request("example")).unwrap();
    let view = wait_status(&store, &job.id, InstallStatus::AwaitingConfirmation).await;
    let revision = store.snapshot().unwrap().revision;
    assert!(store
        .confirm(&job.id, view.confirmation_id.as_ref().unwrap())
        .is_err());
    let snapshot = store.snapshot().unwrap();
    assert_eq!(snapshot.revision, revision);
    assert_eq!(snapshot.jobs[0].status, InstallStatus::AwaitingConfirmation);
    assert_eq!(snapshot.jobs[0].confirmation_id, view.confirmation_id);
    stop(&store).await;
}

#[tokio::test]
async fn cancelling_a_full_queue_never_overflows_recent_results() {
    let store = store(Arc::new(Suspended));
    let active = store.start(request("active")).unwrap();
    wait_status(&store, &active.id, InstallStatus::Running).await;
    for index in 0..31 {
        let job = store.start(request(&format!("old-{index}"))).unwrap();
        store.request_cancel(&job.id).unwrap();
    }
    let queued: Vec<_> = (0..7)
        .map(|index| store.start(request(&format!("burst-{index}"))).unwrap())
        .collect();
    for job in queued {
        store.request_cancel(&job.id).unwrap();
        assert!(
            store
                .snapshot()
                .unwrap()
                .jobs
                .iter()
                .filter(|job| job.status.terminal())
                .count()
                <= 32
        );
    }
    stop(&store).await;
    assert!(store.snapshot().unwrap().jobs.len() <= 32);
}

#[path = "review_tests.rs"]
mod review_tests;

#[tokio::test]
async fn full_queue_recovery_preserves_interrupted_jobs_and_bounds_history() {
    let root = tempfile::tempdir().unwrap();
    let original = store(Arc::new(Suspended)).restore(root.path().join("live.json"));
    let active = original.start(request("active-recovery")).unwrap();
    wait_status(&original, &active.id, InstallStatus::Running).await;
    for index in 0..31 {
        let old = original
            .start(request(&format!("historical-{index}")))
            .unwrap();
        original.request_cancel(&old.id).unwrap();
    }
    let mut interrupted = vec![active.id];
    for index in 0..7 {
        interrupted.push(
            original
                .start(request(&format!("pending-recovery-{index}")))
                .unwrap()
                .id,
        );
    }
    let copy = root.path().join("restart.json");
    std::fs::copy(root.path().join("live.json"), &copy).unwrap();
    stop(&original).await;
    let restored = store(Arc::new(Suspended)).restore(copy);
    let jobs = restored.snapshot().unwrap().jobs;
    assert!(
        jobs.len() <= super::limits::MAX_RECENT,
        "restored {} terminal jobs",
        jobs.len()
    );
    for id in interrupted {
        assert!(jobs
            .iter()
            .any(|job| job.id == id && job.status == InstallStatus::Interrupted));
    }
    stop(&restored).await;
}

#[tokio::test]
async fn malformed_journal_revision_and_duplicate_ids_fail_closed_before_recovery() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.json");
    let original = store(Arc::new(Suspended)).restore(path.clone());
    original.start(request("journal-validation")).unwrap();
    stop(&original).await;
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    for duplicate in [false, true] {
        let mut journal = original.clone();
        if duplicate {
            let job = journal["jobs"][0].clone();
            journal["jobs"].as_array_mut().unwrap().push(job);
        } else {
            journal["revision"] = u64::MAX.into();
        }
        let path = root.path().join(format!("bad-{duplicate}.json"));
        std::fs::write(&path, serde_json::to_vec(&journal).unwrap()).unwrap();
        assert!(super::checkpoint::load(&path).is_err());
        let restored = store(Arc::new(Suspended)).restore(path);
        assert_eq!(
            restored.snapshot().unwrap_err(),
            super::limits::RECOVERY_UNAVAILABLE
        );
        stop(&restored).await;
    }
}

#[tokio::test]
async fn explicit_retry_after_restart_uses_private_source_and_deduplicates() {
    let root = tempfile::tempdir().unwrap();
    let journal = root.path().join("jobs.json");
    let original = store(Arc::new(Suspended)).restore(journal.clone());
    let job = original
        .start(InstallRequest::Git {
            locator: "https://private.example/fixture.git".into(),
        })
        .unwrap();
    wait_status(&original, &job.id, InstallStatus::Running).await;
    assert!(original.retry(&job.id).await.is_err());
    original.request_cancel(&job.id).unwrap();
    wait_status(&original, &job.id, InstallStatus::Cancelled).await;
    stop(&original).await;
    let restored = store(Arc::new(Suspended)).restore(journal);
    let (first, second) = tokio::join!(restored.retry(&job.id), restored.retry(&job.id));
    let first = first.unwrap();
    assert_ne!(first.id, job.id);
    assert_eq!(first.id, second.unwrap().id);
    assert_eq!(first.display_name, "fixture");
    assert!(!serde_json::to_string(&first)
        .unwrap()
        .contains("private.example"));
    {
        let state = restored.lock().unwrap();
        assert!(
            matches!(&state.jobs[state.index(&first.id).unwrap()].request,
            InstallRequest::Git { locator } if locator == "https://private.example/fixture.git")
        );
    }
    stop(&restored).await;
}
