#![cfg(target_os = "linux")]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const ROLE_ENV: &str = "BEAVER_TERMINAL_PARENT_DEATH_ROLE";
const READY_ENV: &str = "BEAVER_TERMINAL_PARENT_DEATH_READY";
const PID_ENV: &str = "BEAVER_TERMINAL_PARENT_DEATH_PID";
const PARENT_TEST: &str = "terminal_helper_parent_probe";

#[test]
fn terminal_helper_dies_when_intermediate_parent_exits() {
    let temp = tempfile::tempdir().expect("probe tempdir");
    let ready = temp.path().join("ready");
    let child_pid = temp.path().join("child-pid");
    let status = test_command(PARENT_TEST)
        .env(ROLE_ENV, "parent")
        .env(READY_ENV, &ready)
        .env(PID_ENV, &child_pid)
        .status()
        .expect("probe parent");
    assert!(status.success());
    wait_for_path(&ready, Duration::from_secs(2));
    let pid = read_pid(&child_pid);
    assert!(wait_until_dead(pid, Duration::from_secs(2)));
}

#[test]
#[ignore = "subprocess entry point"]
#[expect(
    clippy::zombie_processes,
    reason = "the intermediate parent must exit without waiting to prove PDEATHSIG"
)]
fn terminal_helper_parent_probe() {
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("parent"));
    let mut helper = Command::new(env!("CARGO_BIN_EXE_cl-go-dash"));
    helper
        .args([
            "--beaver-terminal-shell-helper",
            &std::process::id().to_string(),
            "--",
            "/bin/sleep",
            "30",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child = helper.spawn().expect("terminal helper");
    std::fs::write(required_path(PID_ENV), child.id().to_string()).expect("child pid");
    std::fs::write(required_path(READY_ENV), b"ready").expect("ready marker");
    unsafe { libc::_exit(0) };
}

fn test_command(test_name: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args(["--ignored", "--exact", test_name, "--nocapture"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn required_path(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).expect("probe path"))
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
        .expect("child pid marker")
        .parse()
        .expect("numeric child pid")
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
