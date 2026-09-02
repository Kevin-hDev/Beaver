use super::super::host_identity::HostIdentity;
use super::super::host_paths::HostPaths;
use super::super::host_process::HostProcess;
use super::super::runtime_hosts::RuntimeHosts;
use super::super::types::{ExtensionApiLevel, ExtensionHostStatus};
use super::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

fn runtime(work: super::super::work_supervision::ExtensionWorkServices) -> ExtensionRuntime {
    let temporary = tempfile::tempdir().unwrap().keep();
    ExtensionRuntime {
        paths: None,
        hosts: Mutex::new(RuntimeHosts::new(temporary).unwrap()),
        sync: Mutex::new(()),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        auto_restarts: super::super::runtime_restart::RestartBudget::default(),
        work,
    }
}

#[tokio::test]
async fn internal_start_cannot_bypass_closed_admission() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    work.begin_closing();
    let runtime = runtime(work);

    assert_eq!(
        runtime.start_untracked().await,
        Err(error_codes::HOST_UNAVAILABLE.to_string())
    );
    assert_eq!(runtime.hosts.lock().await.len(), 0);
}

#[tokio::test]
async fn stop_with_no_channels_is_confirmed_without_waiting() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let runtime = runtime(super::super::work_supervision::ExtensionWorkServices::new(
        coordinator.work_supervisor(),
    ));

    assert!(
        runtime
            .stop_hosts(Instant::now() + Duration::from_millis(20))
            .await
    );
}

async fn runtime_with_real_host() -> (tempfile::TempDir, ExtensionRuntime, Arc<HostProcess>) {
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
    let process = Arc::new(HostProcess::spawn(&paths, &work).await.unwrap());
    let temporary_root = directory.path().join("channels");
    let mut hosts = RuntimeHosts::new(temporary_root).unwrap();
    let reservation = hosts.reserve(HostIdentity::Official).unwrap();
    hosts
        .bind(reservation, ExtensionApiLevel::Stable, Arc::clone(&process))
        .unwrap();
    let runtime = ExtensionRuntime {
        paths: Some(paths),
        hosts: Mutex::new(hosts),
        sync: Mutex::new(()),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        auto_restarts: super::super::runtime_restart::RestartBudget::default(),
        work,
    };
    (directory, runtime, process)
}

#[tokio::test]
async fn stop_runtime_reaps_real_host_reader_and_closes_admission() {
    let (_directory, runtime, _process) = runtime_with_real_host().await;

    assert!(stop_runtime(&runtime, Instant::now() + Duration::from_secs(5)).await);
    assert_eq!(runtime.hosts.lock().await.len(), 0);
    assert_eq!(
        runtime.work.reader_phase(),
        crate::services::work_registry::ServiceWorkPhase::Closed
    );
}

#[tokio::test]
async fn an_unconfirmed_stop_retains_the_real_channel() {
    let (_directory, runtime, process) = runtime_with_real_host().await;
    let child_guard = process.hold_child_for_test().await;

    assert!(
        !runtime
            .stop_channel(&HostIdentity::Official, Some(&process))
            .await
    );
    assert!(!process.is_alive());
    assert!(runtime
        .hosts
        .lock()
        .await
        .snapshot(&HostIdentity::Official)
        .is_some());

    drop(child_guard);
    assert!(process.kill(Instant::now() + Duration::from_secs(5)).await);
}
