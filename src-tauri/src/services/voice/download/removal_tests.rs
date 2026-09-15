use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    install_archive, installed_receipt, receipt_tests::raw_entry, remove_installation, RemovalGate,
};

struct Gate(AtomicBool);

impl RemovalGate for Gate {
    fn release_for_removal(&self, _model_id: &str) -> bool {
        self.0.load(Ordering::Acquire)
    }
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
