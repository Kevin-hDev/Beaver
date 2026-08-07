use std::ffi::OsString;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::Instant;

const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(25);
const READER_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(250);

pub(super) fn run(
    command: &mut Command,
    marker: &[u8],
    timeout: std::time::Duration,
) -> Option<OsString> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0);
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
            .take((super::super::MAX_CAPTURE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let status = wait(&mut child, timeout);
    terminate_descendants(pid);
    let bytes = receiver.recv_timeout(READER_TIMEOUT).ok()?.ok()?;
    if !status.is_some_and(|status| status.success())
        || bytes.len() > super::super::MAX_CAPTURE_BYTES
    {
        return None;
    }
    extract(&bytes, marker)
}

fn wait(
    child: &mut std::process::Child,
    timeout: std::time::Duration,
) -> Option<std::process::ExitStatus> {
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
        if started.elapsed() >= timeout {
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
    if unsafe { libc::kill(-pid, 0) == 0 } {
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    const HELPER_PHASE: &str = "BEAVER_CAPTURE_HELPER_PHASE";
    const HELPER_MARKER: &str = "BEAVER_CAPTURE_HELPER_MARKER";

    #[test]
    fn capture_timeout_remains_bounded_when_writer_escapes_process_group() {
        let temp = tempfile::tempdir().expect("tempdir");
        let marker = temp.path().join("escaped-writer-ready");
        let mut command = Command::new(std::env::current_exe().expect("test executable"));
        command
            .args(["escaped_pipe_writer_helper", "--nocapture"])
            .env(HELPER_PHASE, "parent")
            .env(HELPER_MARKER, &marker);

        let started = Instant::now();
        assert!(run(&mut command, b"missing-marker", Duration::from_millis(100)).is_none());

        assert!(
            marker.exists(),
            "escaped descendant did not leave the process group"
        );
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn escaped_pipe_writer_helper() {
        match std::env::var(HELPER_PHASE).as_deref() {
            Ok("parent") => spawn_escaped_writer(),
            Ok("child") => hold_inherited_stdout(),
            _ => {}
        }
    }

    fn spawn_escaped_writer() {
        let marker = std::env::var_os(HELPER_MARKER).expect("marker path");
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args(["escaped_pipe_writer_helper", "--nocapture"])
            .env(HELPER_PHASE, "child")
            .env(HELPER_MARKER, &marker)
            .spawn()
            .expect("escaped writer");
        std::thread::spawn(move || {
            let _ = child.wait();
        });

        let marker = PathBuf::from(marker);
        let started = Instant::now();
        while !marker.exists() && started.elapsed() < Duration::from_secs(1) {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(marker.exists());
    }

    fn hold_inherited_stdout() {
        assert_ne!(unsafe { libc::setsid() }, -1);
        let marker = PathBuf::from(std::env::var_os(HELPER_MARKER).expect("marker path"));
        std::fs::write(marker, b"ready").expect("write marker");
        std::thread::sleep(Duration::from_secs(5));
    }
}
