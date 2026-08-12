use super::runtime_background::RuntimeBackgroundServices;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn shutdown_cancels_and_waits_for_a_runtime_loop() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let background = RuntimeBackgroundServices::new(coordinator.work_supervisor());
    let stopped = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&stopped);

    background
        .spawn_loop(move |cancel| async move {
            cancel.cancelled().await;
            observed.store(true, Ordering::SeqCst);
        })
        .expect("runtime loop admission");

    assert!(
        background
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(stopped.load(Ordering::SeqCst));
}

#[tokio::test]
async fn runtime_tasks_cannot_restart_after_shutdown() {
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let background = RuntimeBackgroundServices::new(coordinator.work_supervisor());

    assert!(
        background
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(background.spawn_task(|_| async {}).is_err());
    assert!(background.spawn_loop(|_| async {}).is_err());
}
