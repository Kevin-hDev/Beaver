use super::read_response;
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::http::StatusCode;

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write(root: &Path, bytes: &[u8]) {
    std::fs::write(root.join("entry.mjs"), bytes).unwrap();
}

#[test]
fn accepts_unchanged_bytes_and_keeps_the_verified_buffer() {
    let root = tempfile::tempdir().unwrap();
    let original = b"export default true;";
    write(root.path(), original);

    let response = read_response(
        root.path(),
        "entry.mjs",
        &digest(original),
        Some(original.len()),
        "tauri://localhost",
    )
    .unwrap();
    write(root.path(), b"export default null;");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.body(), original);
}

#[test]
fn refuses_changed_content_even_when_its_size_matches() {
    let root = tempfile::tempdir().unwrap();
    let original = b"export default true;";
    let replacement = b"export default null;";
    assert_eq!(original.len(), replacement.len());
    write(root.path(), replacement);

    assert!(read_response(
        root.path(),
        "entry.mjs",
        &digest(original),
        Some(original.len()),
        "tauri://localhost",
    )
    .is_none());
}

#[test]
fn refuses_an_unexpected_size_and_an_oversized_file() {
    let root = tempfile::tempdir().unwrap();
    let original = b"export default true;";
    write(root.path(), original);
    assert!(read_response(
        root.path(),
        "entry.mjs",
        &digest(original),
        Some(original.len() + 1),
        "tauri://localhost",
    )
    .is_none());

    let oversized = vec![b'x'; super::super::ui_contract::MAX_ADVANCED_ARTIFACT_BYTES + 1];
    write(root.path(), &oversized);
    assert!(read_response(
        root.path(),
        "entry.mjs",
        &digest(&oversized),
        Some(oversized.len()),
        "tauri://localhost",
    )
    .is_none());
}

#[cfg(unix)]
#[test]
fn refuses_a_symbolic_link() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), b"export default true;").unwrap();
    std::os::unix::fs::symlink(outside.path(), root.path().join("entry.mjs")).unwrap();

    assert!(read_response(
        root.path(),
        "entry.mjs",
        &digest(b"export default true;"),
        None,
        "tauri://localhost",
    )
    .is_none());
}
