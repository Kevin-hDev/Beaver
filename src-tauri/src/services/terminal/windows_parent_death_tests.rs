use super::caller::authorize;
use super::PtyManager;
use crate::app_exit::AppExitCoordinator;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

const ROLE_ENV: &str = "BEAVER_WINDOWS_TERMINAL_PARENT_ROLE";
const READY_ENV: &str = "BEAVER_WINDOWS_TERMINAL_PARENT_READY";
const PID_ENV: &str = "BEAVER_WINDOWS_TERMINAL_PARENT_PID";
const PARENT_TEST: &str = "services::terminal::windows_parent_death_tests::terminal_parent_probe";

#[test]
fn windows_job_kills_terminal_when_beaver_parent_exits() {
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
    assert!(wait_until_dead(
        read_pid(&child_pid),
        Duration::from_secs(2)
    ));
}

#[test]
#[ignore = "subprocess entry point"]
fn terminal_parent_probe() {
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("parent"));
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = PtyManager::new(coordinator.work_supervisor());
    let owner = authorize("main").expect("main owner");
    let (id, _) = manager
        .spawn_for_test(&owner, None, 80, 24)
        .expect("terminal shell");
    let pid = manager.process_id_for_test(id).expect("terminal pid");
    std::fs::write(required_path(PID_ENV), pid.to_string()).expect("child pid");
    std::fs::write(required_path(READY_ENV), b"ready").expect("ready marker");
    std::process::exit(0);
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
        let mut system = System::new();
        system.refresh_processes(
            sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
            true,
        );
        if system.process(Pid::from_u32(pid)).is_none() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}
