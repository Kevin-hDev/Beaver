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

#[test]
fn root_limit_warning_reports_only_a_new_bounded_diagnostic() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("root-limit.json");
    let before = RootLimitMarker(SystemTime::now());

    write_at(&path, Status::RootPathRead).expect("diagnostic");

    let bytes = std::fs::read(&path).expect("read");
    assert!(bytes.len() < MAX_DIAGNOSTIC_BYTES as usize);
    let warning = warning_since_at(&path, &before).expect("warning");
    assert!(warning.contains("PATH"));
    assert!(warning.contains("lecture seule"));
    assert!(!warning.contains(&temp.path().to_string_lossy().to_string()));

    let unchanged = RootLimitMarker(SystemTime::now());
    assert!(warning_since_at(&path, &unchanged).is_none());
    clear_at(&path).expect("clear");
    assert!(warning_since_at(&path, &before).is_none());
}
