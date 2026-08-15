use super::error::OllamaErrorCode;
use super::install::InstallRequest;
use super::manager::OllamaManager;
use super::probe::TargetValidation;
use super::types::{BundleState, DaemonState, OllamaEndpoint, OllamaStartOutcome, OperationState};
use super::update::{complete_valid_update, ValidatedJournal};
use super::update_completion_support::{CompletionCutpoint, CompletionHarness};
use crate::app_exit::AppExitCoordinator;
use std::num::NonZeroU16;
use std::time::Duration;

const HISTORICAL_SCENARIO_LINES: [u8; 6] = [12, 15, 16, 17, 21, 22];

fn manager() -> (AppExitCoordinator, OllamaManager) {
    let coordinator = AppExitCoordinator::initialize().expect("exit coordinator");
    let manager = OllamaManager::new(coordinator.work_supervisor());
    (coordinator, manager)
}

async fn wait_for_release(manager: &OllamaManager) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while manager.work_diagnostics_for_test().active != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("manager admission released");
}

#[test]
fn historical_matrix_contains_exactly_the_six_current_lines() {
    let mut sorted = HISTORICAL_SCENARIO_LINES.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted, [12, 15, 16, 17, 21, 22]);
    assert_eq!(HISTORICAL_SCENARIO_LINES.len(), 6);
}

#[tokio::test]
async fn line_12_admitted_setup_is_cancelled_by_bounded_closing() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("setup admission");
    manager.begin_closing();
    assert_eq!(manager.status().await.operation, OperationState::Cancelling);
    drop(operation);
    wait_for_release(&manager).await;
    assert_eq!(manager.status().await.operation, OperationState::Cancelling);
}

#[tokio::test]
async fn line_15_stale_generation_cannot_clean_a_newer_operation() {
    let (_coordinator, manager) = manager();
    let old = manager
        .begin_operation(OperationState::Installing)
        .await
        .expect("old operation");
    let old_generation = old.generation_for_test();
    manager.supersede_generation_for_test(OperationState::Updating);
    drop(old);
    manager.release_generation_for_test(old_generation);
    assert_eq!(manager.status().await.operation, OperationState::Updating);
}

#[tokio::test]
async fn line_16_first_install_cut_keeps_the_previous_destination() {
    let (_coordinator, manager) = manager();
    let root = tempfile::tempdir().expect("temporary root");
    let request = InstallRequest::for_test(root.path().to_path_buf());
    std::fs::create_dir_all(&request.paths.active).expect("active directory");
    let result = manager.install(request.clone()).await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaUpdateRecoveryRequired));
    assert!(request.paths.active.is_dir());
    assert_eq!(manager.status().await.bundle, BundleState::RecoveryRequired);
}

#[tokio::test]
async fn line_17_interrupted_update_keeps_both_bundles_and_models() {
    let harness = CompletionHarness::valid();
    harness.set_pending();
    let models_before = harness.models();
    harness.fail_once(CompletionCutpoint::BackupMoveAfter);
    let journal = ValidatedJournal::from_pending(&harness.pending(), &harness.target).unwrap();
    let result = complete_valid_update(&harness, journal).await.unwrap();
    assert!(matches!(
        result,
        super::update::UpdateOutcome::CleanupPending {
            code: OllamaErrorCode::OllamaUpdateCleanupPending
        }
    ));
    assert_eq!(harness.active(), Some(harness.target.clone()));
    assert_eq!(harness.backup(), None);
    assert_eq!(
        harness.state.lock().unwrap().backup_delete,
        Some(harness.previous.clone())
    );
    assert_eq!(harness.models(), models_before);
    harness.drain().await;
    assert!(harness.journal_state().is_none());
}

#[tokio::test]
async fn line_21_cancellation_is_typed_and_releases_the_single_token() {
    let (_coordinator, manager) = manager();
    let operation = manager
        .begin_operation(OperationState::Updating)
        .await
        .expect("update admission");
    assert_eq!(
        manager.cancel_operation().await,
        super::types::CancelOutcome::Cancelled
    );
    drop(operation);
    wait_for_release(&manager).await;
    assert_eq!(manager.work_diagnostics_for_test().active, 0);
    assert_eq!(
        manager.status().await.last_error,
        Some(OllamaErrorCode::OllamaOperationCancelled)
    );
}

#[test]
fn line_22_distinguishes_owned_spawn_failure_and_external_daemon() {
    let endpoint = OllamaEndpoint::loopback(NonZeroU16::new(11500).unwrap());
    let owned = OllamaStartOutcome::OwnedStarted {
        endpoint: endpoint.clone(),
    };
    let failed = OllamaStartOutcome::Failed {
        code: OllamaErrorCode::OllamaStartFailed,
    };
    let external = OllamaStartOutcome::ExternalAvailable {
        endpoint: endpoint.clone(),
    };
    assert!(matches!(owned, OllamaStartOutcome::OwnedStarted { .. }));
    assert_eq!(
        failed,
        OllamaStartOutcome::Failed {
            code: OllamaErrorCode::OllamaStartFailed,
        }
    );
    assert!(matches!(
        external,
        OllamaStartOutcome::ExternalAvailable { .. }
    ));
    assert_ne!(
        TargetValidation::InvalidTarget {
            code: OllamaErrorCode::OllamaBundleInvalid,
        },
        TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaValidationDeferred,
        }
    );
    assert_ne!(
        DaemonState::External {
            endpoint: endpoint.clone()
        },
        DaemonState::Unavailable
    );
}
