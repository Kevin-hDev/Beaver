use std::time::Duration;

use crate::app_exit::AppExitCoordinator;
use crate::services::owned_process::OwnedProcessIdentity;
use crate::services::work_registry::ServiceWorkSupervisor;

#[test]
fn startup_log_hint_exposes_only_a_fixed_category() {
    let root = tempfile::tempdir().unwrap();
    let log = root.path().join("sidecar.log");
    std::fs::write(&log, "ModuleNotFoundError: secret/path").unwrap();
    let body = String::from_utf8_lossy(&super::process::read_log_tail(&log).unwrap()).to_string();
    assert!(body.contains("secret/path"));
    assert_eq!(
        super::process::classify_log_hint(&body),
        Some("module-not-found")
    );
}

#[test]
fn startup_diagnostic_reads_only_the_bounded_tail() {
    let root = tempfile::tempdir().unwrap();
    let log = root.path().join("sidecar.log");
    let mut body = vec![b'x'; 32 * 1024];
    body.extend_from_slice(b"\nModuleNotFoundError: bounded-tail");
    std::fs::write(&log, body).unwrap();

    let tail = super::process::read_log_tail(&log).unwrap();
    assert!(tail.len() <= 16 * 1024);
    assert!(String::from_utf8_lossy(&tail).contains("bounded-tail"));
}

#[tokio::test]
async fn unstable_identity_stops_at_the_supplied_deadline() {
    let coordinator = AppExitCoordinator::initialize().unwrap();
    let supervisor = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    let admission = supervisor.try_admit().unwrap();
    let mut executable = 1_u128;
    let started = std::time::Instant::now();

    let result = super::process::stable_identity_with(
        42,
        tokio::time::Instant::now() + Duration::from_millis(35),
        &admission.cancellation(),
        |_| {
            executable += 1;
            Ok(OwnedProcessIdentity {
                pid: 42,
                native_start_time: 7,
                native_scope: 42,
                executable,
            })
        },
    )
    .await;

    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_millis(150));
}
