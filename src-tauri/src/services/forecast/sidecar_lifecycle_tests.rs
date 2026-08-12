use super::sidecar::ChronosSidecar;
use crate::app_exit::AppExitCoordinator;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

#[tokio::test]
async fn shutdown_reaps_a_real_sidecar_and_refuses_future_work() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = ChronosSidecar::new(coordinator.work_supervisor());
    let pid = sidecar
        .start_test_process_for_test()
        .await
        .expect("real Forecast fixture");
    super::sidecar::schedule_idle_stop(&sidecar).await;
    super::sidecar::schedule_idle_stop(&sidecar).await;
    tokio::time::timeout(Duration::from_secs(1), async {
        while sidecar.idle_counts_for_test().0 != 1 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("single idle worker");
    assert_eq!(sidecar.idle_counts_for_test(), (1, 1));

    assert!(
        sidecar
            .stop_and_wait(Instant::now() + Duration::from_secs(2))
            .await
    );
    let mut processes = sysinfo::System::new();
    processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    assert!(processes.process(sysinfo::Pid::from_u32(pid)).is_none());
    assert_eq!(sidecar.idle_counts_for_test().0, 0);
    assert!(sidecar.try_admit_operation_for_test().is_err());
}

#[tokio::test]
async fn shutdown_waits_for_an_active_forecast_operation() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = ChronosSidecar::new(coordinator.work_supervisor());
    let running = sidecar.clone();
    let finished = Arc::new(AtomicBool::new(false));
    let task_finished = Arc::clone(&finished);
    let (started_sender, started_receiver) = tokio::sync::oneshot::channel();
    let operation = tokio::spawn(async move {
        running
            .run_operation(|cancel| async move {
                let _ = started_sender.send(());
                cancel.cancelled().await;
                tokio::time::sleep(Duration::from_millis(20)).await;
                task_finished.store(true, Ordering::Release);
                Ok::<_, String>(())
            })
            .await
    });
    started_receiver.await.unwrap();

    assert!(
        sidecar
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(finished.load(Ordering::Acquire));
    assert!(operation.await.unwrap().is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_does_not_wait_for_a_slow_health_probe_lock() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = ChronosSidecar::new(coordinator.work_supervisor());
    sidecar
        .start_test_process_for_test()
        .await
        .expect("real Forecast fixture");
    let (started_tx, started_rx) = std::sync::mpsc::sync_channel(1);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let probing = sidecar.clone();
    let probe = tokio::spawn(async move {
        probing
            .probe_running_for_test(move |port, _token| {
                started_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                Some((port, "fixture".to_string(), "fixture".to_string()))
            })
            .await
    });
    started_rx.recv_timeout(Duration::from_secs(1)).unwrap();

    let stopped_quickly = tokio::time::timeout(
        Duration::from_millis(100),
        super::sidecar_stop::stop_state(&sidecar, Instant::now() + Duration::from_millis(100)),
    )
    .await
    .is_ok();
    release_tx.send(()).unwrap();
    let _ = probe.await;
    super::sidecar_stop::stop_state(&sidecar, Instant::now() + Duration::from_secs(1)).await;

    assert!(stopped_quickly, "health probe kept the process lock");
}

#[tokio::test]
async fn cancelling_before_publication_reaps_the_spawned_sidecar() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let sidecar = ChronosSidecar::new(coordinator.work_supervisor());
    let (spawned_tx, spawned_rx) = tokio::sync::oneshot::channel();
    let starting = tokio::spawn(async move {
        sidecar
            .hold_unpublished_test_process_for_test(spawned_tx)
            .await
    });
    let pid = spawned_rx.await.expect("fixture spawned");

    starting.abort();
    let _ = starting.await;
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let mut processes = sysinfo::System::new();
            processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            if processes.process(sysinfo::Pid::from_u32(pid)).is_none() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("unpublished sidecar reaped on cancellation");
}
