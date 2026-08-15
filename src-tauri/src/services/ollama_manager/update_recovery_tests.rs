use super::super::error::OllamaErrorCode;
use super::super::journal::OllamaJournalState;
use super::super::probe::TargetValidation;
use super::super::recovery_decision::RecoveryDecision;
use super::super::update::{execute, UpdateOutcome, UpdateRequest, UpdateSidecar};
use super::support::{FailurePoint, FakeBackend};
use std::path::Path;

fn request(root: &Path) -> UpdateRequest {
    UpdateRequest::for_test(root.to_path_buf())
}

#[tokio::test]
async fn cleanup_pending_refuses_without_prepare_or_mutation() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    let target = backend.target.lock().unwrap().fingerprint.clone();
    backend.set_journal(OllamaJournalState::CleanupPending {
        target,
        previous: backend.previous.clone(),
    });
    let result = execute(&backend, &request(root.path())).await.unwrap();
    assert_eq!(
        result,
        UpdateOutcome::CleanupPending {
            code: OllamaErrorCode::OllamaUpdateCleanupPending
        }
    );
    assert!(backend.events().is_empty());
}

#[tokio::test]
async fn durable_metadata_failure_leaves_active_and_no_prepared_journal() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    backend.fail_at(FailurePoint::Prepare);
    assert_eq!(
        execute(&backend, &request(root.path())).await,
        Err(OllamaErrorCode::OllamaBundleInvalid)
    );
    assert!(backend.journal_phase().is_none());
    assert_eq!(backend.events(), vec!["prepare_target"]);
}

#[tokio::test]
async fn version_and_receipt_cutpoints_leave_staging_non_authoritative() {
    let points = [
        FailurePoint::VersionBefore,
        FailurePoint::VersionAfter,
        FailurePoint::ReceiptBefore,
        FailurePoint::ReceiptAfter,
    ];
    for point in points {
        let root = tempfile::tempdir().unwrap();
        let backend = FakeBackend::new(root.path(), "2.0.0");
        backend.fail_at(point);
        let result = execute(&backend, &request(root.path())).await;
        assert_eq!(result, Err(OllamaErrorCode::OllamaStorageUnavailable));
        assert!(backend.journal_phase().is_none());
        assert!(!backend.staging_authoritative());
        assert!(!backend
            .events()
            .iter()
            .any(|event| event.starts_with("rename")));
        let expected_metadata_events = match point {
            FailurePoint::VersionBefore => 0,
            FailurePoint::VersionAfter | FailurePoint::ReceiptBefore => 1,
            FailurePoint::ReceiptAfter => 2,
            _ => unreachable!("metadata cutpoint expected"),
        };
        assert_eq!(backend.metadata_events().len(), expected_metadata_events);
    }
}

#[tokio::test]
async fn invalid_target_records_rejected_target_and_original_previous() {
    let root = tempfile::tempdir().unwrap();
    let backend = FakeBackend::new(root.path(), "2.0.0");
    let target = backend.target.lock().unwrap().fingerprint.clone();
    backend.set_probe(TargetValidation::InvalidTarget {
        code: OllamaErrorCode::OllamaBundleInvalid,
    });
    let mut update = request(root.path());
    update.sidecar = UpdateSidecar::Owned(std::sync::Arc::new(super::order_tests::TestSidecar));
    let result = execute(&backend, &update).await.unwrap();
    assert_eq!(
        result,
        UpdateOutcome::Deferred {
            code: OllamaErrorCode::OllamaBundleInvalid
        }
    );
    assert!(matches!(
        backend.journal_phase(),
        Some(OllamaJournalState::RollbackPending {
            previous,
            rejected_target: Some(rejected)
        }) if previous == backend.previous && rejected == target
    ));
}

#[tokio::test]
async fn every_journal_cutpoint_leaves_a_single_recovery_snapshot() {
    let points = [
        FailurePoint::PreparedBefore,
        FailurePoint::PreparedAfter,
        FailurePoint::PendingBefore,
        FailurePoint::PendingAfter,
        FailurePoint::CleanupBefore,
        FailurePoint::CleanupAfter,
        FailurePoint::RollbackBefore,
        FailurePoint::RollbackAfter,
    ];
    for point in points {
        let root = tempfile::tempdir().unwrap();
        let backend = FakeBackend::new(root.path(), "2.0.0");
        backend.fail_at(point);
        if matches!(
            point,
            FailurePoint::RollbackBefore | FailurePoint::RollbackAfter
        ) {
            backend.set_probe(TargetValidation::InvalidTarget {
                code: OllamaErrorCode::OllamaBundleInvalid,
            });
        }
        let _ = execute(&backend, &request(root.path())).await;
        let snapshot = backend.journal_phase();
        assert!(snapshot.is_some() || matches!(point, FailurePoint::PreparedBefore));
        if let Some(state) = snapshot {
            assert!(matches!(
                state,
                OllamaJournalState::Prepared { .. }
                    | OllamaJournalState::PendingValidation { .. }
                    | OllamaJournalState::CleanupPending { .. }
                    | OllamaJournalState::RollbackPending { .. }
            ));
        }
        assert!(!matches!(
            backend.recovery_decision(),
            RecoveryDecision::Defer { .. }
        ));
    }
}

#[tokio::test]
async fn rename_and_parent_sync_cutpoints_never_publish_success() {
    let points = [
        FailurePoint::ActiveRenameBefore,
        FailurePoint::ActiveRenameAfter,
        FailurePoint::ActiveSyncBefore,
        FailurePoint::ActiveSyncAfter,
        FailurePoint::TargetRenameBefore,
        FailurePoint::TargetRenameAfter,
        FailurePoint::TargetSyncBefore,
        FailurePoint::TargetSyncAfter,
    ];
    for point in points {
        let root = tempfile::tempdir().unwrap();
        let backend = FakeBackend::new(root.path(), "2.0.0");
        backend.fail_at(point);
        let result = execute(&backend, &request(root.path())).await;
        assert!(result.is_err(), "cutpoint {point:?} unexpectedly succeeded");
        assert!(!matches!(result, Ok(UpdateOutcome::Updated { .. })));
    }
}
