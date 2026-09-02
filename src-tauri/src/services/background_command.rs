use std::ffi::OsStr;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

pub fn new<S: AsRef<OsStr>>(program: S) -> Command {
    let mut command = Command::new(program);
    configure(&mut command);
    command
}

pub fn new_tokio<S: AsRef<OsStr>>(program: S) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(program);
    configure_tokio(&mut command);
    command
}

pub fn configure(command: &mut Command) {
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    #[cfg(not(windows))]
    let _ = command;
}

/// Applique les drapeaux de base et ceux qu'un lanceur particulier exige, sans
/// que celui-ci redefinisse les drapeaux communs.
#[cfg(windows)]
pub fn configure_with_extra_flags(command: &mut Command, extra: u32) {
    command.creation_flags(CREATE_NO_WINDOW | extra);
}

pub fn configure_tokio(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    command.as_std_mut().creation_flags(CREATE_NO_WINDOW);
    #[cfg(not(windows))]
    let _ = command;
}

#[cfg(windows)]
pub fn configure_tokio_with_extra_flags(command: &mut tokio::process::Command, extra: u32) {
    command
        .as_std_mut()
        .creation_flags(CREATE_NO_WINDOW | extra);
}

#[cfg(all(test, windows))]
#[path = "background_command_windows_tests.rs"]
mod windows_tests;
