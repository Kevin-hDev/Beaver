use std::path::{Path, PathBuf};

fn fixture(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("index.mjs"), "export default {};").unwrap();
    std::fs::write(
        root.join("beaver-extension.json"),
        r#"{"id":"test.long-path","name":"Long path","version":"1.0.0","beaverApi":"1","runtime":"node","main":"index.mjs","access":"full"}"#,
    )
    .unwrap();
}

fn boundary_root(parent: &Path, length: usize) -> PathBuf {
    let parent = dunce::canonicalize(parent).unwrap();
    let remaining = length
        .checked_sub(parent.to_str().unwrap().len() + 1)
        .unwrap();
    assert!(remaining > 0 && remaining < 256);
    parent.join("a".repeat(remaining))
}

#[test]
fn manifest_and_entry_crossing_windows_path_boundary_remain_inside_root() {
    let temporary = tempfile::tempdir().unwrap();
    // In the failing Windows layout the root is short, its manifest and entry are long.
    let root = boundary_root(temporary.path(), 253);
    fixture(&root);
    #[cfg(windows)]
    {
        let short_root = dunce::canonicalize(&root).unwrap();
        let long_manifest = dunce::canonicalize(root.join("beaver-extension.json")).unwrap();
        assert!(
            !long_manifest.starts_with(&short_root),
            "exercise the original prefix mismatch"
        );
    }
    let record = super::load_local(root.to_str().unwrap()).unwrap().record;
    assert_eq!(record.manifest.main.as_deref(), Some("index.mjs"));
    assert!(super::super::registry_managed::ensure_current(&record).is_ok());
}

#[test]
fn canonical_containment_still_refuses_an_outside_entry() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("inside");
    fixture(&root);
    let outside = temporary.path().join("outside.mjs");
    std::fs::write(&outside, "export default {};").unwrap();
    assert!(super::resolve_inside(&root, &outside).is_err());
    assert!(super::resolve_inside(&root, &root.join("../outside.mjs")).is_err());
    #[cfg(unix)]
    {
        let link = root.join("escape.mjs");
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        assert!(super::resolve_inside(&root, &link).is_err());
    }
}
