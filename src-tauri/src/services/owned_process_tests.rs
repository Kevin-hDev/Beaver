use super::owned_process::{OwnedProcess, OwnedProcessError};
use super::process_tree::ProcessKind;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};

static REJECTED_PID: AtomicU32 = AtomicU32::new(0);

fn fixture_command() -> Command {
    #[cfg(windows)]
    let mut command = {
        let mut command = Command::new("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Sleep -Seconds 30",
        ]);
        command
    };
    #[cfg(unix)]
    let mut command = {
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

fn process_exists(pid: u32) -> bool {
    let mut system = System::new();
    let pid = Pid::from_u32(pid);
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
    system.process(pid).is_some()
}

fn rejected_admission(pid: u32) -> Result<(), OwnedProcessError> {
    REJECTED_PID.store(pid, Ordering::Release);
    Err(OwnedProcessError::Admission)
}

fn assert_reaped(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("owned fixture remained alive after rejection");
}

#[test]
fn owned_spawn_enters_native_confinement_before_returning() {
    let mut command = fixture_command();
    let mut child = OwnedProcess::spawn(&mut command, ProcessKind::Terminal).expect("owned child");
    let pid = child.id();

    assert!(OwnedProcess::is_confined_for_test(pid));

    super::process_tree::terminate(&mut child, ProcessKind::Terminal);
}

#[test]
fn failed_native_admission_reaps_the_spawned_child() {
    REJECTED_PID.store(0, Ordering::Release);
    let mut command = fixture_command();

    let error = OwnedProcess::spawn_with_admitter_for_test(
        &mut command,
        ProcessKind::Terminal,
        rejected_admission,
    )
    .expect_err("native admission must fail");
    let pid = REJECTED_PID.load(Ordering::Acquire);

    assert_eq!(error, OwnedProcessError::Admission);
    assert!(pid >= 2);
    assert_reaped(pid);
}

#[tokio::test]
async fn owned_tokio_spawn_enters_native_confinement_before_returning() {
    let mut command = tokio::process::Command::from(fixture_command());
    let mut child = OwnedProcess::spawn_tokio(&mut command, ProcessKind::Terminal)
        .await
        .expect("owned Tokio child");
    let pid = child.id().expect("child pid");

    assert!(OwnedProcess::is_confined_for_test(pid));

    super::process_tree::terminate_tokio(&mut child, ProcessKind::Terminal).await;
}
