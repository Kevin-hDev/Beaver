use super::cef_preflight::{run_with_retry, CefPreflightError, CEF_PREFLIGHT_RETRY_DELAY};
use super::cef_unavailable::CefUnavailableCategory;
use std::cell::Cell;
use std::io;

#[test]
fn transient_preflight_is_recreated_once_after_the_central_delay() {
    let attempts = Cell::new(0);
    let mut delays = Vec::new();

    let result = run_with_retry(
        || {
            let attempt = attempts.get() + 1;
            attempts.set(attempt);
            if attempt == 1 {
                Err(CefPreflightError::from_io(
                    CefUnavailableCategory::Object,
                    &io::Error::from(io::ErrorKind::WouldBlock),
                ))
            } else {
                Ok(attempt)
            }
        },
        |delay| delays.push(delay),
    );

    assert_eq!(result.expect("second clean attempt"), 2);
    assert_eq!(attempts.get(), 2);
    assert_eq!(delays, vec![CEF_PREFLIGHT_RETRY_DELAY]);
}

#[test]
fn deterministic_preflight_failure_is_never_retried() {
    let attempts = Cell::new(0);
    let mut slept = false;

    let result: Result<(), _> = run_with_retry(
        || {
            attempts.set(attempts.get() + 1);
            Err(CefPreflightError::deterministic(
                CefUnavailableCategory::Permission,
            ))
        },
        |_| slept = true,
    );

    assert!(result.is_err());
    assert_eq!(attempts.get(), 1);
    assert!(!slept);
}

#[test]
fn a_second_transient_failure_does_not_create_a_third_attempt() {
    let attempts = Cell::new(0);

    let result: Result<(), _> = run_with_retry(
        || {
            attempts.set(attempts.get() + 1);
            Err(CefPreflightError::from_io(
                CefUnavailableCategory::Reaper,
                &io::Error::from(io::ErrorKind::Interrupted),
            ))
        },
        |_| {},
    );

    assert!(result.is_err());
    assert_eq!(attempts.get(), 2);
}

#[test]
fn permission_errors_remain_deterministic() {
    let error = CefPreflightError::from_io(
        CefUnavailableCategory::Permission,
        &io::Error::from(io::ErrorKind::PermissionDenied),
    );

    assert!(!error.retryable());
}
