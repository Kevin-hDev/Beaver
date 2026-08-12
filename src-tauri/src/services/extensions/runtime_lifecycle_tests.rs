use super::super::host_paths::HostPaths;
use super::super::types::ExtensionHostStatus;
use super::*;
use std::time::Duration;
use tokio::sync::Mutex;

#[tokio::test]
async fn internal_start_cannot_bypass_closed_admission() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("host.mjs");
    std::fs::write(
        &script,
        "import { writeFileSync } from 'node:fs'; writeFileSync('started', 'yes');",
    )
    .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    work.begin_closing();
    let runtime = ExtensionRuntime {
        paths: Some(HostPaths {
            node: which::which("node").unwrap().canonicalize().unwrap(),
            script,
            directory: directory.path().to_path_buf(),
        }),
        process: Mutex::new(None),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        auto_restarts: super::super::runtime_restart::RestartBudget::default(),
        work,
    };

    assert_eq!(
        runtime.start_untracked().await,
        Err(error_codes::HOST_UNAVAILABLE.to_string())
    );
    assert!(!directory.path().join("started").exists());
    assert!(runtime.process.lock().await.is_none());
}

#[tokio::test]
async fn stop_runtime_reaps_real_host_reader_and_closes_admission() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("host.mjs");
    std::fs::write(&script, "setInterval(() => {}, 1000);").unwrap();
    let paths = HostPaths {
        node: which::which("node").unwrap().canonicalize().unwrap(),
        script,
        directory: directory.path().to_path_buf(),
    };
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let host = Arc::new(HostProcess::spawn(&paths, &work).await.unwrap());
    let runtime = ExtensionRuntime {
        paths: Some(paths),
        process: Mutex::new(Some(host)),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        auto_restarts: super::super::runtime_restart::RestartBudget::default(),
        work,
    };

    assert!(stop_runtime(&runtime, Instant::now() + Duration::from_secs(5)).await);
    assert!(runtime.process.lock().await.is_none());
    assert_eq!(
        runtime.work.reader_phase(),
        crate::services::work_registry::ServiceWorkPhase::Closed
    );
}

#[tokio::test]
async fn stop_host_respects_the_absolute_deadline_while_process_is_locked() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let runtime = ExtensionRuntime {
        paths: None,
        process: Mutex::new(None),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        auto_restarts: super::super::runtime_restart::RestartBudget::default(),
        work: super::super::work_supervision::ExtensionWorkServices::new(
            coordinator.work_supervisor(),
        ),
    };
    let _guard = runtime.process.lock().await;

    assert!(
        !runtime
            .stop_host(Instant::now() + Duration::from_millis(20))
            .await
    );
}

#[tokio::test]
async fn incomplete_stop_keeps_the_existing_host_in_its_slot() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("host.mjs");
    std::fs::write(&script, "setInterval(() => {}, 1000);").unwrap();
    let paths = HostPaths {
        node: which::which("node").unwrap().canonicalize().unwrap(),
        script,
        directory: directory.path().to_path_buf(),
    };
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let host = Arc::new(HostProcess::spawn(&paths, &work).await.unwrap());
    let runtime = ExtensionRuntime {
        paths: Some(paths),
        process: Mutex::new(Some(Arc::clone(&host))),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        auto_restarts: super::super::runtime_restart::RestartBudget::default(),
        work,
    };

    // Force kill() past its deadline instead of assuming an already-expired timer wins
    // against an immediately-ready process operation on every platform.
    let child_guard = host.hold_child_for_test().await;
    assert!(
        !runtime
            .stop_host(Instant::now() + Duration::from_millis(20))
            .await
    );
    let retained = runtime
        .process
        .lock()
        .await
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, &host));
    drop(child_guard);
    let _ = host.kill(Instant::now() + Duration::from_secs(5)).await;
    assert!(retained, "an unconfirmed host must still own its slot");
}

#[tokio::test]
async fn stale_stop_request_never_clears_a_newer_host() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("host.mjs");
    std::fs::write(&script, "setInterval(() => {}, 1000);").unwrap();
    let paths = HostPaths {
        node: which::which("node").unwrap().canonicalize().unwrap(),
        script,
        directory: directory.path().to_path_buf(),
    };
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let stale = Arc::new(HostProcess::spawn(&paths, &work).await.unwrap());
    assert!(stale.kill(Instant::now() + Duration::from_secs(5)).await);
    let current = Arc::new(HostProcess::spawn(&paths, &work).await.unwrap());
    let slot = Mutex::new(Some(Arc::clone(&current)));

    let outcome =
        stop_host_slot(&slot, Some(&stale), Instant::now() + Duration::from_secs(5)).await;

    assert_eq!(outcome, StopHostOutcome::NotCurrent);
    assert!(
        slot.lock()
            .await
            .as_ref()
            .is_some_and(|host| Arc::ptr_eq(host, &current)),
        "a stale stop request must not release a newer generation"
    );
    let _ = current.kill(Instant::now() + Duration::from_secs(5)).await;
}
