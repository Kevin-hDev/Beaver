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

pub fn configure_tokio(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    command.as_std_mut().creation_flags(CREATE_NO_WINDOW);
    #[cfg(not(windows))]
    let _ = command;
}

#[cfg(all(test, windows))]
#[path = "background_command_windows_tests.rs"]
mod windows_tests;
