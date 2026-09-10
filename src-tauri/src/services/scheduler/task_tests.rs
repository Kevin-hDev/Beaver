use super::Scheduler;
use crate::app_exit::AppExitCoordinator;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

#[tokio::test]
async fn scheduler_shutdown_waits_for_cleanup_and_refuses_late_wakeups() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let scheduler = Scheduler::for_test(coordinator.work_supervisor());
    let finished = Arc::new(AtomicBool::new(false));
    let task_finished = Arc::clone(&finished);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    scheduler
        .spawn_wakeup_for_test(move |cancel| async move {
            let _ = started_tx.send(());
            cancel.cancelled().await;
            tokio::time::sleep(Duration::from_millis(20)).await;
            task_finished.store(true, Ordering::Release);
        })
        .expect("scheduled wakeup");
    started_rx.await.unwrap();

    assert!(
        scheduler
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(finished.load(Ordering::Acquire));
    assert_eq!(
        scheduler
            .spawn_wakeup_for_test(|_| async {})
            .expect_err("stopped scheduler must refuse work")
            .public_code(),
        "service-shutting-down"
    );
}

#[tokio::test]
async fn scheduler_shutdown_waits_for_its_persistent_loop() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let scheduler = Scheduler::for_test(coordinator.work_supervisor());
    let finished = Arc::new(AtomicBool::new(false));
    let loop_finished = Arc::clone(&finished);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    scheduler
        .spawn_loop_for_test(move |cancel| async move {
            let _ = started_tx.send(());
            cancel.cancelled().await;
            loop_finished.store(true, Ordering::Release);
        })
        .expect("scheduler loop");
    started_rx.await.unwrap();

    assert!(
        scheduler
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(finished.load(Ordering::Acquire));
}
