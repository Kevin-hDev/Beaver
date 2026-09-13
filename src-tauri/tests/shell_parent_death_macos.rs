#![cfg(target_os = "macos")]

use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const READY_ENV: &str = "BEAVER_SHELL_GUARD_READY";
const SHELL_PID_ENV: &str = "BEAVER_SHELL_GUARD_SHELL_PID";
const DESCENDANT_PID_ENV: &str = "BEAVER_SHELL_GUARD_DESCENDANT_PID";
const OUTPUT_ENV: &str = "BEAVER_SHELL_GUARD_OUTPUT";

#[test]
fn macos_agent_shell_dies_with_beaver() {
    let temp = tempfile::tempdir().expect("probe tempdir");
    let ready = temp.path().join("ready");
    let shell_pid = temp.path().join("shell-pid");
    let descendant_pid = temp.path().join("descendant-pid");
    let output = temp.path().join("output");
    let mut beaver = test_command("macos_shell_parent_probe");
    beaver
        .env(READY_ENV, &ready)
        .env(SHELL_PID_ENV, &shell_pid)
        .env(DESCENDANT_PID_ENV, &descendant_pid)
        .env(OUTPUT_ENV, &output);
    let mut beaver = beaver.spawn().expect("simulated Beaver");

    wait_for_path(&ready, Duration::from_secs(5));
    let shell_pid = read_pid(&shell_pid);
    let descendant_pid = read_pid(&descendant_pid);
    assert_eq!(unsafe { libc::kill(beaver.id() as i32, libc::SIGKILL) }, 0);
    let _ = beaver.wait();

    assert!(wait_until_dead(shell_pid, Duration::from_secs(5)));
    assert!(wait_until_dead(descendant_pid, Duration::from_secs(5)));
    let size = std::fs::metadata(&output).expect("output metadata").len();
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(
        std::fs::metadata(output).expect("stable output").len(),
        size
    );
}

#[test]
fn macos_guard_keeps_ready_byte_private() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cl-go-dash"));
    command
        .args(["--beaver-shell-guard", "--", "/usr/bin/printf", "visible"])
        .process_group(0);
    let output = command.output().expect("guarded command");
    assert!(
        output.status.success(),
        "guard stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"visible");
    assert!(output.stderr.is_empty());
}

#[test]
#[ignore = "subprocess entry point"]
fn macos_shell_parent_probe() {
    let script = concat!(
        "echo $$ > \"$BEAVER_SHELL_GUARD_SHELL_PID\"; ",
        "(while :; do echo tick >> \"$BEAVER_SHELL_GUARD_OUTPUT\"; sleep 0.02; done) & ",
        "echo $! > \"$BEAVER_SHELL_GUARD_DESCENDANT_PID\"; ",
        "echo ready > \"$BEAVER_SHELL_GUARD_READY\"; wait"
    );
    let mut helper = Command::new(env!("CARGO_BIN_EXE_cl-go-dash"));
    helper
        .args(["--beaver-shell-guard", "--", "/bin/sh", "-c", script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    helper.process_group(0);
    let status = helper.status().expect("guarded shell");
    assert!(status.success());
}

fn test_command(name: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args(["--ignored", "--exact", name, "--nocapture"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn wait_for_path(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline && !path.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(path.exists(), "probe marker deadline");
}

fn read_pid(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .expect("pid marker")
        .trim()
        .parse()
        .expect("numeric pid")
}

fn wait_until_dead(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if unsafe { libc::kill(pid as i32, 0) } == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}
