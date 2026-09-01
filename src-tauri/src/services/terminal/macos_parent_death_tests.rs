use super::caller::authorize;
use super::PtyManager;
use crate::app_exit::AppExitCoordinator;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

const ROLE_ENV: &str = "BEAVER_MACOS_TERMINAL_PARENT_ROLE";
const READY_ENV: &str = "BEAVER_MACOS_TERMINAL_PARENT_READY";
const RELEASE_ENV: &str = "BEAVER_MACOS_TERMINAL_PARENT_RELEASE";
const PID_ENV: &str = "BEAVER_MACOS_TERMINAL_PARENT_PID";
const PARENT_TEST: &str = "services::terminal::macos_parent_death_tests::terminal_parent_probe";

#[test]
fn macos_terminal_shell_dies_when_parent_process_exits() {
    let temp = tempfile::tempdir().expect("probe tempdir");
    let ready = temp.path().join("ready");
    let release = temp.path().join("release");
    let child_pid = temp.path().join("child-pid");
    let mut parent = test_command(PARENT_TEST)
        .env(ROLE_ENV, "parent")
        .env(READY_ENV, &ready)
        .env(RELEASE_ENV, &release)
        .env(PID_ENV, &child_pid)
        .spawn()
        .expect("probe parent");

    wait_for_path(&ready, Instant::now() + Duration::from_secs(2));
    let pid = read_pid(&child_pid);
    assert!(process_is_running(pid), "shell alive before parent exit");
    std::fs::write(&release, b"exit").expect("release parent");
    assert!(wait_for_parent_exit(&mut parent, Instant::now() + Duration::from_secs(2)).success());
    assert!(wait_until_dead(
        pid,
        Instant::now() + Duration::from_secs(2)
    ));
}

#[test]
#[ignore = "subprocess entry point"]
fn terminal_parent_probe() {
    assert_eq!(std::env::var(ROLE_ENV).as_deref(), Ok("parent"));
    let pid = spawn_shell_without_cleanup();
    std::fs::write(required_path(PID_ENV), pid.to_string()).expect("child pid");
    std::fs::write(required_path(READY_ENV), b"ready").expect("ready marker");
    wait_for_path(
        &required_path(RELEASE_ENV),
        Instant::now() + Duration::from_secs(2),
    );
    std::process::exit(0);
}

fn spawn_shell_without_cleanup() -> u32 {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = PtyManager::new(coordinator.work_supervisor());
    let owner = authorize("main").expect("main owner");
    let (id, _) = manager
        .spawn_for_test(&owner, None, 80, 24)
        .expect("terminal shell");
    let pid = manager.process_id_for_test(id).expect("terminal pid");
    // Le parent du probe doit garder le shell vivant sans conserver aucune
    // autorité de fermeture coordonnée après la publication du PID.
    std::mem::forget(manager);
    std::mem::forget(coordinator);
    pid
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

fn wait_for_path(path: &Path, deadline: Instant) {
    while Instant::now() < deadline && !path.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(path.exists(), "probe marker deadline");
}

fn wait_for_parent_exit(parent: &mut Child, deadline: Instant) -> ExitStatus {
    loop {
        if let Some(status) = parent.try_wait().expect("probe parent status") {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = parent.kill();
            let _ = parent.wait();
            panic!("probe parent deadline");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn read_pid(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .expect("child pid marker")
        .parse()
        .expect("numeric child pid")
}

fn process_is_running(pid: u32) -> bool {
    let mut system = System::new();
    system.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
    );
    system.process(Pid::from_u32(pid)).is_some()
}

fn wait_until_dead(pid: u32, deadline: Instant) -> bool {
    while Instant::now() < deadline {
        if !process_is_running(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}
