use std::fs;

use tempfile::tempdir;

use super::attachment_access::{register_paths, verify_access_grant};

const TEST_KEY: [u8; 32] = [7; 32];

#[test]
fn selected_file_gets_a_verifiable_grant() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("notes.txt");
    fs::write(&path, b"safe").unwrap();
    let raw = path.to_string_lossy().to_string();

    let registered = register_paths(&[raw], &TEST_KEY, |_| true).unwrap();

    assert_eq!(registered.len(), 1);
    assert_eq!(registered[0].size, 4);
    assert!(
        verify_access_grant(&registered[0].path, &registered[0].access_grant, &TEST_KEY,).is_ok()
    );
}

#[test]
fn unselected_file_is_refused() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("private.txt");
    fs::write(&path, b"secret").unwrap();

    let result = register_paths(&[path.to_string_lossy().to_string()], &TEST_KEY, |_| false);

    assert!(result.is_err());
}

#[test]
fn traversal_is_refused_before_canonicalization() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    let path = dir.path().join("target.txt");
    fs::write(&path, b"safe").unwrap();
    let raw = nested.join("..").join("target.txt");

    let result = register_paths(&[raw.to_string_lossy().to_string()], &TEST_KEY, |_| true);

    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn symbolic_link_is_refused_before_canonicalization() {
    use std::os::unix::fs::symlink;

    let dir = tempdir().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("selected.txt");
    fs::write(&target, b"safe").unwrap();
    symlink(&target, &link).unwrap();

    let result = register_paths(&[link.to_string_lossy().to_string()], &TEST_KEY, |_| true);

    assert!(result.is_err());
}

#[test]
fn forged_grant_is_refused() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("notes.txt");
    fs::write(&path, b"safe").unwrap();
    let raw = path.to_string_lossy().to_string();
    let registered = register_paths(&[raw], &TEST_KEY, |_| true).unwrap();

    let result = verify_access_grant(
        &registered[0].path,
        "v1.0000000000000000000000000000000000000000000000000000000000000000",
        &TEST_KEY,
    );

    assert!(result.is_err());
}

#[test]
fn more_than_fifteen_paths_are_refused() {
    let paths = (0..16)
        .map(|index| format!("/tmp/{index}"))
        .collect::<Vec<_>>();

    assert!(register_paths(&paths, &TEST_KEY, |_| true).is_err());
}

#[test]
fn file_larger_than_twenty_mib_is_refused() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("large.bin");
    let file = fs::File::create(&path).unwrap();
    file.set_len(20 * 1024 * 1024 + 1).unwrap();

    let result = register_paths(&[path.to_string_lossy().to_string()], &TEST_KEY, |_| true);

    assert!(result.is_err());
}

#[test]
fn legacy_attachment_without_grant_remains_readable_as_metadata() {
    let file: crate::services::agent_local::types_message::FileAttachment =
        serde_json::from_value(serde_json::json!({
            "name": "legacy.png",
            "path": "/tmp/legacy.png",
            "mime_type": "image/png",
            "size": 4,
            "thumbnail": "data:image/png;base64,bGVnYWN5"
        }))
        .unwrap();

    assert!(file.access_grant.is_none());
    assert!(file.thumbnail.is_some());
}

#[test]
fn capabilities_do_not_expose_global_or_docs_file_access() {
    let main: serde_json::Value =
        serde_json::from_str(include_str!("../../capabilities/default.json")).unwrap();
    let docs: serde_json::Value =
        serde_json::from_str(include_str!("../../capabilities/forecast-docs.json")).unwrap();
    let workbench: serde_json::Value =
        serde_json::from_str(include_str!("../../capabilities/forecast-workbench.json")).unwrap();

    assert_eq!(main["windows"], serde_json::json!(["main"]));
    assert!(!main.to_string().contains("\"path\":\"**\""));
    assert!(!main.to_string().contains("fs:allow-stat"));
    assert!(!docs.to_string().contains("fs:"));
    assert!(!workbench.to_string().contains("fs:"));
}
