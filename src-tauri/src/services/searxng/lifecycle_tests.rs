use super::lifecycle::SearxngSidecar;
use crate::app_exit::AppExitCoordinator;
use std::time::{Duration, Instant};

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

#[test]
fn safe_log_error_removes_control_chars_and_truncates() {
    let input = format!("SearXNG: timeout\n{}", "x".repeat(400));
    let output = super::lifecycle::safe_log_error(&input);
    assert!(!output.contains('\n'));
    assert!(output.chars().count() <= 240);
}

#[test]
fn start_failure_cache_can_be_cleared() {
    super::lifecycle::clear_start_failure();
    super::lifecycle::remember_start_failure("SearXNG: arrêt au démarrage");
    assert_eq!(
        super::lifecycle::recent_start_failure(),
        Some("SearXNG: arrêt au démarrage".to_string())
    );
    super::lifecycle::clear_start_failure();
    assert_eq!(super::lifecycle::recent_start_failure(), None);
}
