use super::cef_unavailable::CefUnavailableCategory;
use std::io;
use std::time::Duration;

pub(super) const CEF_PREFLIGHT_RETRY_DELAY: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CefPreflightError {
    category: CefUnavailableCategory,
    retryable: bool,
}

impl CefPreflightError {
    pub(super) const fn deterministic(category: CefUnavailableCategory) -> Self {
        Self {
            category,
            retryable: false,
        }
    }

    pub(super) fn from_io(category: CefUnavailableCategory, error: &io::Error) -> Self {
        Self {
            category,
            retryable: retryable_io_error(error),
        }
    }

    pub(super) const fn category(self) -> CefUnavailableCategory {
        self.category
    }

    pub(super) const fn retryable(self) -> bool {
        self.retryable
    }
}

pub(super) fn run_with_retry<T>(
    mut attempt: impl FnMut() -> Result<T, CefPreflightError>,
    mut sleep: impl FnMut(Duration),
) -> Result<T, CefPreflightError> {
    match attempt() {
        Err(error) if error.retryable() => {
            sleep(CEF_PREFLIGHT_RETRY_DELAY);
            attempt()
        }
        result => result,
    }
}

fn retryable_io_error(error: &io::Error) -> bool {
    if matches!(
        error.kind(),
        io::ErrorKind::Interrupted
            | io::ErrorKind::WouldBlock
            | io::ErrorKind::TimedOut
            | io::ErrorKind::OutOfMemory
    ) {
        return true;
    }
    error.raw_os_error().is_some_and(retryable_os_code)
}

#[cfg(target_os = "windows")]
fn retryable_os_code(code: i32) -> bool {
    const TRANSIENT_WINDOWS_ERRORS: [i32; 10] = [4, 8, 14, 21, 32, 33, 164, 170, 1237, 1450];
    TRANSIENT_WINDOWS_ERRORS.contains(&code)
}

#[cfg(target_os = "macos")]
fn retryable_os_code(code: i32) -> bool {
    matches!(
        code,
        libc::EINTR | libc::EAGAIN | libc::ENOMEM | libc::EBUSY | libc::ETIMEDOUT
    )
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn retryable_os_code(_code: i32) -> bool {
    false
}
