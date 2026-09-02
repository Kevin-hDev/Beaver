use super::super::host_identity::HostIdentity;
use super::super::host_paths::HostPaths;
use super::super::host_process::HostProcess;
use super::super::runtime_hosts::RuntimeHosts;
use super::super::types::{ExtensionApiLevel, ExtensionHostStatus, HostState};
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
        runtime
            .start_untracked(Instant::now() + Duration::from_secs(1))
            .await,
        Err(error_codes::HOST_UNAVAILABLE.to_string())
    );
    assert_eq!(runtime.hosts.lock().await.len(), 0);
}

#[tokio::test]
async fn failed_cautious_retry_never_leaves_the_runtime_stuck_as_starting() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let runtime = runtime(super::super::work_supervision::ExtensionWorkServices::new(
        coordinator.work_supervisor(),
    ));

    assert!(runtime
        .retry_untracked(
            "com.example.missing".to_string(),
            2,
            Instant::now() + Duration::from_secs(1),
        )
        .await
        .is_err());
    assert_eq!(runtime.status.read().unwrap().state, HostState::Error);
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
        work,
    };
    (directory, runtime, process)
}

#[tokio::test]
async fn stop_runtime_reaps_real_host_reader_and_closes_admission() {
    let (_directory, runtime, _process) = runtime_with_real_host().await;

    assert!(stop_runtime(&runtime, Instant::now() + Duration::from_secs(5)).await);
    assert_eq!(runtime.hosts.lock().await.len(), 0);
    assert_eq!(runtime.status.read().unwrap().state, HostState::Stopped);
    assert_eq!(
        runtime.work.reader_phase(),
        crate::services::work_registry::ServiceWorkPhase::Closed
    );
}

#[tokio::test]
async fn spontaneous_process_exit_marks_error_without_a_user_call() {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("crash.mjs");
    std::fs::write(&script, "setTimeout(() => process.exit(23), 50);").unwrap();
    let paths = HostPaths {
        node: which::which("node").unwrap().canonicalize().unwrap(),
        script,
        directory: directory.path().to_path_buf(),
    };
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let work =
        super::super::work_supervision::ExtensionWorkServices::new(coordinator.work_supervisor());
    let (mut hosts, receiver) =
        RuntimeHosts::new_monitored(directory.path().join("channels")).unwrap();
    let identity = HostIdentity::ThirdParty("com.example.crash".to_string());
    let reservation = hosts.reserve(identity.clone()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let process = Arc::new(
        HostProcess::spawn_bound(
            &paths,
            &work,
            reservation.spawn_binding(),
            deadline,
            reservation.temporary_directory(),
        )
        .await
        .unwrap(),
    );
    hosts
        .bind(reservation, ExtensionApiLevel::Stable, process)
        .unwrap();
    let runtime = Arc::new(ExtensionRuntime {
        paths: Some(paths),
        hosts: Mutex::new(hosts),
        sync: Mutex::new(()),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        work,
    });
    runtime.start_exit_monitor(receiver).unwrap();

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if runtime.status.read().unwrap().state == HostState::Error
                && runtime.hosts.lock().await.len() == 0
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("spontaneous exit must be observed");

    runtime.work.begin_closing();
    assert!(
        runtime
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}

#[tokio::test]
async fn an_unconfirmed_stop_retains_the_real_channel() {
    let (_directory, runtime, process) = runtime_with_real_host().await;
    let child_guard = process.hold_child_for_test().await;

    assert_eq!(
        runtime
            .stop_host_if_current(
                &HostIdentity::Official,
                Some(&process),
                Instant::now() + Duration::from_millis(20),
                false,
            )
            .await,
        super::super::runtime::StopHostOutcome::Unconfirmed
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

#[tokio::test]
async fn a_retained_pre_bind_process_is_reaped_after_its_reader_exits() {
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
    let temporary_root = directory.path().join("channels");
    let (mut hosts, receiver) = RuntimeHosts::new_monitored(temporary_root).unwrap();
    let identity = HostIdentity::ThirdParty("com.example.prebind".to_string());
    let reservation = hosts.reserve(identity).unwrap();
    let channel_directory = reservation.temporary_directory().to_path_buf();
    let process = Arc::new(
        HostProcess::spawn_bound(
            &paths,
            &work,
            reservation.spawn_binding(),
            Instant::now() + Duration::from_secs(5),
            reservation.temporary_directory(),
        )
        .await
        .unwrap(),
    );
    hosts.retain_failed(reservation, ExtensionApiLevel::Stable, process);
    let runtime = Arc::new(ExtensionRuntime {
        paths: Some(paths),
        hosts: Mutex::new(hosts),
        sync: Mutex::new(()),
        status: std::sync::RwLock::new(ExtensionHostStatus::default()),
        work,
    });
    runtime.start_exit_monitor(receiver).unwrap();

    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if runtime.hosts.lock().await.len() == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("retained pre-bind process must be reaped");

    assert!(!channel_directory.exists());
    runtime.work.begin_closing();
    assert!(
        runtime
            .work
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
}

#[tokio::test]
async fn confirmed_user_revocation_releases_its_restart_budget() {
    let (_directory, runtime, _process) = runtime_with_real_host().await;
    let identity = HostIdentity::Official;
    {
        let mut hosts = runtime.hosts.lock().await;
        assert!(hosts.allow_restart(&identity));
        assert!(hosts.allow_restart(&identity));
        assert!(hosts.allow_restart(&identity));
        assert!(!hosts.allow_restart(&identity));
    }

    assert!(
        runtime
            .revoke_extension(&identity, Instant::now() + Duration::from_secs(5))
            .await
    );

    assert!(runtime.hosts.lock().await.allow_restart(&identity));
}
