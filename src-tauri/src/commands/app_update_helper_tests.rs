use std::fs;

use super::copy_helper;

#[test]
fn copies_a_bounded_regular_helper_with_private_permissions() {
    let resources = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let source = resources.path().join("cl-go-dash-updater");
    fs::write(&source, b"helper").unwrap();

    let copied = copy_helper(&source, resources.path(), destination.path()).unwrap();
    assert_eq!(fs::read(copied.path()).unwrap(), b"helper");
    assert!(copied
        .path()
        .starts_with(fs::canonicalize(destination.path()).unwrap()));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(copied.path()).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}

#[test]
fn rejects_empty_outside_and_symlinked_helpers() {
    let resources = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let empty = resources.path().join("empty");
    let outside_file = outside.path().join("helper");
    fs::write(&empty, b"").unwrap();
    fs::write(&outside_file, b"helper").unwrap();

    assert!(copy_helper(&empty, resources.path(), destination.path()).is_err());
    assert!(copy_helper(&outside_file, resources.path(), destination.path()).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link = resources.path().join("link");
        symlink(&outside_file, &link).unwrap();
        assert!(copy_helper(&link, resources.path(), destination.path()).is_err());
    }
}
