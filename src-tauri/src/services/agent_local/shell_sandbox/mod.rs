mod helper;
mod launch;
mod scope;
mod tool_cache_env;
mod tool_cache_platform;
mod tool_cache_prepare;
mod tool_cache_roots;
mod tool_roots;
mod tool_roots_entries;
mod tool_roots_path;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub use launch::prepare_command;
#[cfg(unix)]
pub(crate) use launch::prepare_profile_capture;
pub use launch::{cleanup_stale, cleanup_temp};

pub fn run_helper_if_requested() -> Option<i32> {
    helper::run_if_requested()
}
