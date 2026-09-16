use super::{
    install_archive, installed_receipt, partial_path, receipt::receipt_path,
    receipt_tests::raw_entry,
};

#[test]
fn publishes_files_before_the_receipt_and_cleans_the_partial() {
    let data = tempfile::tempdir().unwrap();
    let archive = partial_path(data.path(), "silero-vad");
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"vad").unwrap();
    let entry = raw_entry(b"vad");

    let receipt = install_archive(&entry, &archive, data.path()).unwrap();

    assert_eq!(
        std::fs::read(receipt.install_dir(data.path()).join("silero_vad.onnx")).unwrap(),
        b"vad"
    );
    assert!(installed_receipt(&entry, data.path()).unwrap().is_some());
    assert!(!archive.exists());
}

#[test]
fn corrupt_archive_never_publishes_a_receipt() {
    let data = tempfile::tempdir().unwrap();
    let archive = data.path().join("silero.part");
    std::fs::write(&archive, b"bad").unwrap();
    let entry = raw_entry(b"vad");

    assert!(install_archive(&entry, &archive, data.path()).is_err());
    assert!(installed_receipt(&entry, data.path()).unwrap().is_none());
}

#[test]
fn an_existing_receipt_reuses_the_shared_vad_without_a_second_archive() {
    let data = tempfile::tempdir().unwrap();
    let archive = partial_path(data.path(), "silero-vad");
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"vad").unwrap();
    let entry = raw_entry(b"vad");
    let first = install_archive(&entry, &archive, data.path()).unwrap();

    let reused = install_archive(&entry, &archive, data.path()).unwrap();

    assert_eq!(reused, first);
    assert!(!archive.exists());
}

#[test]
fn a_corrupt_receipt_is_repaired_by_a_verified_reinstall() {
    let data = tempfile::tempdir().unwrap();
    let archive = partial_path(data.path(), "silero-vad");
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"vad").unwrap();
    let receipt = receipt_path(data.path(), "silero-vad");
    std::fs::create_dir_all(receipt.parent().unwrap()).unwrap();
    std::fs::write(&receipt, b"not-json").unwrap();
    let entry = raw_entry(b"vad");

    install_archive(&entry, &archive, data.path()).unwrap();

    assert!(installed_receipt(&entry, data.path()).unwrap().is_some());
}
