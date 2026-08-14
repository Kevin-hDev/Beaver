use super::durable_fs::{OllamaDurableFs, PlatformOllamaDurableFs};

#[test]
fn windows_native_primitive_flushes_parent_after_replace() {
    let root = tempfile::tempdir().expect("temporary directory");
    let fs = PlatformOllamaDurableFs::default();
    let tmp = root.path().join("journal.tmp");
    let final_path = root.path().join("journal.json");

    fs.write_new_atomic(&tmp, &final_path, b"first")
        .expect("new durable write");
    fs.replace_atomic(&tmp, &final_path, b"second")
        .expect("replacement durable write");
    assert_eq!(std::fs::read(final_path).unwrap(), b"second");
}
