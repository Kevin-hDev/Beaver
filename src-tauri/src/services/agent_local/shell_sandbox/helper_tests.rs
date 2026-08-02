use super::parse;
use std::ffi::OsString;

#[test]
fn rejects_missing_separator_and_relative_executable() {
    let invalid_separator = vec![
        OsString::from("/tmp/sandbox"),
        OsString::from("bad"),
        OsString::from("/bin/sh"),
        OsString::from("-c"),
    ];
    let relative_executable = vec![
        OsString::from("/tmp/sandbox"),
        OsString::from("--"),
        OsString::from("sh"),
        OsString::from("-c"),
    ];

    assert!(parse(invalid_separator).is_err());
    assert!(parse(relative_executable).is_err());
}

#[test]
#[cfg(not(windows))]
fn rejects_parent_components_before_canonicalization() {
    let parent_component = vec![
        OsString::from("/tmp/sandbox"),
        OsString::from("--"),
        OsString::from("/bin/../bin/sh"),
        OsString::from("-c"),
    ];

    assert!(parse(parent_component).is_err());
}

#[test]
fn accepts_a_bounded_absolute_command() {
    let parsed = parse(vec![
        OsString::from("/tmp/sandbox"),
        OsString::from("--"),
        OsString::from(if cfg!(windows) { "C:\\Windows\\System32\\cmd.exe" } else { "/bin/sh" }),
        OsString::from("-c"),
    ]);

    if cfg!(windows) {
        assert!(parsed.is_err());
    } else {
        assert!(parsed.is_ok());
    }
}

#[test]
#[cfg(not(windows))]
fn accepts_the_explicit_profile_capture_mode() {
    let parsed = parse(vec![
        OsString::from(super::super::launch::profile_capture_arg()),
        OsString::from("/tmp/sandbox"),
        OsString::from("--"),
        OsString::from("/bin/sh"),
        OsString::from("-c"),
    ])
    .expect("profile capture");

    assert_eq!(parsed.0, super::super::scope::Mode::ProfileCapture);
}
