use super::durable_fs::platform_fs;
use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::process_receipt::{
    ProcessReceipt, ProcessReceiptError, ProcessReceiptRecovery, ProcessReceiptStore,
};
use crate::services::paths::ollama_paths;
use std::sync::Arc;
#[cfg(unix)]
use std::time::Duration;
use std::time::Instant;

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
fn production_recovery_handles_missing_and_invalid_receipts() {
    let root = tempfile::tempdir().expect("tempdir");
    let paths = ollama_paths(root.path());
    let path = paths.process_receipt.clone();
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        path.clone(),
        path.with_extension("tmp"),
    );
    assert_eq!(
        store
            .recover_active(&receipt().bundle, 1, Instant::now())
            .expect("missing receipt"),
        ProcessReceiptRecovery::Missing
    );

    std::fs::write(&path, b"invalid receipt").expect("invalid receipt");
    assert_eq!(
        store.recover_active(&receipt().bundle, 1, Instant::now()),
        Err(ProcessReceiptError::Invalid)
    );
}

#[cfg(unix)]
#[test]
fn production_recovery_removes_a_terminated_process_receipt() {
    let root = tempfile::tempdir().expect("root");
    let paths = ollama_paths(root.path());
    let path = paths.process_receipt.clone();
    let store = ProcessReceiptStore::new(
        Arc::new(platform_fs()),
        path.clone(),
        path.with_extension("tmp"),
    );
    let mut command = std::process::Command::new("/bin/sleep");
    command.arg("30");
    let mut child = crate::services::owned_process::OwnedProcess::spawn(
        &mut command,
        crate::services::process_tree::ProcessKind::Ollama,
    )
    .expect("child");
    let identity =
        crate::services::owned_process::OwnedProcess::identity(child.id()).expect("identity");
    child.kill().expect("terminate child");
    child.wait().expect("reap child");
    let terminated = ProcessReceipt::new(
        identity.pid,
        identity.native_start_time,
        identity.native_scope,
        receipt().bundle,
    )
    .expect("receipt");
    store.write_new(&terminated).expect("write terminated");
    assert_eq!(
        store
            .recover_active(&terminated.bundle, identity.executable, Instant::now())
            .expect("terminated process"),
        ProcessReceiptRecovery::StaleRemoved
    );
    assert!(store.read().expect("removed terminated receipt").is_none());
    crate::services::owned_process::release(identity.pid);
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
    assert_eq!(
        store
            .recover_active(
                &expected.bundle,
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
    assert_eq!(
        store
            .recover_active(&active, identity.executable, Instant::now())
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
