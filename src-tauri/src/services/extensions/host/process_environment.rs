use std::ffi::{OsStr, OsString};
use std::path::Path;

const MAX_PATH_CHARS: usize = 16_384;
#[cfg(windows)]
const MAX_SYSTEM_ROOT_CHARS: usize = 1_024;

pub(super) fn inherited_path() -> Result<OsString, ()> {
    std::env::var_os("PATH")
        .filter(|value| valid_environment_value(value, MAX_PATH_CHARS))
        .ok_or(())
}

pub(super) fn configure_host(
    command: &mut tokio::process::Command,
    path: OsString,
    temporary_directory: &Path,
) -> Result<(), ()> {
    configure_minimal(command.as_std_mut(), path, temporary_directory)
}

pub(super) fn configure_installer(
    command: &mut std::process::Command,
    path: OsString,
    temporary_directory: &Path,
) -> Result<(), ()> {
    configure_minimal(command, path, temporary_directory)?;
    command.env("HOME", temporary_directory);
    #[cfg(windows)]
    command.env("USERPROFILE", temporary_directory);
    Ok(())
}

fn configure_minimal(
    command: &mut std::process::Command,
    path: OsString,
    temporary_directory: &Path,
) -> Result<(), ()> {
    if !valid_environment_value(&path, MAX_PATH_CHARS) {
        return Err(());
    }
    command
        .env_clear()
        .env("PATH", path)
        .env("TMPDIR", temporary_directory)
        .env("TMP", temporary_directory)
        .env("TEMP", temporary_directory);
    #[cfg(windows)]
    {
        let system_root = std::env::var_os("SystemRoot").ok_or(())?;
        if !valid_environment_value(&system_root, MAX_SYSTEM_ROOT_CHARS) {
            return Err(());
        }
        command.env("SystemRoot", system_root);
    }
    Ok(())
}

pub(super) fn valid_environment_value(value: &OsStr, maximum: usize) -> bool {
    let text = value.to_string_lossy();
    text.chars().count() <= maximum && !text.chars().any(char::is_control)
}

#[cfg(test)]
#[path = "process_environment_tests.rs"]
mod tests;
