use std::ffi::OsString;
use std::fs;

use super::{ditto_spec, InstallTransaction, StagedBundle};
use crate::updater_worker::macos_bundle::{BundleKind, ValidatedBundle};

fn bundle(root: &std::path::Path, name: &str, marker: &[u8]) -> ValidatedBundle {
    let path = root.join(name);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("marker"), marker).unwrap();
    ValidatedBundle {
        executable: path.join("cl-go-dash"),
        root: path,
        kind: if name == "CL-GO.app" {
            BundleKind::Legacy
        } else {
            BundleKind::Beaver
        },
    }
}

fn stage(root: &std::path::Path, marker: &[u8]) -> StagedBundle {
    let path = root.join(".Beaver.app.update-test");
    fs::create_dir(&path).unwrap();
    fs::write(path.join("marker"), marker).unwrap();
    StagedBundle { path, armed: true }
}

#[test]
fn legacy_install_keeps_old_bundle_until_commit() {
    let root = tempfile::tempdir().unwrap();
    let current = bundle(root.path(), "CL-GO.app", b"old");
    let transaction = InstallTransaction::begin(&current, stage(root.path(), b"new")).unwrap();
    assert!(current.root.exists());
    assert_eq!(
        fs::read(transaction.installed_bundle().join("marker")).unwrap(),
        b"new"
    );
    transaction.commit().unwrap();
    assert!(!current.root.exists());
    assert!(root.path().join("Beaver.app").exists());
}

#[test]
fn beaver_upgrade_restores_old_bundle_on_rollback() {
    let root = tempfile::tempdir().unwrap();
    let current = bundle(root.path(), "Beaver.app", b"old");
    let mut transaction = InstallTransaction::begin(&current, stage(root.path(), b"new")).unwrap();
    assert_eq!(
        fs::read(transaction.installed_bundle().join("marker")).unwrap(),
        b"new"
    );
    transaction.rollback().unwrap();
    assert_eq!(fs::read(current.root.join("marker")).unwrap(), b"old");
}

#[test]
fn beaver_upgrade_deletes_backup_only_on_commit() {
    let root = tempfile::tempdir().unwrap();
    let current = bundle(root.path(), "Beaver.app", b"old");
    let transaction = InstallTransaction::begin(&current, stage(root.path(), b"new")).unwrap();
    assert_eq!(
        fs::read(transaction.installed_bundle().join("marker")).unwrap(),
        b"new"
    );
    transaction.commit().unwrap();
    assert_eq!(
        fs::read(root.path().join("Beaver.app/marker")).unwrap(),
        b"new"
    );
    assert_eq!(root.path().read_dir().unwrap().count(), 1);
}

#[test]
fn ditto_copy_uses_two_separate_paths() {
    let source = std::path::Path::new("/Volumes/Beaver/Beaver.app");
    let destination = std::path::Path::new("/Applications/.Beaver update.app");
    let spec = ditto_spec(source, destination);
    assert_eq!(spec.program, std::path::Path::new("/usr/bin/ditto"));
    assert_eq!(
        spec.args,
        vec![
            OsString::from("/Volumes/Beaver/Beaver.app"),
            OsString::from("/Applications/.Beaver update.app"),
        ]
    );
}
