use super::linux_spawn_worker::LinuxSpawnWorker;
use crate::app_exit::AppExitCoordinator;
use crate::services::work_registry::ServiceWorkSupervisor;
use std::time::{Duration, Instant};

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
    let source = include_str!("../../commands/terminal.rs");
    assert!(source.contains("manager.spawn_linux"));
    assert!(source.contains("#[cfg(target_os = \"linux\")]"));
}
