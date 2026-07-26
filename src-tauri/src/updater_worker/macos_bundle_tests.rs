use std::collections::BTreeMap;
use std::fs;

use super::{validate_beaver_source, validate_current, BundleKind};

fn create_bundle(root: &std::path::Path, name: &str, identifier: &str) -> std::path::PathBuf {
    let bundle = root.join(name);
    let executable_dir = bundle.join("Contents/MacOS");
    fs::create_dir_all(&executable_dir).unwrap();
    fs::write(executable_dir.join("cl-go-dash"), b"binary").unwrap();
    let mut info = BTreeMap::new();
    info.insert(
        "CFBundleIdentifier".to_string(),
        plist::Value::String(identifier.to_string()),
    );
    info.insert(
        "CFBundleExecutable".to_string(),
        plist::Value::String("cl-go-dash".to_string()),
    );
    plist::to_file_xml(bundle.join("Contents/Info.plist"), &info).unwrap();
    bundle
}

#[test]
fn accepts_only_expected_names_identifier_and_executable() {
    let root = tempfile::tempdir().unwrap();
    let beaver = create_bundle(root.path(), "Beaver.app", "com.clgo.dash");
    let legacy = create_bundle(root.path(), "CL-GO.app", "com.clgo.dash");
    assert_eq!(
        validate_beaver_source(&beaver).unwrap().kind,
        BundleKind::Beaver
    );
    assert_eq!(validate_current(&legacy).unwrap().kind, BundleKind::Legacy);

    let wrong = create_bundle(root.path(), "Wrong.app", "com.example.wrong");
    assert!(validate_beaver_source(&wrong).is_err());
    assert!(validate_current(&wrong).is_err());
}

#[test]
fn rejects_symlinked_bundle_and_missing_executable() {
    let root = tempfile::tempdir().unwrap();
    let beaver = create_bundle(root.path(), "Beaver.app", "com.clgo.dash");
    fs::remove_file(beaver.join("Contents/MacOS/cl-go-dash")).unwrap();
    assert!(validate_beaver_source(&beaver).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let real = create_bundle(root.path(), "Real.app", "com.clgo.dash");
        let link = root.path().join("Linked.app");
        symlink(real, &link).unwrap();
        assert!(validate_current(&link).is_err());
    }
}
