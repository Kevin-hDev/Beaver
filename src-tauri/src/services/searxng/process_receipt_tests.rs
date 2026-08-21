use std::time::{Duration, Instant};

use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity, OwnedProcessInspection};

use super::process_receipt::{
    classify_recovery, RecoveryOutcome, SearxngProcessReceipt, SearxngProcessReceiptStore,
};

fn identity() -> OwnedProcessIdentity {
    OwnedProcessIdentity {
        pid: 42,
        native_start_time: 7,
        native_scope: 9,
        executable: (u128::from(11_u64) << 64) | u128::from(13_u64),
    }
}

fn store() -> (tempfile::TempDir, SearxngProcessReceiptStore) {
    let root = tempfile::tempdir().expect("temporary receipt root");
    let path = root.path().join("searxng-sidecar.pid");
    let store = SearxngProcessReceiptStore::at(path);
    (root, store)
}

#[test]
fn receipt_refuses_oversized_unknown_duplicate_and_zero_fields() {
    let (_root, store) = store();
    std::fs::write(store.path(), vec![b'x'; 4_097]).expect("oversized receipt");
    assert!(store.read().is_err());

    std::fs::write(
        store.path(),
        br#"{"schema_version":2,"pid":42,"native_start_time":7,"native_scope":9,"executable_high":11,"executable_low":13,"pending":false,"unknown":1}"#,
    )
    .expect("unknown field receipt");
    assert!(store.read().is_err());

    std::fs::write(
        store.path(),
        br#"{"schema_version":2,"pid":42,"pid":42,"native_start_time":7,"native_scope":9,"executable_high":11,"executable_low":13,"pending":false}"#,
    )
    .expect("duplicate receipt");
    assert!(store.read().is_err());

    std::fs::write(
        store.path(),
        br#"{"schema_version":2,"pid":0,"native_start_time":7,"native_scope":9,"executable_high":0,"executable_low":11,"pending":false}"#,
    )
    .expect("zero field receipt");
    assert!(store.read().is_err());
}

#[test]
fn receipt_writes_atomically_and_round_trips_the_exact_identity() {
    let (_root, store) = store();
    store.write(&identity()).expect("receipt write");

    assert!(
        std::fs::metadata(store.path())
            .expect("receipt metadata")
            .len()
            < 4_096
    );
    assert_eq!(
        store.read().expect("receipt read"),
        SearxngProcessReceipt::from_identity(identity())
    );
}

#[test]
fn pending_receipt_recovers_the_same_process_after_its_executable_changes() {
    let initial = identity();
    let current = OwnedProcessIdentity {
        executable: (u128::from(17_u64) << 64) | u128::from(19_u64),
        ..initial
    };
    let receipt = SearxngProcessReceipt::pending(initial);
    let mut recovered = None;

    let outcome = classify_recovery(
        receipt,
        |_| true,
        |_| false,
        |_, _| Ok(OwnedProcessInspection::Owned(current)),
        |identity, _| {
            recovered = Some(identity);
            Ok(())
        },
        Instant::now(),
    );

    assert_eq!(outcome, RecoveryOutcome::Exact);
    assert_eq!(recovered, Some(current));
}

#[test]
fn pending_receipt_is_durable_before_identity_stabilization() {
    let (_root, store) = store();

    store
        .write_pending(&identity())
        .expect("pending receipt write");

    assert_eq!(
        store.read().unwrap(),
        SearxngProcessReceipt::pending(identity())
    );
}

#[test]
fn stale_exact_and_ambiguous_inspections_have_closed_outcomes() {
    let receipt = SearxngProcessReceipt::from_identity(identity());
    let stale = classify_recovery(
        receipt.clone(),
        |_| true,
        |_| false,
        |_, _| Ok(OwnedProcessInspection::Unowned),
        |_, _| Ok(()),
        Instant::now(),
    );
    assert_eq!(stale, RecoveryOutcome::Stale);

    let reused = OwnedProcessIdentity {
        native_start_time: identity().native_start_time + 1,
        ..identity()
    };
    assert_eq!(
        classify_recovery(
            receipt.clone(),
            |_| true,
            |_| false,
            |_, _| Ok(OwnedProcessInspection::Owned(reused)),
            |_, _| Ok(()),
            Instant::now(),
        ),
        RecoveryOutcome::Stale
    );

    let exact = classify_recovery(
        receipt.clone(),
        |_| true,
        |_| false,
        |_, _| Ok(OwnedProcessInspection::Owned(identity())),
        |_, _| Ok(()),
        Instant::now(),
    );
    assert_eq!(exact, RecoveryOutcome::Exact);

    assert_eq!(
        classify_recovery(
            receipt.clone(),
            |_| true,
            |_| false,
            |_, _| Ok(OwnedProcessInspection::Owned(identity())),
            |_, _| Err(()),
            Instant::now(),
        ),
        RecoveryOutcome::Blocked
    );

    let blocked = classify_recovery(
        receipt,
        |_| true,
        |_| false,
        |_, _| Err(()),
        |_, _| Ok(()),
        Instant::now(),
    );
    assert_eq!(blocked, RecoveryOutcome::Blocked);

    assert_eq!(
        classify_recovery(
            SearxngProcessReceipt::from_identity(identity()),
            |_| true,
            |_| true,
            |_, _| Err(()),
            |_, _| panic!("an exited child must not be signalled again"),
            Instant::now(),
        ),
        RecoveryOutcome::Exact
    );
}

#[test]
fn legacy_numeric_receipt_is_removed_without_process_recovery() {
    let (_root, store) = store();
    std::fs::write(store.path(), b"4242\n").expect("legacy receipt");

    assert_eq!(
        store
            .recover_and_reap(Instant::now() + Duration::from_secs(1))
            .expect("legacy recovery"),
        RecoveryOutcome::LegacyUnverifiable
    );
    assert!(!store.path().exists());
}

#[test]
fn invalid_json_blocks_recovery_and_remains_available_for_diagnosis() {
    let (_root, store) = store();
    std::fs::write(store.path(), b"{").expect("invalid receipt");

    assert!(store
        .recover_and_reap(Instant::now() + Duration::from_secs(1))
        .is_err());
    assert!(store.path().exists());
}

#[cfg(unix)]
#[tokio::test]
async fn exact_receipt_reaps_a_real_owned_python_process() {
    let (_root, store) = store();
    let child = super::process::spawn_test_fixture()
        .await
        .expect("owned Python fixture");
    let pid = child.id().expect("fixture pid");
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().expect("coordinator");
    let supervisor = crate::services::work_registry::ServiceWorkSupervisor::<1>::new(
        coordinator.work_supervisor(),
    );
    let admission = supervisor.try_admit().expect("admission");
    let identity = super::process::stable_identity(
        pid,
        tokio::time::Instant::now() + Duration::from_millis(250),
        &admission.cancellation(),
    )
    .await
    .expect("fixture identity");
    assert_eq!(
        OwnedProcess::inspect_for_recovery(pid, identity.native_start_time)
            .expect("fixture inspection"),
        OwnedProcessInspection::Owned(identity)
    );
    store.write(&identity).expect("fixture receipt");
    assert!(OwnedProcess::process_exists(pid));
    assert_eq!(
        store.read().expect("stored fixture receipt"),
        SearxngProcessReceipt::from_identity(identity)
    );
    assert_eq!(
        OwnedProcess::inspect_for_recovery(pid, identity.native_start_time)
            .expect("second fixture inspection"),
        OwnedProcessInspection::Owned(identity)
    );
    assert_eq!(
        classify_recovery(
            store.read().expect("receipt for inspection"),
            OwnedProcess::process_exists,
            |_| false,
            |pid, started| OwnedProcess::inspect_for_recovery(pid, started).map_err(|_| ()),
            |_, _| Ok(()),
            Instant::now() + Duration::from_secs(2),
        ),
        RecoveryOutcome::Exact
    );

    assert_eq!(
        store
            .recover_and_reap(Instant::now() + Duration::from_secs(2))
            .expect("exact recovery"),
        RecoveryOutcome::Exact
    );
    assert!(!OwnedProcess::process_exists(pid));
    assert!(!store.path().exists());
}

#[cfg(unix)]
#[tokio::test]
async fn cancelled_recovery_keeps_the_receipt_and_process_for_a_safe_retry() {
    let (_root, store) = store();
    let mut child = super::process::spawn_test_fixture().await.unwrap();
    let pid = child.id().unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let supervisor = crate::services::work_registry::ServiceWorkSupervisor::<1>::new(
        coordinator.work_supervisor(),
    );
    let admission = supervisor.try_admit().unwrap();
    let identity = super::process::stable_identity(
        pid,
        tokio::time::Instant::now() + Duration::from_millis(250),
        &admission.cancellation(),
    )
    .await
    .unwrap();
    store.write(&identity).unwrap();

    let outcome = store
        .recover_and_reap_with(Instant::now() + Duration::from_secs(1), || true)
        .unwrap();

    assert_eq!(outcome, RecoveryOutcome::Blocked);
    assert!(store.path().exists());
    assert!(OwnedProcess::process_exists(pid));
    crate::services::process_tree::terminate_tokio(
        &mut child,
        crate::services::process_tree::ProcessKind::Searxng,
    )
    .await;
    store.remove().unwrap();
}
