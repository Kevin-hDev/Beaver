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
