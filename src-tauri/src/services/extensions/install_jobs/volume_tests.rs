use super::*;
use std::time::{Duration, Instant};

struct Fixture {
    root: tempfile::TempDir,
    store: InstallJobStore,
    name: String,
}
impl Fixture {
    fn new() -> Self {
        Self::with_ui(false)
    }
    fn with_ui(real_ui: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        let node = which::which("node").unwrap().canonicalize().unwrap();
        let cli = root.path().join("npm.mjs");
        std::fs::write(&cli, include_str!("volume_fixture.mjs")).unwrap();
        let repository = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned();
        let builder = if real_ui {
            repository.join("scripts/extensions/ui-build.mjs")
        } else {
            cli.clone()
        };
        let app = crate::app_exit::AppExitCoordinator::initialize().unwrap();
        let mut store = InstallJobStore::new(
            super::super::work_supervision::ExtensionWorkServices::new(app.work_supervisor()),
            Some(super::executor::ProductionExecutor::for_test(
                super::super::npm_runner::NpmRunner::for_test(node.clone(), cli.clone()),
                super::super::ui_builder::UiBuildRuntime {
                    node,
                    builder,
                    directory: repository,
                },
            )),
            None,
        )
        .restore(root.path().join("jobs.json"));
        store.disk_policy = super::disk_policy::DiskPolicy {
            warning_bytes: if real_ui { 1 } else { 1024 },
            reserve_bytes: 1024,
            poll_interval: Duration::from_millis(10),
        };
        Self {
            root,
            store,
            name: format!("test-volume-{}", uuid::Uuid::new_v4().simple()),
        }
    }
    async fn wait(&self, id: &str, wanted: InstallStatus) -> InstallJobView {
        tokio::time::timeout(Duration::from_secs(8), async {
            loop {
                let view = self
                    .store
                    .snapshot()
                    .unwrap()
                    .jobs
                    .into_iter()
                    .find(|job| job.id == id)
                    .unwrap();
                if view.status == wanted {
                    return view;
                }
                assert!(
                    !view.status.terminal(),
                    "unexpected {:?}: {:?}",
                    view.status,
                    view.error_code
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }
}

#[tokio::test]
async fn local_ui_volume_can_continue_or_cancel_without_modifying_user_source() {
    for cancel in [false, true] {
        let fixture = Fixture::with_ui(true);
        let source = fixture.root.path().join("source");
        std::fs::create_dir(&source).unwrap();
        let repository = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned();
        let original = repository.join("scripts/extensions/fixtures/ui/advanced-valid");
        for name in ["index.mjs", "entry.ts", "style.css"] {
            std::fs::copy(original.join(name), source.join(name)).unwrap();
        }
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(original.join("beaver-extension.json")).unwrap())
                .unwrap();
        manifest["id"] = fixture.name.clone().into();
        std::fs::write(
            source.join("beaver-extension.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let before = std::fs::read(source.join("entry.ts")).unwrap();
        let job = fixture
            .store
            .start(InstallRequest::Local {
                path: source.to_str().unwrap().into(),
            })
            .unwrap();
        let waiting = fixture
            .wait(&job.id, InstallStatus::AwaitingConfirmation)
            .await;
        assert_eq!(waiting.phase, InstallPhase::BuildingUi);
        if cancel {
            fixture.store.request_cancel(&job.id).unwrap();
        } else {
            fixture
                .store
                .confirm(&job.id, waiting.confirmation_id.as_deref().unwrap())
                .unwrap();
        }
        fixture
            .wait(
                &job.id,
                if cancel {
                    InstallStatus::Cancelled
                } else {
                    InstallStatus::Completed
                },
            )
            .await;
        assert!(
            fixture
                .store
                .work
                .stop_and_wait(Instant::now() + Duration::from_secs(2))
                .await
        );
        assert_eq!(std::fs::read(source.join("entry.ts")).unwrap(), before);
        assert!(!source.join("node_modules").exists());
        if cancel {
            assert!(super::super::registry::find(&fixture.name).is_err());
        } else {
            let record = super::super::registry::find(&fixture.name).unwrap();
            super::super::ui_artifact::validate_record(&record).unwrap();
            super::super::registry::remove(&fixture.name).unwrap();
            super::super::ui_artifact_store::remove(&record).unwrap();
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.store.work.begin_closing();
    }
}

#[tokio::test]
async fn volume_confirmation_has_no_writer_replays_locked_phase_and_rejects_changed_lock() {
    for corrupt_lock in [false, true] {
        let fixture = Fixture::new();
        let first = fixture
            .store
            .start(InstallRequest::Npm {
                locator: fixture.name.clone(),
            })
            .unwrap();
        let second = fixture
            .store
            .start(InstallRequest::Npm {
                locator: "queued-volume-fixture".into(),
            })
            .unwrap();
        let waiting = fixture
            .wait(&first.id, InstallStatus::AwaitingConfirmation)
            .await;
        let checkpoint = {
            let state = fixture.store.lock().unwrap();
            state.jobs[state.index(&first.id).unwrap()]
                .checkpoint
                .clone()
                .unwrap()
        };
        assert!(checkpoint.native_process.is_none());
        let pid: u32 = std::fs::read_to_string(fixture.root.path().join("pid"))
            .unwrap()
            .parse()
            .unwrap();
        assert!(!crate::services::owned_process::OwnedProcess::process_exists(pid));
        let occupied = super::disk_usage::measure(&checkpoint).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(super::disk_usage::measure(&checkpoint).unwrap(), occupied);
        let queued = fixture
            .store
            .snapshot()
            .unwrap()
            .jobs
            .into_iter()
            .find(|job| job.id == second.id)
            .unwrap();
        assert_eq!(
            queued.queue_blocker,
            Some(QueueBlocker::Confirmation {
                job_id: first.id.clone()
            })
        );
        fixture.store.request_cancel(&second.id).unwrap();
        let prompt = waiting.confirmation_id.unwrap();
        assert!(fixture
            .store
            .confirm(&first.id, &uuid::Uuid::new_v4().to_string())
            .is_err());
        if corrupt_lock {
            let lock = super::super::managed_store::root()
                .join(format!(".staging-{}", checkpoint.token))
                .join("package-lock.json");
            std::fs::write(lock, "{}").unwrap();
        }
        fixture.store.confirm(&first.id, &prompt).unwrap();
        assert!(fixture.store.confirm(&first.id, &prompt).is_err());
        let status = if corrupt_lock {
            InstallStatus::Failed
        } else {
            InstallStatus::Completed
        };
        fixture.wait(&first.id, status).await;
        assert!(
            fixture
                .store
                .work
                .stop_and_wait(Instant::now() + Duration::from_secs(2))
                .await
        );
        if corrupt_lock {
            assert!(super::super::registry::find(&fixture.name).is_err());
        } else {
            let record = super::super::registry::find(&fixture.name).unwrap();
            let installed = super::super::managed_store::install_root(&record).unwrap();
            assert!(
                super::super::managed_tree::measure_with_budget(&installed, u64::MAX).unwrap()
                    > 1024
            );
            let state = fixture.store.lock().unwrap();
            let saved = state.jobs[state.index(&first.id).unwrap()]
                .checkpoint
                .as_ref()
                .unwrap();
            assert!(saved.allowance.confirmation_used);
            assert!(saved.allowance.approved_total_bytes > 1024);
            drop(state);
            super::super::registry::remove(&fixture.name).unwrap();
            super::super::managed_store::remove_record(&record).unwrap();
        }
    }
}
#[tokio::test]
async fn disk_space_lost_after_consent_stops_instead_of_asking_again() {
    let mut fixture = Fixture::new();
    let free = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1024 * 1024));
    fixture.store.free_bytes_for_test = Some(free.clone());
    let job = fixture
        .store
        .start(InstallRequest::Npm {
            locator: fixture.name.clone(),
        })
        .unwrap();
    let waiting = fixture
        .wait(&job.id, InstallStatus::AwaitingConfirmation)
        .await;
    fixture
        .store
        .confirm(&job.id, waiting.confirmation_id.as_deref().unwrap())
        .unwrap();
    free.store(0, std::sync::atomic::Ordering::SeqCst);
    let failed = fixture.wait(&job.id, InstallStatus::Failed).await;
    assert_eq!(
        failed.error_code.as_deref(),
        Some(super::limits::INSUFFICIENT_SPACE)
    );
    assert!(super::super::registry::find(&fixture.name).is_err());
    assert!(
        fixture
            .store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}

#[tokio::test]
async fn closing_during_volume_wait_interrupts_without_implicit_consent() {
    let fixture = Fixture::new();
    let job = fixture
        .store
        .start(InstallRequest::Npm {
            locator: fixture.name.clone(),
        })
        .unwrap();
    fixture
        .wait(&job.id, InstallStatus::AwaitingConfirmation)
        .await;
    assert!(
        fixture
            .store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    let view = fixture
        .store
        .snapshot()
        .unwrap()
        .jobs
        .into_iter()
        .find(|view| view.id == job.id)
        .unwrap();
    assert_eq!(view.status, InstallStatus::Interrupted);
    assert!(view.confirmation_id.is_none());
    let state = fixture.store.lock().unwrap();
    let checkpoint = state.jobs[state.index(&job.id).unwrap()]
        .checkpoint
        .as_ref()
        .unwrap();
    assert!(!checkpoint.allowance.confirmation_used);
    assert!(checkpoint.native_process.is_none());
    assert!(super::super::registry::find(&fixture.name).is_err());
}

#[cfg(unix)]
#[tokio::test]
async fn invalid_disk_measurement_refuses_consent_and_preserves_the_link_target() {
    let fixture = Fixture::new();
    let job = fixture
        .store
        .start(InstallRequest::Npm {
            locator: fixture.name.clone(),
        })
        .unwrap();
    let waiting = fixture
        .wait(&job.id, InstallStatus::AwaitingConfirmation)
        .await;
    let token = fixture.store.lock().unwrap().jobs[0]
        .checkpoint
        .as_ref()
        .unwrap()
        .token
        .clone();
    let staging = super::super::managed_store::root().join(format!(".staging-{token}"));
    std::os::unix::fs::symlink(fixture.root.path(), staging.join("outside")).unwrap();
    assert!(fixture
        .store
        .confirm(&job.id, waiting.confirmation_id.as_deref().unwrap())
        .is_err());
    assert_eq!(
        fixture.store.snapshot().unwrap().jobs[0].status,
        InstallStatus::AwaitingConfirmation
    );
    fixture.store.request_cancel(&job.id).unwrap();
    fixture.wait(&job.id, InstallStatus::Cancelled).await;
    assert!(fixture.root.path().join("npm.mjs").exists());
    assert!(!staging.exists());
    assert!(
        fixture
            .store
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}
