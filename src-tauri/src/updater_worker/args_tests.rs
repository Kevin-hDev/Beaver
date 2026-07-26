use std::ffi::OsString;
use std::fs;

use super::{parse_from, Platform};

fn valid_args(asset: &std::path::Path) -> Vec<OsString> {
    vec![
        "cl-go-dash-updater".into(),
        "--apply-update".into(),
        asset.as_os_str().to_owned(),
        "--parent-pid".into(),
        "42".into(),
    ]
}

#[test]
fn accepts_only_the_exact_command_and_positive_pid() {
    let temp = tempfile::tempdir().unwrap();
    let asset = temp
        .path()
        .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440000.dmg");
    fs::write(&asset, b"dmg").unwrap();

    let parsed = parse_from(valid_args(&asset), temp.path(), Platform::Macos).unwrap();
    assert_eq!(parsed.asset, fs::canonicalize(asset).unwrap());
    assert_eq!(parsed.parent_pid, 42);

    for invalid in ["0", "-1", "abc", "4294967296"] {
        let mut args = valid_args(&parsed.asset);
        args[4] = invalid.into();
        assert!(parse_from(args, temp.path(), Platform::Macos).is_err());
    }
}

#[test]
fn rejects_extra_arguments_wrong_extension_and_wrong_root() {
    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let asset = temp
        .path()
        .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440000.dmg");
    let wrong_extension = temp
        .path()
        .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440000.exe");
    let outside_asset = outside
        .path()
        .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440000.dmg");
    for path in [&asset, &wrong_extension, &outside_asset] {
        fs::write(path, b"asset").unwrap();
    }

    let mut extra = valid_args(&asset);
    extra.push("--anything".into());
    assert!(parse_from(extra, temp.path(), Platform::Macos).is_err());
    assert!(parse_from(valid_args(&wrong_extension), temp.path(), Platform::Macos).is_err());
    assert!(parse_from(valid_args(&outside_asset), temp.path(), Platform::Macos).is_err());
}

#[test]
fn rejects_parent_components_and_symlinks() {
    let temp = tempfile::tempdir().unwrap();
    let asset = temp
        .path()
        .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440000.dmg");
    fs::write(&asset, b"asset").unwrap();
    let with_parent = temp
        .path()
        .join("child")
        .join("..")
        .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440000.dmg");
    fs::create_dir(temp.path().join("child")).unwrap();
    assert!(parse_from(valid_args(&with_parent), temp.path(), Platform::Macos).is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link = temp
            .path()
            .join("cl-go-dash-update-550e8400-e29b-41d4-a716-446655440001.dmg");
        symlink(&asset, &link).unwrap();
        assert!(parse_from(valid_args(&link), temp.path(), Platform::Macos).is_err());
    }
}
