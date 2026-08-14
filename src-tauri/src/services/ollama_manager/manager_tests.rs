use super::error::OllamaErrorCode;
use super::types::OperationState;
use super::OllamaManager;
use crate::app_exit::AppExitCoordinator;
use std::future;
use std::time::Duration;

#[path = "types_tests.rs"]
mod types_tests;

fn manager() -> (AppExitCoordinator, OllamaManager) {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    (coordinator, manager)
}

async fn wait_for_no_active_work(manager: &OllamaManager) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while manager.work_diagnostics_for_test().active != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Ollama admission released");
}

#[tokio::test]
async fn one_manager_admission_is_active_at_a_time() {
    let (_coordinator, manager) = manager();
    let first = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("first admission");

    let second = manager.begin_operation(OperationState::Updating).await;
    assert!(matches!(
        second,
        Err(OllamaErrorCode::OllamaOperationInProgress)
    ));
    drop(first);
    wait_for_no_active_work(&manager).await;
}

#[tokio::test]
async fn closing_refuses_new_operations() {
    let (_coordinator, manager) = manager();
    manager.begin_closing();

    let result = manager.begin_operation(OperationState::Installing).await;
    assert!(matches!(result, Err(OllamaErrorCode::OllamaClosing)));
}

#[tokio::test]
async fn cancellation_marks_cancelling_and_releases_admission() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Updating)
        .await
        .expect("operation admission");
    manager.begin_closing();
    drop(operation);

    wait_for_no_active_work(&manager).await;
    assert_eq!(manager.status().await.operation, OperationState::Cancelling);
}

#[tokio::test]
async fn an_error_releases_the_admission() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("operation admission");
    operation.fail_for_test(OllamaErrorCode::OllamaInternal);

    wait_for_no_active_work(&manager).await;
    let status = manager.status().await;
    assert_eq!(status.operation, OperationState::Idle);
    assert_eq!(status.last_error, Some(OllamaErrorCode::OllamaInternal));
}

#[tokio::test]
async fn a_panicking_task_releases_the_admission() {
    let (_coordinator, manager) = manager();
    let task_manager = manager.clone();
    let task = tokio::spawn(async move {
        let _operation = task_manager
            .begin_operation(OperationState::Recovering)
            .await
            .expect("operation admission");
        panic!("expected task panic");
    });

    assert!(task.await.expect_err("task must panic").is_panic());
    wait_for_no_active_work(&manager).await;
    assert_eq!(manager.status().await.operation, OperationState::Idle);
}

#[tokio::test]
async fn abandoning_a_future_releases_the_admission() {
    let (_coordinator, manager) = manager();
    let task_manager = manager.clone();
    let task = tokio::spawn(async move {
        let _operation = task_manager
            .begin_operation(OperationState::Installing)
            .await
            .expect("operation admission");
        future::pending::<()>().await;
    });

    tokio::time::timeout(Duration::from_secs(1), async {
        while manager.work_diagnostics_for_test().active != 1 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("operation became active");
    task.abort();
    assert!(task
        .await
        .expect_err("future must be abandoned")
        .is_cancelled());
    wait_for_no_active_work(&manager).await;
    assert_eq!(manager.status().await.operation, OperationState::Idle);
}

#[tokio::test]
async fn stale_generation_cannot_reset_a_new_operation() {
    let (_coordinator, manager) = manager();
    let old = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("old operation admission");
    let old_generation = old.generation_for_test();
    manager.supersede_generation_for_test(OperationState::Updating);

    drop(old);
    assert_eq!(manager.status().await.operation, OperationState::Updating);
    manager.release_generation_for_test(old_generation);
    assert_eq!(manager.status().await.operation, OperationState::Updating);
}

#[tokio::test]
async fn one_hundred_cycles_reuse_the_single_slot() {
    let (_coordinator, manager) = manager();
    for _ in 0..100 {
        let operation = manager
            .begin_operation(OperationState::Installing)
            .await
            .expect("slot must be reusable");
        drop(operation);
        wait_for_no_active_work(&manager).await;
    }
    assert_eq!(manager.work_diagnostics_for_test().saturation_refusals, 0);
}

#[tokio::test]
async fn generation_overflow_fails_closed_without_recycling() {
    let (_coordinator, manager) = manager();
    manager.set_generation_for_test(u64::MAX);

    let result = manager.begin_operation(OperationState::Installing).await;
    assert!(matches!(result, Err(OllamaErrorCode::OllamaInternal)));
    assert_eq!(manager.generation_for_test(), u64::MAX);
    assert_eq!(manager.work_diagnostics_for_test().active, 0);
}
