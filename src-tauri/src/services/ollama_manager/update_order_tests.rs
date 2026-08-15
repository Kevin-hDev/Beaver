use super::super::error::OllamaErrorCode;
use super::super::journal::OllamaJournalState;
use super::super::probe::TargetValidation;
use super::super::update::{
    execute, OwnedSidecarController, UpdateOutcome, UpdateRequest, UpdateSidecar,
};
use super::support::{FailurePoint, FakeBackend};
use std::path::Path;
use std::sync::Arc;

fn request(root: &Path) -> UpdateRequest {
    UpdateRequest::for_test(root.to_path_buf())
}

fn owned_request(root: &Path) -> UpdateRequest {
    let mut value = request(root);
    value.sidecar = UpdateSidecar::Owned(Arc::new(TestSidecar));
    value
}

pub(super) struct TestSidecar;

impl OwnedSidecarController for TestSidecar {
    fn stop(&self) -> Result<(), OllamaErrorCode> {
        Ok(())
    }

    fn reap(&self) -> Result<(), OllamaErrorCode> {
        Ok(())
    }
}

#[tokio::test]
async fn order_persists_prepared_before_owned_stop_and_reap() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    let result = execute(&backend, &owned_request(root.path())).await;
    assert!(matches!(result, Ok(UpdateOutcome::CleanupPending { .. })));
    assert_eq!(
        backend.events(),
        vec![
            "prepare_target",
            "persist_prepared",
            "stop_owned_sidecar",
            "reap_owned_sidecar",
            "rename_active_to_backup",
            "sync_parent_active_backup",
            "rename_target_to_active",
            "sync_parent_target_active",
            "persist_pending_validation",
            "probe_active",
            "persist_cleanup_pending",
        ]
    );
}

#[tokio::test]
async fn stop_failure_forbids_every_rename() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    backend.fail_at(FailurePoint::Stop);
    assert_eq!(
        execute(&backend, &owned_request(root.path())).await,
        Err(OllamaErrorCode::OllamaStopFailed)
    );
    assert!(!backend
        .events()
        .iter()
        .any(|event| event.starts_with("rename")));
    assert!(matches!(
        backend.journal_phase(),
        Some(OllamaJournalState::Prepared { .. })
    ));
}

#[tokio::test]
async fn reap_failure_forbids_every_rename() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    backend.fail_at(FailurePoint::Reap);
    assert_eq!(
        execute(&backend, &owned_request(root.path())).await,
        Err(OllamaErrorCode::OllamaStopFailed)
    );
    assert!(!backend
        .events()
        .iter()
        .any(|event| event.starts_with("rename")));
}

#[tokio::test]
async fn external_daemon_is_not_stopped_reaped_or_used_as_validation() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    let mut update = request(root.path());
    update.sidecar = UpdateSidecar::External;
    let result = execute(&backend, &update).await.unwrap();
    assert!(matches!(result, UpdateOutcome::Deferred { .. }));
    assert!(backend.events().is_empty());
    assert!(backend.journal_phase().is_none());
    assert!(!backend.staging_authoritative());
}

#[tokio::test]
async fn already_current_does_not_create_a_transaction() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    backend.make_current();
    assert_eq!(
        execute(&backend, &request(root.path())).await,
        Ok(UpdateOutcome::AlreadyCurrent)
    );
    assert_eq!(backend.events(), vec!["prepare_target"]);
    assert!(backend.journal_phase().is_none());
}

#[tokio::test]
async fn deferred_probe_keeps_pending_validation() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    backend.set_probe(TargetValidation::Deferred {
        code: OllamaErrorCode::OllamaValidationDeferred,
    });
    let result = execute(&backend, &owned_request(root.path()))
        .await
        .unwrap();
    assert_eq!(
        result,
        UpdateOutcome::Deferred {
            code: OllamaErrorCode::OllamaValidationDeferred
        }
    );
    assert!(matches!(
        backend.journal_phase(),
        Some(OllamaJournalState::PendingValidation { .. })
    ));
}
