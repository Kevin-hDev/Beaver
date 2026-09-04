use super::{
    read_verified_file,
    verified_file_read::{inspect, read_inspected, read_inspected_with_hook, FileReadError},
};
use tokio_util::sync::CancellationToken;

#[test]
fn inspection_keeps_size_and_open_identity_without_reading_bytes() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("resource.txt"), b"four").expect("resource");

    let inspected = inspect(root.path(), "resource.txt", 4).expect("inspection");

    assert_eq!(inspected.size, 4);
    assert_eq!(read_inspected(inspected, 4).expect("read").bytes, b"four");
}

#[test]
fn inspected_read_rejects_a_size_change_before_accepting_bytes() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("resource.txt");
    std::fs::write(&path, b"four").expect("resource");
    let inspected = inspect(root.path(), "resource.txt", 8).expect("inspection");
    std::fs::write(&path, b"larger file").expect("mutate");

    assert!(matches!(read_inspected(inspected, 8), Err(FileReadError::Access)));
}

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

    let inspected = inspect(root.path(), "resource.txt", 64).expect("inspection");
    std::fs::rename(&replacement, &file).expect("replace");
    let result = read_inspected_with_hook(inspected, 64, None, || {}, || {});

    assert!(matches!(result, Err(FileReadError::Access)));
}

#[test]
fn rejects_a_file_replaced_after_content_was_read() {
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("resource.txt");
    let replacement = root.path().join("replacement.txt");
    std::fs::write(&file, b"original").expect("original");
    std::fs::write(&replacement, b"replacement").expect("replacement");

    let result = read_inspected_with_hook(
        inspect(root.path(), "resource.txt", 64).expect("inspection"),
        64,
        None,
        || {
        std::fs::rename(&replacement, &file).expect("replace");
        },
        || {},
    );

    assert!(matches!(result, Err(FileReadError::Access)));
}

#[test]
fn rejects_a_file_replaced_during_content_read() {
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("resource.txt");
    let replacement = root.path().join("replacement.txt");
    std::fs::write(&file, vec![1_u8; 128 * 1024]).expect("original");
    std::fs::write(&replacement, vec![2_u8; 128 * 1024]).expect("replacement");
    let mut replaced = false;

    let result = read_inspected_with_hook(
        inspect(root.path(), "resource.txt", 128 * 1024).expect("inspection"),
        128 * 1024,
        None,
        || {},
        || {
            if !replaced {
                std::fs::rename(&replacement, &file).expect("replace during read");
                replaced = true;
            }
        },
    );

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

    let inspected = inspect(root.path(), "resource.txt", 64).expect("inspection");
    std::fs::remove_file(&file).expect("remove");
    symlink(outside.path(), &file).expect("symlink");
    let result = read_inspected_with_hook(inspected, 64, None, || {}, || {});

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

#[test]
fn cancellation_during_or_after_read_rejects_the_verified_file() {
    let root = tempfile::tempdir().expect("root");
    std::fs::write(root.path().join("resource.txt"), vec![1_u8; 128 * 1024])
        .expect("resource");

    let during = CancellationToken::new();
    let during_chunk = during.clone();
    let result = read_inspected_with_hook(
        inspect(root.path(), "resource.txt", 128 * 1024).expect("inspection"),
        128 * 1024,
        Some(&during),
        || {},
        move || during_chunk.cancel(),
    );
    assert!(matches!(result, Err(FileReadError::Cancelled)));

    let after = CancellationToken::new();
    let after_content = after.clone();
    let result = read_inspected_with_hook(
        inspect(root.path(), "resource.txt", 128 * 1024).expect("inspection"),
        128 * 1024,
        Some(&after),
        move || after_content.cancel(),
        || {},
    );
    assert!(matches!(result, Err(FileReadError::Cancelled)));
}
