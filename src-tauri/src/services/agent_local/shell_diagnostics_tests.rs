use super::*;

#[test]
fn diagnostic_is_bounded_and_contains_no_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("status.json");

    write_at(&path, Status::PathFallback).expect("diagnostic");

    let bytes = std::fs::read(path).expect("read");
    assert!(bytes.len() < MAX_DIAGNOSTIC_BYTES as usize);
    let text = String::from_utf8(bytes).expect("utf8");
    assert!(text.contains("path_fallback"));
    assert!(!text.contains("/Users/"));
}
