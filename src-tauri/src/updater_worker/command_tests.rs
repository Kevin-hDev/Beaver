use std::ffi::OsString;
use std::time::Duration;

use super::{run_bounded_output, run_status, CommandSpec};

#[cfg(unix)]
#[test]
fn captures_only_bounded_direct_command_output() {
    let spec = CommandSpec::new("/usr/bin/printf", vec![OsString::from("beaver")]);
    assert_eq!(
        run_bounded_output(&spec, Duration::from_secs(1), 16).unwrap(),
        b"beaver"
    );
    assert!(run_bounded_output(&spec, Duration::from_secs(1), 3).is_err());
}

#[cfg(unix)]
#[test]
fn requires_a_successful_direct_program() {
    assert!(run_status(
        &CommandSpec::new("/usr/bin/true", vec![]),
        Duration::from_secs(1)
    )
    .is_ok());
    assert!(run_status(
        &CommandSpec::new("/usr/bin/false", vec![]),
        Duration::from_secs(1)
    )
    .is_err());
    assert!(run_status(
        &CommandSpec::new("relative", vec![]),
        Duration::from_secs(1)
    )
    .is_err());
}
