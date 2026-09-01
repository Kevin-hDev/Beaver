use super::linux_spawn_worker::LinuxSpawnWorker;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn worker() -> (AppExitCoordinator, LinuxSpawnWorker) {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = ServiceWorkSupervisor::<1>::new(coordinator.work_supervisor());
    (coordinator, LinuxSpawnWorker::new(work))
}

#[tokio::test]
async fn dead_worker_returns_terminal_error_without_admitting_new_work() {
    let (_coordinator, worker) = worker();
    let probe = worker.run_test_probe(|| 1).await.unwrap();
    assert_eq!(probe.value, 1);
    assert_eq!(
        probe.thread_name.as_deref(),
        Some("beaver-terminal-linux-spawn")
    );
    assert_ne!(probe.thread_id, std::thread::current().id());
    assert_eq!(
        worker
            .submit(Box::new(|| Ok((7, "token".to_string()))))
            .await,
        Ok((7, "token".to_string()))
    );
    worker.terminate_for_test().await;
    let ran = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&ran);

    let error = worker
        .run_test_probe(move || {
            observed.store(true, Ordering::Release);
            2
        })
        .await
        .unwrap_err();

    assert_eq!(error, "terminal-error");
    assert!(!ran.load(Ordering::Acquire));
    let observed = Arc::clone(&ran);
    let second_error = worker
        .run_test_probe(move || {
            observed.store(true, Ordering::Release);
            3
        })
        .await
        .unwrap_err();
    assert_eq!(second_error, "terminal-error");
    assert!(!ran.load(Ordering::Acquire));
    assert_eq!(worker.diagnostics_for_test().active, 0);
}

#[tokio::test]
async fn test_configuration_can_close_a_live_worker() {
    let (_coordinator, worker) = worker();
    worker.run_test_probe(|| 1).await.unwrap();

    assert!(
        worker
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
}
