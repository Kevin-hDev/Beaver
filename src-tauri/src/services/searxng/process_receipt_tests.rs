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
        br#"{"schema_version":1,"pid":42,"native_start_time":7,"native_scope":9,"executable_high":0,"executable_low":11,"unknown":1}"#,
    )
    .expect("unknown field receipt");
    assert!(store.read().is_err());

    std::fs::write(
        store.path(),
        br#"{"schema_version":1,"pid":42,"pid":0,"native_start_time":7,"native_scope":9,"executable_high":0,"executable_low":11}"#,
    )
    .expect("duplicate receipt");
    assert!(store.read().is_err());

    std::fs::write(
        store.path(),
        br#"{"schema_version":1,"pid":0,"native_start_time":7,"native_scope":9,"executable_high":0,"executable_low":11}"#,
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
    let identity = super::process::stable_identity(pid)
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
