use super::lifecycle::SearxngSidecar;
use crate::app_exit::AppExitCoordinator;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

#[test]
fn stale_post_readiness_start_cannot_clean_the_previous_runtime() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = SearxngSidecar::new(coordinator.work_supervisor());
    let admission = sidecar.work.try_admit_server().expect("server admission");
    let cancel = admission.cancellation();
    let generation = sidecar.publication_generation.load(Ordering::Acquire);
    sidecar
        .publication_generation
        .fetch_add(1, Ordering::AcqRel);
    let mut cleaned = false;

    let result = super::start_readiness::run_if_start_active(&sidecar, &cancel, generation, || {
        cleaned = true
    });

    assert!(result.is_err());
    assert!(!cleaned);
}

#[tokio::test]
async fn shutdown_permanently_refuses_a_new_searxng_start() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = SearxngSidecar::new(coordinator.work_supervisor());

    assert!(
        sidecar
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(sidecar.try_admit_start_for_test().is_err());
}

#[tokio::test]
async fn shutdown_reaps_a_real_python_sidecar_process() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = SearxngSidecar::new(coordinator.work_supervisor());
    let pid = sidecar
        .start_test_process_for_test()
        .await
        .expect("real Python fixture");

    assert!(
        sidecar
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    let mut processes = sysinfo::System::new();
    processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    assert!(processes.process(sysinfo::Pid::from_u32(pid)).is_none());
}

#[tokio::test]
async fn shutdown_does_not_wait_for_a_slow_unpublished_start() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = SearxngSidecar::new(coordinator.work_supervisor());
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let starting = sidecar.clone();
    let startup = tokio::spawn(async move {
        starting
            .suspend_test_start_before_publication_for_test(started_tx, release_rx)
            .await
    });
    let pid = started_rx.await.expect("fixture started");
    let deadline = Instant::now() + Duration::from_millis(100);

    let returned =
        tokio::time::timeout(Duration::from_millis(750), sidecar.stop_and_wait(deadline))
            .await
            .is_ok();
    let _ = release_tx.send(());
    let _ = startup.await;
    let child_gone = wait_until_process_is_gone(pid, Duration::from_secs(2)).await;

    assert!(returned, "shutdown waited on an unpublished start lock");
    assert!(
        child_gone,
        "unpublished kill_on_drop child survived shutdown"
    );
}

#[tokio::test]
async fn stale_generation_cannot_publish_over_shutdown() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = SearxngSidecar::new(coordinator.work_supervisor());

    let pid = sidecar
        .reject_stale_test_publication_for_test()
        .await
        .expect("stale fixture");

    assert!(sidecar.published_pid_for_test().await.is_none());
    assert!(wait_until_process_is_gone(pid, Duration::from_secs(2)).await);
}

async fn wait_until_process_is_gone(pid: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        let mut processes = sysinfo::System::new();
        processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        if processes.process(sysinfo::Pid::from_u32(pid)).is_none() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[test]
fn safe_log_error_removes_control_chars_and_truncates() {
    let input = format!("SearXNG: timeout\n{}", "x".repeat(400));
    let output = super::startup_failure::safe_log_error(&input);
    assert!(!output.contains('\n'));
    assert!(output.chars().count() <= 240);
}

#[test]
fn start_failure_cache_can_be_cleared() {
    super::startup_failure::clear();
    super::startup_failure::remember("SearXNG: arrêt au démarrage");
    assert_eq!(
        super::startup_failure::recent(),
        Some("SearXNG: arrêt au démarrage".to_string())
    );
    super::startup_failure::clear();
    assert_eq!(super::startup_failure::recent(), None);
}
