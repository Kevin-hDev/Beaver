use super::{AppUpdateRuntime, MAX_APP_UPDATE_DOWNLOADS};
use crate::app_exit::AppExitCoordinator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn update_download_admission_is_bounded_and_released_on_failure() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let runtime = AppUpdateRuntime::new(coordinator.work_supervisor());
    let admissions = (0..MAX_APP_UPDATE_DOWNLOADS)
        .map(|_| runtime.try_admit().expect("update download slot"))
        .collect::<Vec<_>>();

    assert_eq!(runtime.try_admit().unwrap_err(), "update-download-error");
    drop(admissions);

    let result = runtime
        .run_download(|_| async { Err::<(), _>("update-download-error".to_string()) })
        .await;
    assert_eq!(result.unwrap_err(), "update-download-error");
    assert_eq!(runtime.diagnostics().active, 0);
}

#[tokio::test]
async fn shutdown_cancels_waits_and_permanently_refuses_update_downloads() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let runtime = AppUpdateRuntime::new(coordinator.work_supervisor());
    let observed = Arc::new(AtomicBool::new(false));
    let task_runtime = runtime.clone();
    let task_observed = Arc::clone(&observed);
    let running = tokio::spawn(async move {
        task_runtime
            .run_download(move |cancel| async move {
                cancel.cancelled().await;
                task_observed.store(true, Ordering::Release);
                Err::<(), _>("update-download-error".to_string())
            })
            .await
    });

    tokio::task::yield_now().await;
    assert!(
        runtime
            .stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(running.await.unwrap().unwrap_err(), "update-download-error");
    assert!(observed.load(Ordering::Acquire));
    assert_eq!(runtime.diagnostics().active, 0);
    assert_eq!(
        runtime
            .run_download(|_| async { Ok::<(), String>(()) })
            .await
            .unwrap_err(),
        "update-download-error"
    );
}
