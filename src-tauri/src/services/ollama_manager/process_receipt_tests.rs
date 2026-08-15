use super::durable_fs::platform_fs;
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
#[cfg(unix)]
use super::process::DefaultOllamaProcessLauncher;
use super::process_receipt::{
    ProcessReceipt, ProcessReceiptError, ProcessReceiptRecovery, ProcessReceiptStore, RecoveryProbe,
};
use crate::services::paths::ollama_paths;
use std::sync::Arc;
#[cfg(unix)]
use std::time::{Duration, Instant};

fn receipt() -> ProcessReceipt {
    ProcessReceipt::new(
        42,
        99,
        7,
        BundleFingerprint {
            version: OllamaVersion::parse("1.2.3").expect("version"),
            executable_sha256: Sha256Digest::from_hex(&"ab".repeat(32)).expect("digest"),
        },
    )
    .expect("receipt")
}

#[test]
fn process_receipt_round_trip_is_durable_and_bounded() {
    let root = tempfile::tempdir().expect("tempdir");
    let paths = ollama_paths(root.path());
    let path = paths.process_receipt.clone();
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        path.clone(),
        path.with_extension("tmp"),
    );
    store.write_new(&receipt()).expect("write receipt");
    assert_eq!(store.read().expect("read receipt"), Some(receipt()));
    store.remove().expect("remove receipt");
    assert_eq!(store.read().expect("missing receipt"), None);
}

#[test]
fn process_receipt_rejects_unknown_fields_and_oversized_documents() {
    let bytes = br#"{"schema_version":1,"pid":42,"native_start_time":99,"native_scope":7,"bundle":{"version":"1.2.3","executable_sha256":"abababababababababababababababababababababababababababababababab"},"extra":true}"#;
    let nested = br#"{"schema_version":1,"pid":42,"native_start_time":99,"native_scope":7,"bundle":{"version":"1.2.3","executable_sha256":"abababababababababababababababababababababababababababababababab","extra":true}}"#;
    for document in [bytes.as_slice(), nested.as_slice()] {
        assert_eq!(
            ProcessReceipt::parse_bounded(document),
            Err(ProcessReceiptError::Invalid)
        );
    }
    assert_eq!(
        ProcessReceipt::parse_bounded(&vec![b'x'; 4097]),
        Err(ProcessReceiptError::Oversized)
    );
}

#[test]
fn process_receipt_rejects_duplicate_top_level_and_nested_fields() {
    let duplicate_pid = br#"{"schema_version":1,"pid":42,"pid":43,"native_start_time":99,"native_scope":7,"bundle":{"version":"1.2.3","executable_sha256":"abababababababababababababababababababababababababababababababab"}}"#;
    let duplicate_version = br#"{"schema_version":1,"pid":42,"native_start_time":99,"native_scope":7,"bundle":{"version":"1.2.3","version":"1.2.4","executable_sha256":"abababababababababababababababababababababababababababababababab"}}"#;
    let duplicate_hash = br#"{"schema_version":1,"pid":42,"native_start_time":99,"native_scope":7,"bundle":{"version":"1.2.3","executable_sha256":"abababababababababababababababababababababababababababababababab","executable_sha256":"cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd"}}"#;
    for bytes in [
        duplicate_pid.as_slice(),
        duplicate_version.as_slice(),
        duplicate_hash.as_slice(),
    ] {
        assert_eq!(
            ProcessReceipt::parse_bounded(bytes),
            Err(ProcessReceiptError::Invalid)
        );
    }
}

#[test]
fn process_receipt_rejects_zero_identity_fields() {
    let mut value = serde_json::to_value(receipt()).expect("json");
    value["pid"] = serde_json::json!(0);
    let bytes = serde_json::to_vec(&value).expect("json bytes");
    assert_eq!(
        ProcessReceipt::parse_bounded(&bytes),
        Err(ProcessReceiptError::Invalid)
    );
}

#[test]
fn process_receipt_recovery_is_fail_closed_and_keeps_exact_proof() {
    let root = tempfile::tempdir().expect("tempdir");
    let paths = ollama_paths(root.path());
    let path = paths.process_receipt.clone();
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        path.clone(),
        path.with_extension("tmp"),
    );
    let expected = receipt();
    let identity = crate::services::owned_process::OwnedProcessIdentity {
        pid: expected.pid,
        native_scope: expected.native_scope,
        native_start_time: expected.native_start_time,
        executable: 0x1234,
    };
    store.write_new(&expected).expect("write");
    assert!(matches!(
        store
            .recover_identity(&expected.bundle, 0x1234, |_| Ok(identity))
            .expect("exact recovery"),
        ProcessReceiptRecovery::Exact(_)
    ));
    assert!(store.read().expect("retained proof").is_some());
    let exact = match store
        .recover_identity(&expected.bundle, 0x1234, |_| Ok(identity))
        .expect("exact recovery")
    {
        ProcessReceiptRecovery::Exact(receipt) => receipt,
        other => panic!("unexpected recovery state: {other:?}"),
    };
    assert!(store
        .reap_exact(&exact, |_| Err(ProcessReceiptError::Storage))
        .is_err());
    assert!(store.read().expect("kept after failed reap").is_some());
    store
        .reap_exact(&exact, |receipt| {
            assert_eq!(receipt.pid, expected.pid);
            Ok(())
        })
        .expect("reap cleanup");

    store.write_new(&expected).expect("write stale");
    let reused = crate::services::owned_process::OwnedProcessIdentity {
        native_start_time: expected.native_start_time + 1,
        ..identity
    };
    assert_eq!(
        store
            .recover_identity(&expected.bundle, 0x1234, |_| Ok(reused))
            .expect("pid reuse"),
        ProcessReceiptRecovery::StaleRemoved
    );
    assert!(store.read().expect("removed").is_none());

    store.write_new(&expected).expect("write wrong scope");
    let wrong_scope = crate::services::owned_process::OwnedProcessIdentity {
        native_scope: expected.native_scope + 1,
        ..identity
    };
    assert_eq!(
        store
            .recover_identity(&expected.bundle, 0x1234, |_| Ok(wrong_scope))
            .expect("scope mismatch"),
        ProcessReceiptRecovery::StaleRemoved
    );

    store.write_new(&expected).expect("write wrong executable");
    let wrong_executable = crate::services::owned_process::OwnedProcessIdentity {
        executable: 0x5678,
        ..identity
    };
    assert_eq!(
        store
            .recover_identity(&expected.bundle, 0x1234, |_| Ok(wrong_executable))
            .expect("executable mismatch"),
        ProcessReceiptRecovery::StaleRemoved
    );

    store.write_new(&expected).expect("write ambiguous");
    assert_eq!(
        store
            .recover(&expected.bundle, |_| RecoveryProbe::Ambiguous)
            .expect("ambiguous recovery"),
        ProcessReceiptRecovery::RecoveryRequired
    );
    assert!(store.read().expect("kept ambiguous").is_some());
    store.remove().expect("cleanup");

    store.write_new(&expected).expect("write missing");
    assert_eq!(
        store
            .recover_identity(&expected.bundle, 0x1234, |_| Err(RecoveryProbe::Missing))
            .expect("missing recovery"),
        ProcessReceiptRecovery::StaleRemoved
    );
    assert!(store.read().expect("removed missing").is_none());

    store.write_new(&expected).expect("write wrong bundle");
    let other = BundleFingerprint {
        version: OllamaVersion::parse("9.9.9").expect("version"),
        executable_sha256: expected.bundle.executable_sha256.clone(),
    };
    let inspected = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let inspected_by_probe = std::sync::Arc::clone(&inspected);
    assert_eq!(
        store
            .recover(&other, |_| {
                inspected_by_probe.store(true, std::sync::atomic::Ordering::SeqCst);
                RecoveryProbe::Ambiguous
            })
            .expect("stale bundle"),
        ProcessReceiptRecovery::RecoveryRequired
    );
    assert!(inspected.load(std::sync::atomic::Ordering::SeqCst));
    store.remove().expect("cleanup ambiguous stale receipt");

    store.write_new(&expected).expect("write wrong hash");
    let other_hash = BundleFingerprint {
        version: expected.bundle.version.clone(),
        executable_sha256: Sha256Digest::from_hex(&"cd".repeat(32)).expect("digest"),
    };
    assert_eq!(
        store
            .recover(&other_hash, |_| RecoveryProbe::Ambiguous)
            .expect("stale hash"),
        ProcessReceiptRecovery::RecoveryRequired
    );
    assert!(store.read().expect("retained hash mismatch").is_some());
    store.remove().expect("cleanup hash mismatch");
}

#[cfg(unix)]
#[test]
fn production_recovery_reaps_exact_process_before_removing_receipt() {
    let root = tempfile::tempdir().expect("root");
    let paths = ollama_paths(root.path());
    let path = paths.process_receipt.clone();
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        path.clone(),
        path.with_extension("tmp"),
    );
    let mut command = std::process::Command::new("/bin/sleep");
    command
        .arg("30")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        crate::services::process_tree::ProcessKind::Ollama,
    )
    .expect("child");
    let identity =
        crate::services::owned_process::OwnedProcess::identity(child.id()).expect("identity");
    let expected = ProcessReceipt::new(
        identity.pid,
        identity.native_start_time,
        identity.native_scope,
        receipt().bundle,
    )
    .expect("receipt");
    store.write_new(&expected).expect("write");
    let launcher = DefaultOllamaProcessLauncher::new(expected.bundle.clone());
    assert_eq!(
        launcher
            .recover_receipt(
                &store,
                identity.executable,
                Instant::now() + Duration::from_secs(2),
            )
            .expect("recovery"),
        ProcessReceiptRecovery::Reaped
    );
    assert!(store.read().expect("removed receipt").is_none());
    let _ = child.wait();
}

#[cfg(unix)]
#[test]
fn production_recovery_inspects_exact_process_before_bundle_mismatch_removal() {
    let root = tempfile::tempdir().expect("root");
    let paths = ollama_paths(root.path());
    let path = paths.process_receipt.clone();
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        path.clone(),
        path.with_extension("tmp"),
    );
    let mut command = std::process::Command::new("/bin/sleep");
    command
        .arg("30")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        crate::services::process_tree::ProcessKind::Ollama,
    )
    .expect("child");
    let identity =
        crate::services::owned_process::OwnedProcess::identity(child.id()).expect("identity");
    let recorded = receipt();
    store
        .write_new(
            &ProcessReceipt::new(
                identity.pid,
                identity.native_start_time,
                identity.native_scope,
                recorded.bundle.clone(),
            )
            .expect("receipt"),
        )
        .expect("write");
    let active = BundleFingerprint {
        version: OllamaVersion::parse("9.9.9").expect("version"),
        executable_sha256: recorded.bundle.executable_sha256.clone(),
    };
    let launcher = DefaultOllamaProcessLauncher::new(active);
    assert_eq!(
        launcher
            .recover_receipt(&store, identity.executable, Instant::now())
            .expect("inspect"),
        ProcessReceiptRecovery::RecoveryRequired
    );
    assert!(store.read().expect("retained receipt").is_some());
    crate::services::owned_process::OwnedProcess::recover_exact(
        identity,
        Instant::now() + Duration::from_secs(2),
    )
    .expect("cleanup");
    store.remove().expect("remove");
    let _ = child.wait();
}
