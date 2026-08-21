use std::path::PathBuf;

use super::runtime_error::RuntimeError;
use super::runtime_manifest::MANIFEST_NAME;
use super::wheels::{for_source, sync_at, STAMP_NAME};

#[test]
fn wheelhouse_exposes_the_validated_manifest() {
    let (_parent, source, _wheels) = valid_wheelhouse();
    let wheelhouse = for_source(&source).unwrap().expect("wheelhouse");

    assert_eq!(wheelhouse.manifest.python_major, 3);
    assert_eq!(wheelhouse.manifest.python_minor, 14);
}

#[test]
fn wheelhouse_rejects_a_stamp_not_bound_to_the_manifest() {
    let (_parent, source, wheels) = valid_wheelhouse();
    std::fs::write(wheels.join(STAMP_NAME), "b".repeat(64)).unwrap();

    assert!(matches!(
        for_source(&source),
        Err(RuntimeError::ManifestInvalid)
    ));
}

#[test]
fn wheelhouse_rejects_a_hard_linked_stamp() {
    let (_parent, source, wheels) = valid_wheelhouse();
    let stamp = wheels.join(STAMP_NAME);
    let target = wheels.parent().unwrap().join("stamp-target");
    std::fs::rename(&stamp, &target).unwrap();
    std::fs::hard_link(&target, &stamp).unwrap();

    assert!(matches!(
        for_source(&source),
        Err(RuntimeError::ManifestInvalid)
    ));
    assert_eq!(std::fs::read(target).unwrap(), "a".repeat(64).as_bytes());
}

#[test]
fn wheelhouse_rejects_an_oversized_manifest_and_a_foreign_file() {
    let (_parent, source, wheels) = valid_wheelhouse();
    std::fs::write(wheels.join(MANIFEST_NAME), vec![b'x'; 513]).unwrap();
    assert!(matches!(
        for_source(&source),
        Err(RuntimeError::ManifestInvalid)
    ));

    std::fs::write(wheels.join(MANIFEST_NAME), manifest()).unwrap();
    std::fs::write(wheels.join("foreign.txt"), b"unexpected").unwrap();
    assert!(matches!(
        for_source(&source),
        Err(RuntimeError::WheelhouseUnavailable)
    ));
}

#[test]
fn wheelhouse_rejects_more_than_the_bounded_entry_count() {
    let (_parent, source, wheels) = valid_wheelhouse();
    for index in 0..510 {
        std::fs::write(wheels.join(format!("extra-{index}.whl")), b"wheel").unwrap();
    }

    assert!(matches!(
        for_source(&source),
        Err(RuntimeError::WheelhouseUnavailable)
    ));
}

#[test]
fn interrupted_wheelhouse_publication_recovers_one_complete_generation() {
    let root = tempfile::tempdir().unwrap();
    let current = root.path().join("wheels");
    let staged = root.path().join("wheels.next");
    let previous = root.path().join("wheels.previous");
    std::fs::create_dir(&previous).unwrap();
    std::fs::write(previous.join("old.whl"), b"old").unwrap();
    std::fs::create_dir(&staged).unwrap();
    std::fs::write(staged.join("partial.whl"), b"partial").unwrap();

    super::generational_publication::recover(
        super::generational_publication::Paths {
            current: &current,
            staged: &staged,
            previous: &previous,
        },
        super::generational_publication::RecoveryPolicy::CommitImmediately,
        RuntimeError::WheelhouseUnavailable,
    )
    .unwrap();

    assert_eq!(std::fs::read(current.join("old.whl")).unwrap(), b"old");
    assert!(!staged.exists());
    assert!(!previous.exists());
}

#[test]
fn sync_recovers_orphan_staging_before_copying_the_archive_generation() {
    let (root, _source, _wheels) = valid_wheelhouse();
    let archive = root.path().join("archive.tar");
    let install = root.path().join("install");
    std::fs::create_dir(&install).unwrap();
    let current = install.join("wheels");
    let staged = install.join("wheels.next");
    let previous = install.join("wheels.previous");
    std::fs::create_dir(&staged).unwrap();
    std::fs::write(staged.join("partial.whl"), b"partial").unwrap();

    sync_at(&archive, &current, &staged, &previous).unwrap();

    assert!(current.join("a.whl").is_file());
    assert!(!current.join("partial.whl").exists());
    assert!(!staged.exists());
    assert!(!previous.exists());
}

#[cfg(unix)]
#[test]
fn wheelhouse_rejects_a_symlinked_manifest() {
    let (_parent, source, wheels) = valid_wheelhouse();
    let target = wheels.join("manifest-target");
    std::fs::write(&target, manifest()).unwrap();
    std::fs::remove_file(wheels.join(MANIFEST_NAME)).unwrap();
    std::os::unix::fs::symlink(target, wheels.join(MANIFEST_NAME)).unwrap();

    assert!(matches!(
        for_source(&source),
        Err(RuntimeError::ManifestInvalid)
    ));
}

fn valid_wheelhouse() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let parent = tempfile::tempdir().unwrap();
    let source = parent.path().join("source");
    std::fs::create_dir(&source).unwrap();
    let wheels = parent.path().join("wheels");
    std::fs::create_dir(&wheels).unwrap();
    std::fs::write(wheels.join("a.whl"), b"wheel").unwrap();
    std::fs::write(wheels.join(STAMP_NAME), "a".repeat(64)).unwrap();
    std::fs::write(wheels.join(MANIFEST_NAME), manifest()).unwrap();

    (parent, source, wheels)
}

fn manifest() -> &'static [u8] {
    br#"{"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#
}
