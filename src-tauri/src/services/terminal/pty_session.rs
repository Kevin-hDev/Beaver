#[cfg(unix)]
#[path = "pty_session_unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "pty_session_windows.rs"]
mod platform;

pub use platform::PtySession;
