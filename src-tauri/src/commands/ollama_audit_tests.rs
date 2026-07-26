use crate::commands::ollama_bundle_utils::{
    archives_to_download, is_valid_semver, write_version_file,
};

#[test]
fn semver_rejects_path_traversal() {
    assert!(!is_valid_semver("1.0.0/../../evil"));
    assert!(!is_valid_semver("1.0.0%0d%0a"));
    assert!(!is_valid_semver("1.0.0\n"));
    assert!(!is_valid_semver(""));
    assert!(!is_valid_semver("abc"));
    assert!(!is_valid_semver("1.0"));
}

#[test]
fn semver_accepts_valid_versions() {
    assert!(is_valid_semver("0.23.1"));
    assert!(is_valid_semver("0.30.0-rc3"));
    assert!(is_valid_semver("1.0.0-beta.1"));
}

#[test]
fn semver_rejects_v_prefix() {
    assert!(!is_valid_semver("v1.0.0"));
}

#[test]
fn archives_returns_nonempty() {
    let a = archives_to_download();
    assert!(!a.is_empty());
    for name in &a {
        assert!(
            name.ends_with(".tgz") || name.ends_with(".tar.zst") || name.ends_with(".zip"),
            "unexpected archive format: {name}"
        );
    }
}

#[test]
fn version_file_roundtrip() {
    let dir = std::env::temp_dir().join("cl-go-test-version-roundtrip");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    write_version_file(&dir, "0.23.1");
    let content = std::fs::read_to_string(dir.join("VERSION")).unwrap();
    assert_eq!(content.trim(), "0.23.1");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn version_file_atomic_no_partial() {
    let dir = std::env::temp_dir().join("cl-go-test-version-atomic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    write_version_file(&dir, "1.0.0");
    assert!(
        !dir.join("VERSION.tmp").exists(),
        "tmp file should be cleaned up"
    );
    assert!(dir.join("VERSION").exists());

    let _ = std::fs::remove_dir_all(&dir);
}
