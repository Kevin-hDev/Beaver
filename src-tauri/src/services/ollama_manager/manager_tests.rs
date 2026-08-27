use super::error::OllamaErrorCode;
use super::install::InstallRequest;
use super::progress::OllamaProgressUpdate;
use super::types::BundleState;
use super::types::{DaemonState, OllamaEndpoint};
use super::types::{OllamaProgressStage, OperationState};
use super::OllamaManager;
use crate::app_exit::AppExitCoordinator;
use std::future;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Notify;

#[path = "types_tests.rs"]
mod types_tests;

fn manager() -> (AppExitCoordinator, OllamaManager) {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    (coordinator, manager)
}

#[tokio::test]
async fn external_daemon_never_marks_the_beaver_bundle_ready() {
    let (_coordinator, manager) = manager();
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11_434).expect("port"));

    manager.publish_external_daemon(endpoint.clone());

    let status = manager.status().await;
    assert_eq!(status.bundle, BundleState::Absent);
    assert_eq!(status.daemon, DaemonState::External { endpoint });
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
async fn late_shutdown_still_reaps_when_no_setup_operation_holds_the_lock() {
    let (_coordinator, manager) = manager();

    assert!(manager
        .stop_for_shutdown(
            Instant::now() - Duration::from_millis(1),
            Instant::now() + Duration::from_secs(1),
        )
        .await
        .is_ok());
}

#[tokio::test]
async fn late_shutdown_does_not_cross_an_active_setup_operation() {
    let (_coordinator, manager) = manager();
    let _operation = manager.hold_operation_for_test().await;

    assert_eq!(
        manager
            .stop_for_shutdown(
                Instant::now() - Duration::from_millis(1),
                Instant::now() + Duration::from_secs(1),
            )
            .await,
        Err(OllamaErrorCode::OllamaSetupTimeout)
    );
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
async fn admitted_install_publishes_transaction_and_initial_progress() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("install admission");

    let status = manager.status().await;
    assert_eq!(status.bundle, BundleState::TransactionPending);
    assert_eq!(status.operation, OperationState::Installing);
    assert_eq!(status.progress, Some(OllamaProgressStage::Preparing));
    drop(operation);
}

#[tokio::test]
async fn stale_operation_progress_cannot_overwrite_the_current_generation() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("install admission");
    let reporter = manager.progress_reporter_for_generation(operation.generation_for_test(), None);
    reporter(OllamaProgressUpdate {
        stage: OllamaProgressStage::Downloading,
        completed: 4,
        total: 10,
    });
    assert_eq!(
        manager.status().await.progress,
        Some(OllamaProgressStage::Downloading)
    );

    manager.supersede_generation_for_test(OperationState::Recovering);
    reporter(OllamaProgressUpdate {
        stage: OllamaProgressStage::Committing,
        completed: 10,
        total: 10,
    });
    assert_eq!(
        manager.status().await.progress,
        Some(OllamaProgressStage::Downloading)
    );
    drop(operation);
}

#[tokio::test]
async fn closing_refuses_new_operations() {
    let (_coordinator, manager) = manager();
    manager.begin_closing();

    let result = manager.begin_operation(OperationState::Installing).await;
    assert!(matches!(result, Err(OllamaErrorCode::OllamaClosing)));
}

#[tokio::test]
async fn closing_wins_after_admission_before_publication() {
    let (_coordinator, manager) = manager();
    let admitted = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let admitted_wait = admitted.notified();
    tokio::pin!(admitted_wait);
    admitted_wait.as_mut().enable();
    let task_manager = manager.clone();
    let task_admitted = Arc::clone(&admitted);
    let task_resume = Arc::clone(&resume);
    let task = tokio::spawn(async move {
        let result = task_manager
            .begin_operation_paused_for_test(OperationState::Installing, task_admitted, task_resume)
            .await;
        result.map(drop)
    });

    admitted_wait.await;
    manager.begin_closing();
    resume.notify_one();

    let result = task.await.expect("operation task");
    assert!(matches!(result, Err(OllamaErrorCode::OllamaClosing)));
    assert_eq!(manager.status().await.operation, OperationState::Idle);
    wait_for_no_active_work(&manager).await;
}

#[tokio::test]
async fn cancellation_marks_cancelling_and_releases_admission() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Updating)
        .await
        .expect("operation admission");
    manager.begin_closing();
    assert_eq!(manager.status().await.operation, OperationState::Cancelling);
    drop(operation);

    wait_for_no_active_work(&manager).await;
    assert_eq!(manager.status().await.operation, OperationState::Cancelling);
}

#[tokio::test]
async fn cancellation_waits_for_an_imminent_operation_admission() {
    let (_coordinator, manager) = manager();
    let cancelling_manager = manager.clone();
    let cancellation = tokio_util::sync::CancellationToken::new();
    let registered_cancellation = cancellation.clone();
    let cancel_task = tokio::spawn(async move {
        cancelling_manager
            .cancel_operation_when_admitted(Duration::from_millis(250))
            .await
    });

    tokio::time::sleep(Duration::from_millis(20)).await;
    let operation = manager
        .begin_operation(OperationState::Updating)
        .await
        .expect("operation admission");
    manager.set_operation_cancellation(registered_cancellation);

    assert_eq!(
        cancel_task.await.expect("cancel task"),
        super::types::CancelOutcome::Cancelled
    );
    assert!(cancellation.is_cancelled());
    drop(operation);
}

#[tokio::test]
async fn a_token_registered_after_cancelling_is_cancelled_immediately() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Updating)
        .await
        .expect("operation admission");

    assert_eq!(
        manager.cancel_operation().await,
        super::types::CancelOutcome::Cancelled
    );
    let cancellation = tokio_util::sync::CancellationToken::new();
    manager.set_operation_cancellation(cancellation.clone());

    assert!(cancellation.is_cancelled());
    drop(operation);
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
async fn install_error_publishes_recovery_required_and_releases_admission() {
    let (_coordinator, manager) = manager();
    let root = tempfile::tempdir().unwrap();
    let request = InstallRequest::for_test(root.path().to_path_buf());
    std::fs::create_dir_all(&request.paths.active).unwrap();

    let result = manager.install(request).await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaUpdateRecoveryRequired));
    let status = manager.status().await;
    assert_eq!(status.bundle, BundleState::RecoveryRequired);
    assert_eq!(status.operation, OperationState::Idle);
    assert_eq!(manager.work_diagnostics_for_test().active, 0);
}

#[tokio::test]
async fn cancelled_install_without_mutation_restores_absent_instead_of_recovery_required() {
    let (_coordinator, manager) = manager();
    let root = tempfile::tempdir().unwrap();
    let request = InstallRequest::for_test(root.path().to_path_buf());
    request.cancellation.cancel();

    assert_eq!(
        manager.install(request).await,
        Err(OllamaErrorCode::OllamaOperationCancelled)
    );
    let status = manager.status().await;
    assert_eq!(status.bundle, BundleState::Absent);
    assert_eq!(status.operation, OperationState::Idle);
    assert_eq!(
        status.last_error,
        Some(OllamaErrorCode::OllamaOperationCancelled)
    );
}

#[tokio::test]
async fn failed_install_without_mutation_reconstructs_absent_and_keeps_retry_usable() {
    let (_coordinator, manager) = manager();
    let root = tempfile::tempdir().unwrap();
    let request = InstallRequest::for_test(root.path().to_path_buf());

    assert_eq!(
        manager.install(request).await,
        Err(OllamaErrorCode::OllamaDownloadFailed)
    );
    let status = manager.status().await;
    assert_eq!(status.bundle, BundleState::Absent);
    assert_eq!(status.operation, OperationState::Idle);
    assert_eq!(
        status.last_error,
        Some(OllamaErrorCode::OllamaDownloadFailed)
    );
    assert_eq!(manager.request_recovery_retry(), Ok(()));
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

#[test]
fn default_update_reap_budget_starts_when_reap_begins() {
    let operation_started = Instant::now();
    std::thread::sleep(Duration::from_millis(20));
    let reap_started = Instant::now();

    let deadline = super::manager::resolve_update_reap_deadline(None);

    assert!(deadline > operation_started + super::constants::PROCESS_REAP_FALLBACK_TIMEOUT);
    assert!(deadline >= reap_started + super::constants::PROCESS_REAP_FALLBACK_TIMEOUT);
}

#[test]
fn explicit_update_deadline_remains_absolute() {
    let requested = Instant::now() + Duration::from_secs(1);

    assert_eq!(
        super::manager::resolve_update_reap_deadline(Some(requested)),
        requested
    );
}
