use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    install_archive, installed_receipt, receipt_tests::raw_entry, remove_installation,
    resume_incomplete_removals, RemovalGate,
};

struct Gate(AtomicBool);

impl RemovalGate for Gate {
    fn release_for_removal(&self, _model_id: &str) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[test]
fn startup_removes_model_revisions_not_named_by_the_receipt() {
    let data = tempfile::tempdir().unwrap();
    let archive = data.path().join("silero.part");
    std::fs::write(&archive, b"vad").unwrap();
    let entry = raw_entry(b"vad");
    let receipt = install_archive(&entry, &archive, data.path()).unwrap();
    let obsolete = receipt
        .install_dir(data.path())
        .parent()
        .unwrap()
        .join("obsolete-revision");
    std::fs::create_dir_all(&obsolete).unwrap();
    std::fs::write(obsolete.join("old.onnx"), b"old").unwrap();

    resume_incomplete_removals(data.path()).unwrap();

    assert!(!obsolete.exists());
    assert!(receipt.install_dir(data.path()).is_dir());
}

#[cfg(unix)]
#[test]
fn startup_preserves_the_installed_legacy_silero_revision() {
    let data = tempfile::tempdir().unwrap();
    let receipt: super::InstallationReceipt =
        serde_json::from_str(include_str!("fixtures/silero-vad-legacy-receipt.json")).unwrap();
    super::receipt::save_receipt(data.path(), &receipt).unwrap();
    let legacy = data
        .path()
        .join("voice-models/models/silero-vad")
        .join(&receipt.revision);
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("silero_vad.onnx"), b"legacy model").unwrap();

    resume_incomplete_removals(data.path()).unwrap();

    assert_eq!(receipt.install_dir(data.path()), legacy);
    assert!(legacy.join("silero_vad.onnx").is_file());
}

#[test]
fn busy_model_is_untouched_then_free_model_loses_receipt_before_files() {
    let data = tempfile::tempdir().unwrap();
    let archive = data.path().join("silero.part");
    std::fs::write(&archive, b"vad").unwrap();
    let entry = raw_entry(b"vad");
    install_archive(&entry, &archive, data.path()).unwrap();
    let gate = Gate(AtomicBool::new(false));

    assert_eq!(
        remove_installation(data.path(), &entry.id, &gate).unwrap_err(),
        "model-download-model-busy"
    );
    assert!(installed_receipt(&entry, data.path()).unwrap().is_some());

    gate.0.store(true, Ordering::Release);
    remove_installation(data.path(), &entry.id, &gate).unwrap();
    assert!(installed_receipt(&entry, data.path()).unwrap().is_none());
}
