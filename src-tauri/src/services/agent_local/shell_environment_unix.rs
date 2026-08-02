use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);
const READER_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(250);
const MAX_SHELL_CANDIDATES: usize = 4;

pub(super) fn discover() -> Option<OsString> {
    shell_candidates()
        .into_iter()
        .find_map(|shell| capture(&shell, None))
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

fn capture(shell: &Path, base_path: Option<&OsStr>) -> Option<OsString> {
    let marker = format!("__BEAVER_PATH_{}__", uuid::Uuid::new_v4().simple());
    let script = format!(
        "printf '%s' '{marker}'; printf '%s' \"$PATH\"; printf '%s' '{marker}'"
    );
    let mut command = Command::new(shell);
    command
        .args(["-l", "-i", "-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0);
    if let Some(base_path) = base_path {
        command.env("PATH", base_path);
    }
    let mut child = command.spawn().ok()?;
    let pid = child.id();
    let Some(stdout) = child.stdout.take() else {
        terminate_group(pid);
        let _ = child.kill();
        let _ = child.wait();
        return None;
    };
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut bytes = Vec::with_capacity(8_192);
        let result = stdout
            .take((super::MAX_CAPTURE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let status = wait(&mut child);
    terminate_descendants(pid);
    let bytes = receiver.recv_timeout(READER_TIMEOUT).ok()?.ok()?;
    if !status.is_some_and(|status| status.success()) {
        return None;
    }
    if bytes.len() > super::MAX_CAPTURE_BYTES {
        return None;
    }
    extract(&bytes, marker.as_bytes())
}

fn wait(child: &mut std::process::Child) -> Option<std::process::ExitStatus> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => {
                terminate_group(child.id());
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
        if started.elapsed() >= super::CAPTURE_TIMEOUT {
            terminate_group(child.id());
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn terminate_descendants(pid: u32) {
    let Ok(pid) = i32::try_from(pid) else { return };
    // SAFETY: le shell de découverte a été placé dans son propre groupe.
    let exists = unsafe { libc::kill(-pid, 0) == 0 };
    if exists {
        terminate_group(pid as u32);
    }
}

fn terminate_group(pid: u32) {
    let Ok(pid) = i32::try_from(pid) else { return };
    // SAFETY: seul le groupe dédié au processus enfant validé est ciblé.
    unsafe {
        libc::kill(-pid, libc::SIGTERM);
        libc::kill(-pid, libc::SIGKILL);
    }
}

fn extract(output: &[u8], marker: &[u8]) -> Option<OsString> {
    use std::os::unix::ffi::OsStringExt;

    let start = find(output, marker)? + marker.len();
    let end = find(&output[start..], marker)? + start;
    (end > start).then(|| OsString::from_vec(output[start..end].to_vec()))
}

fn find(input: &[u8], needle: &[u8]) -> Option<usize> {
    input.windows(needle.len()).position(|window| window == needle)
}

#[cfg(test)]
pub(super) fn capture_for_test(shell: &Path, base_path: &OsStr) -> Option<OsString> {
    capture(shell, Some(base_path))
}
