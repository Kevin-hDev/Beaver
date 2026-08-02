use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

pub(crate) const MAX_PATH_INPUTS: usize = 256;
const MAX_CAPTURE_BYTES: usize = 128 * 1024;
const CAPTURE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

#[derive(Clone)]
struct ShellPath {
    value: OsString,
    entries: Vec<PathBuf>,
    overflow: bool,
    discovered: bool,
}

static SHELL_PATH: OnceLock<ShellPath> = OnceLock::new();

pub(crate) fn initialize() -> bool {
    if let Some(path) = SHELL_PATH.get() {
        return path.discovered;
    }
    let resolved = discover_login_path()
        .or_else(process_path)
        .unwrap_or_else(system_path);
    let _ = SHELL_PATH.set(resolved);
    SHELL_PATH.get().is_some_and(|path| path.discovered)
}

pub(crate) fn value() -> OsString {
    current().value
}

pub(crate) fn entries() -> (Vec<PathBuf>, bool) {
    let path = current();
    (path.entries, path.overflow)
}

fn current() -> ShellPath {
    SHELL_PATH
        .get()
        .cloned()
        .or_else(process_path)
        .unwrap_or_else(system_path)
}

fn process_path() -> Option<ShellPath> {
    normalize(std::env::var_os("PATH")?, false)
}

fn system_path() -> ShellPath {
    #[cfg(windows)]
    let root = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    #[cfg(windows)]
    let value = std::env::join_paths([root.join("System32"), root.clone()])
        .unwrap_or_else(|_| OsString::from(r"C:\Windows\System32;C:\Windows"));
    #[cfg(not(windows))]
    let value = OsString::from("/usr/bin:/bin:/usr/sbin:/sbin");
    normalize(value, false).expect("system PATH must be valid")
}

fn normalize(value: OsString, discovered: bool) -> Option<ShellPath> {
    let mut entries = Vec::with_capacity(MAX_PATH_INPUTS);
    let mut overflow = false;
    for entry in std::env::split_paths(&value) {
        if !valid_entry(&entry) || entries.contains(&entry) {
            continue;
        }
        if entries.len() >= MAX_PATH_INPUTS {
            overflow = true;
            continue;
        }
        entries.push(entry);
    }
    if entries.is_empty() {
        return None;
    }
    let value = std::env::join_paths(&entries).ok()?;
    Some(ShellPath {
        value,
        entries,
        overflow,
        discovered,
    })
}

fn valid_entry(path: &Path) -> bool {
    let Some(text) = path.to_str() else { return false };
    path.is_absolute()
        && text.chars().take(4_097).count() <= 4_096
        && !text.chars().any(char::is_control)
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

#[cfg(unix)]
#[path = "shell_environment_unix.rs"]
mod unix;

#[cfg(unix)]
fn discover_login_path() -> Option<ShellPath> {
    unix::discover().and_then(|value| normalize(value, true))
}

#[cfg(not(unix))]
fn discover_login_path() -> Option<ShellPath> {
    None
}

#[cfg(test)]
#[path = "shell_environment_tests.rs"]
mod tests;
