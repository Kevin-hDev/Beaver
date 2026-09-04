use super::{
    read_verified_file,
    verified_file_read::{read_after, read_after_content, FileReadError},
};

#[test]
fn reads_a_regular_file_at_the_exact_bound_and_rejects_an_oversized_file() {
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("resource.txt");
    std::fs::write(&file, b"four").expect("resource");

    let loaded = read_verified_file(root.path(), "resource.txt", 4).expect("exact bound");
    assert_eq!(loaded.bytes, b"four");
    assert!(matches!(
        read_verified_file(root.path(), "resource.txt", 3),
        Err(FileReadError::Limit)
    ));
}

#[test]
fn rejects_a_file_replaced_between_identity_checks() {
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("resource.txt");
    let replacement = root.path().join("replacement.txt");
    std::fs::write(&file, b"original").expect("original");
    std::fs::write(&replacement, b"replacement").expect("replacement");

    let result = read_after(root.path(), "resource.txt", 64, || {
        std::fs::rename(&replacement, &file).expect("replace");
    });

    assert!(matches!(result, Err(FileReadError::Access)));
}

#[test]
fn rejects_a_file_replaced_after_content_was_read() {
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("resource.txt");
    let replacement = root.path().join("replacement.txt");
    std::fs::write(&file, b"original").expect("original");
    std::fs::write(&replacement, b"replacement").expect("replacement");

    let result = read_after_content(root.path(), "resource.txt", 64, || {
        std::fs::rename(&replacement, &file).expect("replace");
    });

    assert!(matches!(result, Err(FileReadError::Access)));
}

#[cfg(unix)]
#[test]
fn rejects_a_late_symlink() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let outside = tempfile::NamedTempFile::new().expect("outside");
    let file = root.path().join("resource.txt");
    std::fs::write(&file, b"original").expect("original");

    let result = read_after(root.path(), "resource.txt", 64, || {
        std::fs::remove_file(&file).expect("remove");
        symlink(outside.path(), &file).expect("symlink");
    });

    assert!(matches!(result, Err(FileReadError::Access)));
}

#[cfg(unix)]
#[test]
fn rejects_an_existing_symlink_even_when_its_target_is_inside_the_root() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("target.txt"), b"content").expect("target");
    symlink("target.txt", root.path().join("resource.txt")).expect("symlink");

    assert!(matches!(
        read_verified_file(root.path(), "resource.txt", 64),
        Err(FileReadError::Access)
    ));
}

#[cfg(unix)]
#[test]
fn rejects_an_existing_symlink_to_outside_the_root() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let outside = tempfile::NamedTempFile::new().expect("outside");
    symlink(outside.path(), root.path().join("resource.txt")).expect("symlink");

    assert!(matches!(
        read_verified_file(root.path(), "resource.txt", 64),
        Err(FileReadError::Access)
    ));
}

#[test]
fn distinguishes_a_missing_file_from_an_access_failure() {
    let root = tempfile::tempdir().expect("root");

    assert!(matches!(
        read_verified_file(root.path(), "missing.txt", 64),
        Err(FileReadError::NotFound)
    ));
}
