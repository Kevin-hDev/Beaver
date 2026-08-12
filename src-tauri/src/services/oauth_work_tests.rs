use super::oauth_work::{OAuthWorkAdmissionError, OAuthWorkServices, MAX_OAUTH_FLOWS};
use crate::app_exit::AppExitCoordinator;
use std::time::{Duration, Instant};

#[test]
fn oauth_flow_registry_is_fixed_and_bounded() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = OAuthWorkServices::new(coordinator.work_supervisor());
    let admissions = (0..MAX_OAUTH_FLOWS)
        .map(|_| work.try_admit().expect("OAuth flow slot"))
        .collect::<Vec<_>>();

    assert_eq!(
        work.try_admit().expect_err("OAuth capacity must be fixed"),
        OAuthWorkAdmissionError::Busy
    );
    drop(admissions);
}

#[tokio::test]
async fn shutdown_cancels_waits_and_permanently_refuses_oauth_restart() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = OAuthWorkServices::new(coordinator.work_supervisor());
    let completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = completed.clone();
    work.spawn(move |cancel| async move {
        cancel.cancelled().await;
        observed.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .expect("supervised OAuth flow");

    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(
        work.try_admit().expect_err("OAuth restart after close"),
        OAuthWorkAdmissionError::ShuttingDown
    );
}

#[tokio::test]
async fn supervised_run_returns_only_after_its_cancelled_task_finishes() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let work = OAuthWorkServices::new(coordinator.work_supervisor());
    let running_work = work.clone();
    let (started, observed) = tokio::sync::oneshot::channel();
    let running = tokio::spawn(async move {
        running_work
            .run(move |cancel| async move {
                let _ = started.send(());
                cancel.cancelled().await;
                7_u8
            })
            .await
    });
    observed.await.expect("supervised task started");

    assert!(
        work.stop_and_wait(Instant::now() + Duration::from_secs(1))
            .await
    );
    assert_eq!(running.await.unwrap().unwrap(), 7);
}
