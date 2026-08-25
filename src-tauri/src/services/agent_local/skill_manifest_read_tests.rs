use std::fs::{self, OpenOptions};
use std::io::Write;

use tempfile::tempdir;

use super::skill_limits::MAX_SKILL_CONTENT_BYTES;
use super::skill_manifest_read::{read, read_after_metadata};

#[test]
fn manifest_accepts_exact_limit_and_rejects_limit_plus_one() {
    let dir = tempdir().unwrap();
    let exact = dir.path().join("exact.md");
    let oversized = dir.path().join("oversized.md");
    fs::write(&exact, vec![b'x'; MAX_SKILL_CONTENT_BYTES]).unwrap();
    fs::write(&oversized, vec![b'x'; MAX_SKILL_CONTENT_BYTES + 1]).unwrap();

    assert_eq!(read(&exact).unwrap().len(), MAX_SKILL_CONTENT_BYTES);
    assert!(read(&oversized).is_err());
}

#[cfg(unix)]
#[test]
fn manifest_symlink_is_never_followed() {
    use std::os::unix::fs::symlink;

    let dir = tempdir().unwrap();
    let target = dir.path().join("target.md");
    let link = dir.path().join("SKILL.md");
    fs::write(&target, b"secret").unwrap();
    symlink(&target, &link).unwrap();

    assert!(read(&link).is_err());
}

#[cfg(unix)]
#[test]
fn same_sized_path_substitution_cannot_change_the_opened_manifest() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("SKILL.md");
    fs::write(&path, b"safe").unwrap();

    let content = read_after_metadata(&path, || {
        fs::remove_file(&path).unwrap();
        fs::write(&path, b"evil").unwrap();
    })
    .unwrap();

    assert_eq!(content, "safe");
}

#[test]
fn growth_after_metadata_is_stopped_by_the_read_limit() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("SKILL.md");
    fs::write(&path, vec![b'x'; MAX_SKILL_CONTENT_BYTES]).unwrap();

    let result = read_after_metadata(&path, || {
        OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"y")
            .unwrap();
    });

    assert!(result.is_err());
}

#[test]
fn invalid_utf8_is_rejected_after_the_bounded_read() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("SKILL.md");
    fs::write(&path, [0xff]).unwrap();

    assert!(read(&path).is_err());
}
