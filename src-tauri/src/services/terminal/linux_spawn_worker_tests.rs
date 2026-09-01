use super::linux_spawn_worker::LinuxSpawnWorker;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::time::{Duration, Instant};

const READY_ENV: &str = "BEAVER_LINUX_WORKER_PARENT_DEATH_READY";
const EXPECTED_PARENT_ENV: &str = "BEAVER_LINUX_WORKER_EXPECTED_PARENT";
const CHILD_TEST: &str = "services::terminal::linux_spawn_worker_tests::parent_death_child_probe";

fn worker() -> (AppExitCoordinator, LinuxSpawnWorker) {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    (coordinator, LinuxSpawnWorker::new(work))
}

#[tokio::test]
async fn one_named_worker_serves_successive_requests() {
    let (_coordinator, worker) = worker();
    let first = worker.run_test_probe(|| 1).await.unwrap();
    let second = worker.run_test_probe(|| 2).await.unwrap();
    let diagnostics = worker.diagnostics_for_test();
    assert_eq!(
        first.thread_name.as_deref(),
        Some("beaver-terminal-linux-spawn")
    );
    assert_eq!(first.thread_id, second.thread_id);
    assert_eq!((first.value, second.value), (1, 2));
    assert_eq!((diagnostics.active, diagnostics.high_water), (1, 1));
    assert!(
        worker
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn queue_is_bounded_to_sixteen_pending_requests() {
    let (_coordinator, worker) = worker();
    let (entered, observed) = std::sync::mpsc::sync_channel(1);
    let (release, blocked) = std::sync::mpsc::sync_channel(1);
    let first = worker
        .queue_test_probe(move || {
            entered.send(()).unwrap();
            blocked.recv().unwrap();
            0
        })
        .unwrap();
    observed.recv_timeout(Duration::from_secs(1)).unwrap();
    let pending = (0..16)
        .map(|value| worker.queue_test_probe(move || value))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        worker.queue_test_probe(|| 17).unwrap_err(),
        "terminal-error"
    );
    release.send(()).unwrap();
    first.await.unwrap().unwrap();
    for result in pending {
        result.await.unwrap().unwrap();
    }
    assert!(
        worker
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}

#[tokio::test]
async fn panic_is_generic_and_does_not_kill_the_worker() {
    let (_coordinator, worker) = worker();
    assert_eq!(
        worker
            .run_test_probe(|| panic!("private panic payload"))
            .await
            .err(),
        Some("terminal-error".to_string())
    );
    assert_eq!(worker.run_test_probe(|| 7).await.unwrap().value, 7);
    assert!(
        worker
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(worker.run_test_probe(|| 3).await.is_err());
}

#[tokio::test]
async fn closing_refuses_new_requests_and_joins_the_worker() {
    let (_coordinator, worker) = worker();
    worker.run_test_probe(|| 1).await.unwrap();
    worker.begin_closing();
    assert_eq!(
        worker.run_test_probe(|| 2).await.err(),
        Some("terminal-shutting-down".to_string())
    );
    assert!(
        worker
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}

#[test]
fn linux_pty_spawn_uses_the_durable_worker() {
    let command_source = include_str!("../../commands/terminal.rs");
    let session_source = include_str!("pty_session_unix.rs");
    assert!(command_source.contains("manager.spawn_linux"));
    assert!(command_source.contains("#[cfg(target_os = \"linux\")]"));
    assert!(session_source.contains("shell_helper::ROLE_FLAG"));
    assert!(!session_source.contains("terminal_blocking::run"));
    assert!(include_str!("shell_helper.rs").contains("--beaver-terminal-shell-helper"));
}

#[tokio::test]
async fn child_outlives_fifteen_seconds_on_the_durable_worker_then_dies() {
    let (_coordinator, worker) = worker();
    let temp = tempfile::tempdir().unwrap();
    let ready = temp.path().join("ready");
    let child_ready = ready.clone();
    let pid = worker
        .run_test_probe(move || spawn_parent_death_probe(&child_ready) as usize)
        .await
        .unwrap()
        .value as u32;
    wait_for_path(&ready, Duration::from_secs(2));

    tokio::time::sleep(Duration::from_secs(16)).await;
    assert!(process_is_running(pid));
    assert!(
        worker
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    assert!(wait_until_dead(pid, Duration::from_secs(2)));
}

#[test]
fn child_dies_when_its_ephemeral_creator_thread_exits() {
    let temp = tempfile::tempdir().unwrap();
    let ready = temp.path().join("ready");
    let child_ready = ready.clone();
    let pid = std::thread::spawn(move || spawn_parent_death_probe(&child_ready))
        .join()
        .unwrap();
    wait_for_path(&ready, Duration::from_secs(2));
    assert!(wait_until_dead(pid, Duration::from_secs(2)));
}

#[test]
#[ignore = "subprocess entry point"]
fn parent_death_child_probe() {
    let expected_parent = std::env::var(EXPECTED_PARENT_ENV).unwrap().parse().unwrap();
    super::shell_helper::arm_parent_death_signal(expected_parent).unwrap();
    std::fs::write(required_path(READY_ENV), b"ready").unwrap();
    std::thread::sleep(Duration::from_secs(30));
}

#[expect(
    clippy::zombie_processes,
    reason = "the probe must stay owned by its creator thread until PDEATHSIG fires"
)]
fn spawn_parent_death_probe(ready: &std::path::Path) -> u32 {
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", CHILD_TEST, "--nocapture"])
        .env(READY_ENV, ready)
        .env(EXPECTED_PARENT_ENV, std::process::id().to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    wait_for_path(ready, Duration::from_secs(2));
    pid
}

fn required_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os(name).unwrap())
}

fn wait_for_path(path: &std::path::Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline && !path.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(path.exists());
}

fn process_is_running(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

fn wait_until_dead(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !process_is_running(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}
