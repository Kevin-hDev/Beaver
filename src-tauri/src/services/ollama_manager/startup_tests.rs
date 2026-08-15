use super::error::OllamaErrorCode;
use super::manager::OllamaManager;
use super::startup::{OllamaStartupBarrier, StartupBarrierState};
use crate::app_exit::AppExitCoordinator;
use std::time::Duration;

#[tokio::test]
async fn startup_waits_while_pending_and_opens_once_ready() {
    let barrier = OllamaStartupBarrier::new();
    assert_eq!(barrier.state(), StartupBarrierState::Pending);
    let wait = tokio::spawn({
        let barrier = barrier.clone();
        async move { barrier.wait_ready().await }
    });
    tokio::task::yield_now().await;
    assert!(!wait.is_finished());
    barrier.publish(StartupBarrierState::Ready);
    assert_eq!(barrier.wait_ready().await, StartupBarrierState::Ready);
    assert_eq!(wait.await.unwrap(), StartupBarrierState::Ready);
}

#[test]
fn blocked_state_preserves_recovery_code() {
    let barrier = OllamaStartupBarrier::new();
    barrier.publish(StartupBarrierState::Blocked {
        code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
    });
    assert_eq!(
        barrier.state(),
        StartupBarrierState::Blocked {
            code: OllamaErrorCode::OllamaUpdateRecoveryRequired,
        }
    );
}

#[tokio::test]
async fn polling_waits_for_pending_and_blocked_is_not_ready() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let pending = tokio::time::timeout(Duration::from_millis(10), manager.poll_once()).await;
    assert!(pending.is_err());
    manager.publish_startup_for_test(
        manager.generation_for_test(),
        StartupBarrierState::Blocked {
            code: OllamaErrorCode::OllamaRecoveryDeferred,
        },
    );
    let blocked = tokio::time::timeout(Duration::from_millis(10), manager.poll_once())
        .await
        .expect("blocked polling returns");
    assert_eq!(blocked.last_error, None);
}

#[test]
fn stale_generation_cannot_replace_newer_barrier_state() {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    let old_generation = manager.generation_for_test();
    manager.set_generation_for_test(old_generation + 1);
    manager.publish_startup_for_test(old_generation, StartupBarrierState::Ready);
    assert_eq!(manager.startup_state(), StartupBarrierState::Pending);
    manager.publish_startup_for_test(old_generation + 1, StartupBarrierState::Ready);
    assert_eq!(manager.startup_state(), StartupBarrierState::Ready);
}
