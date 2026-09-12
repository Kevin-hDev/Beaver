use super::{parse_watchdog, wait_ready_with_timeout, READY};
use super::super::macos_parent_watchdog::{watchdog_action, WatchdogAction};
use std::ffi::OsString;
use std::process::{Command, Stdio};

#[test]
fn watchdog_arguments_are_bounded_numbers() {
    let valid = ["12", "12", "34", "56", "78", "90"]
        .into_iter()
        .map(OsString::from)
        .collect();
    assert!(parse_watchdog(valid).is_ok());

    let invalid = ["12", "12", "34", "not-a-number", "78", "90"]
        .into_iter()
        .map(OsString::from)
        .collect();
    assert!(parse_watchdog(invalid).is_err());
}

#[test]
fn readiness_rejects_timeout_eof_and_invalid_byte() {
    assert_ready("sleep 1", 20, false);
    assert_ready("true", 100, false);
    assert_ready("printf X", 100, false);
    assert_ready(&format!("printf '\\{:03o}'", READY), 100, true);
}

#[test]
fn transient_process_inspection_keeps_the_watchdog_alive() {
    assert_eq!(watchdog_action(Err(()), Ok(true)), WatchdogAction::Wait);
    assert_eq!(watchdog_action(Ok(false), Err(())), WatchdogAction::Wait);
}

fn assert_ready(script: &str, timeout_ms: i32, expected: bool) {
    let mut child = Command::new("/bin/sh")
        .args(["-c", script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("readiness fixture");
    let result = wait_ready_with_timeout(child.stdout.take().expect("fixture stdout"), timeout_ms);
    if !expected {
        let _ = child.kill();
    }
    let _ = child.wait();
    assert_eq!(result.is_ok(), expected);
}
