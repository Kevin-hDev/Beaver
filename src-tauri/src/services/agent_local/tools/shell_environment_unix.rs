use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Instant;

const MAX_SHELL_CANDIDATES: usize = 4;

#[path = "shell_environment_capture.rs"]
mod capture;

pub(super) fn discover() -> Option<OsString> {
    let base_path = super::process_path()
        .map(|path| path.value)
        .unwrap_or_else(|| super::system_path().value);
    let started = Instant::now();
    for shell in shell_candidates() {
        let remaining = super::CAPTURE_TIMEOUT.checked_sub(started.elapsed())?;
        if let Some(path) = capture_sandboxed(&shell, &base_path, remaining) {
            return Some(path);
        }
    }
    None
}

fn shell_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::with_capacity(MAX_SHELL_CANDIDATES);
    if let Some(shell) = std::env::var_os("SHELL").map(PathBuf::from) {
        push_shell(&mut candidates, shell);
    }
    #[cfg(target_os = "macos")]
    push_shell(&mut candidates, PathBuf::from("/bin/zsh"));
    push_shell(&mut candidates, PathBuf::from("/bin/bash"));
    push_shell(&mut candidates, PathBuf::from("/bin/sh"));
    candidates
}

fn push_shell(candidates: &mut Vec<PathBuf>, shell: PathBuf) {
    let Some(shell) = valid_shell(&shell) else { return };
    if candidates.len() < MAX_SHELL_CANDIDATES && !candidates.contains(&shell) {
        candidates.push(shell);
    }
}

fn valid_shell(shell: &Path) -> Option<PathBuf> {
    let input = shell.to_str()?;
    if !shell.is_absolute()
        || input.chars().take(4_097).count() > 4_096
        || input.chars().any(char::is_control)
        || shell
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    let shell = dunce::canonicalize(shell).ok()?;
    let name = shell.file_name()?.to_string_lossy();
    (shell.is_file()
        && matches!(name.as_ref(), "zsh" | "bash" | "sh" | "dash" | "ksh" | "ksh93"))
    .then_some(shell)
}

fn capture_sandboxed(shell: &Path, base_path: &OsStr, timeout: std::time::Duration) -> Option<OsString> {
    capture_refined(base_path, timeout, |path, remaining| {
        capture_sandboxed_once(shell, path, remaining)
    })
}

fn capture_refined(
    base_path: &OsStr,
    timeout: std::time::Duration,
    mut capture: impl FnMut(&OsStr, std::time::Duration) -> Option<OsString>,
) -> Option<OsString> {
    let started = Instant::now();
    let first = super::normalize(capture(base_path, timeout)?, true)?.value;
    if first == base_path {
        return Some(first);
    }
    let Some(remaining) = timeout.checked_sub(started.elapsed()) else {
        return Some(first);
    };
    capture(&first, remaining)
        .and_then(|path| super::normalize(path, true).map(|path| path.value))
        .or(Some(first))
}

fn capture_sandboxed_once(
    shell: &Path,
    base_path: &OsStr,
    timeout: std::time::Duration,
) -> Option<OsString> {
    let marker = format!("__BEAVER_PATH_{}__", uuid::Uuid::new_v4().simple());
    let script = format!(
        "printf '%s' '{marker}'; printf '%s' \"$PATH\"; printf '%s' '{marker}'"
    );
    let arguments = ["-l", "-i", "-c", &script]
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
    let mut prepared = super::super::shell_sandbox::prepare_profile_capture(
        shell,
        &arguments,
        base_path,
    )
    .ok()?;
    capture::run(prepared.command_mut(), marker.as_bytes(), timeout)
}

#[cfg(test)]
fn capture_direct(shell: &Path, base_path: &OsStr) -> Option<OsString> {
    let marker = format!("__BEAVER_PATH_{}__", uuid::Uuid::new_v4().simple());
    let script = format!(
        "printf '%s' '{marker}'; printf '%s' \"$PATH\"; printf '%s' '{marker}'"
    );
    let mut command = std::process::Command::new(shell);
    command
        .args(["-l", "-i", "-c", &script])
        .env("PATH", base_path);
    capture::run(
        &mut command,
        marker.as_bytes(),
        super::CAPTURE_TIMEOUT,
    )
}

#[cfg(test)]
pub(super) fn capture_for_test(shell: &Path, base_path: &OsStr) -> Option<OsString> {
    capture_direct(shell, base_path)
}

#[cfg(test)]
pub(super) fn refine_for_test(
    base_path: &OsStr,
    capture: impl FnMut(&OsStr, std::time::Duration) -> Option<OsString>,
) -> Option<OsString> {
    capture_refined(base_path, super::CAPTURE_TIMEOUT, capture)
}
