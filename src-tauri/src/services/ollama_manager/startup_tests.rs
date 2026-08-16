use super::bundle_receipt::{self, BundleReceipt};
use super::durable_fs::platform_fs;
use super::error::OllamaErrorCode;
use super::fingerprint::{BundleFingerprint, OllamaVersion};
use super::journal::OllamaMigrationMarker;
use super::manager::OllamaManager;
use super::process_receipt::{ProcessReceipt, ProcessReceiptStore};
use super::recovery_decision::{decide_recovery, JournalPresence, RecoveryDecision};
use super::startup::{OllamaStartupBarrier, StartupBarrierState};
use super::types::BundleState;
use crate::app_exit::AppExitCoordinator;
use crate::services::paths::{ollama_paths, OllamaPaths};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::Duration;

fn complete_active_bundle(paths: &OllamaPaths) -> BundleFingerprint {
    let bin = paths.active.join("bin");
    fs::create_dir_all(&bin).expect("active bin");
    let executable = bin.join(if cfg!(windows) {
        "ollama.exe"
    } else {
        "ollama"
    });
    fs::write(&executable, b"#!/bin/sh\nexit 0\n").expect("test executable");
    #[cfg(unix)]
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))
        .expect("executable permissions");
    let fingerprint = BundleFingerprint {
        version: OllamaVersion::parse("1.2.3").expect("version"),
        executable_sha256: super::probe_http::hash_file(&executable)
            .ok()
            .expect("hash"),
    };
    let durable = platform_fs();
    bundle_receipt::write_version(&durable, &paths.active, "1.2.3").expect("version receipt");
    bundle_receipt::write_receipt(
        &durable,
        &paths.active,
        &BundleReceipt::new(fingerprint.clone()),
    )
    .expect("bundle receipt");
    fs::write(
        &paths.migration_marker,
        serde_json::to_vec(&OllamaMigrationMarker::new()).expect("marker bytes"),
    )
    .expect("migration marker");
    fingerprint
}

#[tokio::test]
async fn startup_waits_while_pending_and_opens_once_ready() {
    let barrier = OllamaStartupBarrier::new();
    assert_eq!(barrier.state(), StartupBarrierState::Pending);
    let wait = tokio::spawn({
        let barrier = barrier.clone();
        async move { barrier.wait_ready().await }
    });
    tokio::task::yield_now().await;
    assert!(!wait.is_finished());
    barrier.publish(StartupBarrierState::Ready);
    assert_eq!(barrier.wait_ready().await, StartupBarrierState::Ready);
    assert_eq!(wait.await.unwrap(), StartupBarrierState::Ready);
}

#[test]
fn blocked_state_preserves_recovery_code() {
    let barrier = OllamaStartupBarrier::new();
    barrier.publish(StartupBarrierState::Blocked {
        code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
    });
    assert_eq!(
        barrier.state(),
        StartupBarrierState::Blocked {
            code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
        }
    );
}

#[tokio::test]
async fn polling_waits_for_pending_and_blocked_is_not_ready() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let pending = tokio::time::timeout(Duration::from_millis(10), manager.poll_once()).await;
    assert!(pending.is_err());
    manager.publish_startup_for_test(
        manager.generation_for_test(),
        StartupBarrierState::Blocked {
            code: OllamaErrorCode::OllamaRecoveryDeferred,
        },
    );
    let blocked = tokio::time::timeout(Duration::from_millis(10), manager.poll_once())
        .await
        .expect("blocked polling returns");
    assert_eq!(blocked.last_error, None);
}

#[test]
fn stale_generation_cannot_replace_newer_barrier_state() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let old_generation = manager.generation_for_test();
    manager.set_generation_for_test(old_generation + 1);
    manager.publish_startup_for_test(old_generation, StartupBarrierState::Ready);
    assert_eq!(manager.startup_state(), StartupBarrierState::Pending);
    manager.publish_startup_for_test(old_generation + 1, StartupBarrierState::Ready);
    assert_eq!(manager.startup_state(), StartupBarrierState::Ready);
}

#[tokio::test]
async fn empty_layout_opens_startup_with_bundle_absent() {
    let root = tempfile::tempdir_in(".").expect("data root");
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());

    let state = manager
        .run_startup_recovery_at_for_test(ollama_paths(root.path()))
        .await;

    assert_eq!(state, StartupBarrierState::Ready);
    assert_eq!(manager.status().await.bundle, BundleState::Absent);
}

#[tokio::test]
async fn startup_removes_a_stale_process_receipt_before_publishing_ready() {
    let root = tempfile::tempdir_in(".").expect("data root");
    let paths = ollama_paths(root.path());
    let fingerprint = complete_active_bundle(&paths);
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        paths.process_receipt.clone(),
        paths.process_receipt.with_extension("tmp"),
    );
    store
        .write_new(
            &ProcessReceipt::new(std::process::id(), 1, 1, fingerprint)
                .expect("stale process receipt"),
        )
        .expect("write stale process receipt");
    let snapshot = super::cleanup::snapshot(JournalPresence::Absent, &platform_fs(), &paths);
    assert_eq!(
        decide_recovery(&snapshot),
        RecoveryDecision::Ready,
        "snapshot: {snapshot:?}"
    );
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());

    let state = manager.run_startup_recovery_at_for_test(paths).await;

    assert_eq!(state, StartupBarrierState::Ready);
    assert_eq!(manager.status().await.bundle, BundleState::Ready);
    assert!(store.read().expect("receipt state").is_none());
}
