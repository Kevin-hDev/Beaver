use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use tokio::process::Command;

pub(super) fn gui_path() -> OsString {
    crate::services::agent_local::shell_environment::value()
}

pub(super) fn command_for(program: &Path, path: &OsStr) -> Command {
    let mut command = Command::new(program);
    command.env("PATH", path);
    command
}

pub(super) fn locate(name: &Path, path: &OsStr) -> Option<PathBuf> {
    let suffixes = lookup_suffixes();
    std::env::split_paths(path)
        .find_map(|directory| locate_with_suffixes(name, &directory, &suffixes))
}

pub(super) fn lookup_suffixes() -> [&'static OsStr; 1] {
    #[cfg(windows)]
    let suffixes = [OsStr::new(".exe")];
    #[cfg(not(windows))]
    let suffixes = [OsStr::new("")];
    suffixes
}

pub(super) fn locate_with_suffixes(
    name: &Path,
    directory: &Path,
    suffixes: &[&OsStr],
) -> Option<PathBuf> {
    suffixes
        .iter()
        .map(|suffix| {
            let mut file_name = name.as_os_str().to_os_string();
            file_name.push(suffix);
            directory.join(file_name)
        })
        .find(|candidate| executable_file(candidate))
}

fn executable_file(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    true
}
