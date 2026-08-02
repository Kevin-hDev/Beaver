mod helper;
mod launch;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub use launch::prepare_command;
pub use launch::{cleanup_stale, cleanup_temp};

pub fn run_helper_if_requested() -> Option<i32> {
    helper::run_if_requested()
}
