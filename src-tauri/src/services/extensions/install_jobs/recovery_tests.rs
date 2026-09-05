use super::*;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

struct Waiting(Arc<AtomicUsize>);
impl InstallExecutor for Waiting {
    fn execute(&self, _: InstallRequest, control: InstallControl) -> InstallFuture {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            control.cancelled().await;
            InstallOutcome {
                result: Err(InstallInterruption::Cancelled),
                cleanup_confirmed: true,
            }
        })
    }
}
fn store(path: &std::path::Path, calls: Arc<AtomicUsize>) -> InstallJobStore {
    let app = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    InstallJobStore::new(
        super::super::work_supervision::ExtensionWorkServices::new(app.work_supervisor()),
        Some(Arc::new(Waiting(calls))),
        None,
    )
    .restore(path.to_owned())
}
async fn interrupted(
    ready: bool,
) -> (
    tempfile::TempDir,
    InstallJobStore,
    String,
    std::path::PathBuf,
) {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("index.mjs"), "export default {};").unwrap();
    std::fs::write(source.join("beaver-extension.json"), r#"{"id":"test.recovery","name":"Recovery fixture","version":"1.0.0","beaverApi":"1","runtime":"node","main":"index.mjs","access":"full","apiLevel":"stable"}"#).unwrap();
    let path = root.path().join("jobs.json");
    let calls = Arc::new(AtomicUsize::new(0));
    let old = store(&path, calls.clone());
    let job = old
        .start(InstallRequest::Local {
            path: source.to_str().unwrap().into(),
        })
        .unwrap();
    while calls.load(Ordering::SeqCst) == 0 {
        tokio::task::yield_now().await;
    }
    if ready {
        let record = super::super::manifest::load_local(source.to_str().unwrap())
            .unwrap()
            .record;
        let mut state = old.lock().unwrap();
        let index = state.index(&job.id).unwrap();
        state.jobs[index].checkpoint = Some(super::checkpoint::InstallCheckpoint {
            version: 1,
            token: uuid::Uuid::new_v4().simple().to_string(),
            record: Some(record),
            safe_phase: Some(InstallPhase::BuildingUi),
            budget_bytes: super::super::managed_tree::MAX_TOTAL_BYTES,
            ..Default::default()
        });
        old.persist(&state).unwrap();
    }
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        old.work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    std::fs::write(&path, bytes).unwrap();
    let recovered = store(&path, Arc::new(AtomicUsize::new(0)));
    (root, recovered, job.id, source)
}

#[tokio::test]
async fn durable_queued_jobs_restore_interrupted_without_spontaneous_execution() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("jobs.json");
    let calls = Arc::new(AtomicUsize::new(0));
    let old = store(&path, calls.clone());
    old.start(InstallRequest::Npm {
        locator: "first".into(),
    })
    .unwrap();
    old.start(InstallRequest::Npm {
        locator: "second".into(),
    })
    .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert!(
        old.work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    std::fs::write(&path, bytes).unwrap();
    let resumed_calls = Arc::new(AtomicUsize::new(0));
    let restored = store(&path, resumed_calls.clone());
    assert_eq!(restored.snapshot().unwrap().jobs.len(), 2);
    assert!(restored
        .snapshot()
        .unwrap()
        .jobs
        .iter()
        .all(|job| job.status == InstallStatus::Interrupted && !job.can_resume));
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(resumed_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn ready_checkpoint_allows_one_explicit_resume_and_invalidates_old_consent() {
    let (_root, store, id, _source) = interrupted(true).await;
    assert!(store.snapshot().unwrap().jobs[0].can_resume);
    assert!(store.resume(&id).is_ok());
    assert!(store.resume(&id).is_err());
    assert!(store.snapshot().unwrap().jobs[0].confirmation_id.is_none());
    assert!(
        store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}

#[tokio::test]
async fn changed_source_refuses_resume_and_explicit_dismiss_preserves_local_source() {
    let (_root, store, id, source) = interrupted(true).await;
    std::fs::write(source.join("index.mjs"), "export default {changed:true};").unwrap();
    assert!(store.resume(&id).is_err());
    store.dismiss_reconciled(&id).await.unwrap();
    assert!(source.join("index.mjs").exists());
    assert!(store.snapshot().unwrap().jobs.is_empty());
}

#[tokio::test]
async fn absent_checkpoint_refuses_resume_without_reserving_worker() {
    let (_root, store, id, _) = interrupted(false).await;
    assert!(!store.snapshot().unwrap().jobs[0].can_resume);
    assert!(store.resume(&id).is_err());
    assert_eq!(store.work.operation_diagnostics().active, 0);
}

#[test]
fn corrupt_or_incompatible_journal_blocks_admission_and_preserves_evidence() {
    for bytes in [
        b"{".as_slice(),
        br#"{"version":2,"revision":0,"jobs":[]}"#.as_slice(),
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("jobs.json");
        std::fs::write(&path, bytes).unwrap();
        let store = store(&path, Arc::new(AtomicUsize::new(0)));
        assert!(store
            .start(InstallRequest::Npm {
                locator: "example".into()
            })
            .is_err());
        assert_eq!(
            store.snapshot().unwrap_err(),
            super::limits::RECOVERY_UNAVAILABLE
        );
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}

#[tokio::test]
async fn incompatible_checkpoint_with_known_stopped_ownership_allows_safe_retry_cleanup() {
    let (root, original, id, source) = interrupted(true).await;
    {
        let mut state = original.lock().unwrap();
        state.jobs[0].checkpoint.as_mut().unwrap().version = 2;
        original.persist(&state).unwrap();
    }
    let restored = store(
        &root.path().join("jobs.json"),
        Arc::new(AtomicUsize::new(0)),
    );
    assert!(!restored.snapshot().unwrap().jobs[0].can_resume);
    assert!(restored.resume(&id).is_err());
    restored.dismiss_reconciled(&id).await.unwrap();
    assert!(source.join("index.mjs").exists());
}

#[path = "recovery_publication_tests.rs"]
mod publication;
