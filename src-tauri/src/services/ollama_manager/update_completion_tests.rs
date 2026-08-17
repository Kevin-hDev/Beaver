use super::error::OllamaErrorCode;
use super::journal::{OllamaJournalState, OllamaTransactionJournal};
use super::update::{
    complete_valid_update, reject_target_and_restore, RejectedJournal, UpdateOutcome,
    ValidatedJournal,
};
use super::update_completion_support::{fingerprint, CompletionHarness};

#[test]
fn validated_journal_rejects_stale_phase_and_fingerprint() {
    let harness = CompletionHarness::valid();
    let journal = harness.pending();
    let validated = ValidatedJournal::from_pending(&journal, &harness.target).unwrap();
    assert_eq!(validated.target(), &harness.target);
    let wrong = fingerprint("9.9.9", "99");
    assert!(ValidatedJournal::from_pending(&journal, &wrong).is_err());
    let stale = OllamaTransactionJournal::new(OllamaJournalState::CleanupPending {
        target: harness.target.clone(),
        previous: harness.previous.clone(),
    });
    assert!(ValidatedJournal::from_pending(&stale, &harness.target).is_err());
}

#[tokio::test]
async fn valid_update_reaches_ready_and_removes_only_old_bundle() {
    let harness = CompletionHarness::valid();
    harness.set_pending();
    let journal = ValidatedJournal::from_pending(&harness.pending(), &harness.target).unwrap();
    let result = complete_valid_update(&harness, journal).await.unwrap();
    assert!(matches!(result, UpdateOutcome::Updated { .. }));
    assert_eq!(harness.active(), Some(harness.target.clone()));
    assert_eq!(harness.backup(), None);
    assert_eq!(harness.journal_state(), None);
    assert_eq!(harness.models(), b"model-store");
}

#[tokio::test]
async fn cleanup_failure_keeps_new_bundle_usable_and_pending() {
    let harness = CompletionHarness::valid();
    harness.set_pending();
    harness.fail_once(super::update_completion_support::CompletionCutpoint::BackupDeleteBefore);
    let journal = ValidatedJournal::from_pending(&harness.pending(), &harness.target).unwrap();
    let result = complete_valid_update(&harness, journal).await.unwrap();
    assert_eq!(
        result,
        UpdateOutcome::CleanupPending {
            code: OllamaErrorCode::OllamaUpdateCleanupPending
        }
    );
    assert_eq!(harness.active(), Some(harness.target.clone()));
    assert!(matches!(
        harness.journal_state(),
        Some(OllamaJournalState::CleanupPending { .. })
    ));
    harness.drain().await;
    assert_eq!(harness.journal_state(), None);
}

#[tokio::test]
async fn rejected_target_restores_previous_and_keeps_models() {
    let harness = CompletionHarness::valid();
    harness.set_pending();
    let models = harness.models();
    let result = reject_target_and_restore(&harness, harness.rejected())
        .await
        .unwrap();
    assert_eq!(
        result,
        UpdateOutcome::Deferred {
            code: OllamaErrorCode::OllamaBundleInvalid
        }
    );
    assert_eq!(harness.active(), Some(harness.previous.clone()));
    assert_eq!(harness.failed(), None);
    assert_eq!(harness.journal_state(), None);
    assert_eq!(harness.models(), models);
}

#[test]
fn rejection_never_writes_an_empty_rejected_target() {
    let harness = CompletionHarness::valid();
    let rejected = RejectedJournal::from_pending(
        &harness.pending(),
        &harness.target,
        OllamaErrorCode::OllamaBundleInvalid,
    )
    .unwrap();
    assert_eq!(rejected.rejected_target(), &harness.target);
}

pub(crate) async fn success_cutpoint(
    cutpoint: super::update_completion_support::CompletionCutpoint,
) {
    let harness = CompletionHarness::valid();
    harness.set_pending();
    harness.fail_once(cutpoint);
    let journal = ValidatedJournal::from_pending(&harness.pending(), &harness.target).unwrap();
    let _ = complete_valid_update(&harness, journal).await;
    harness.drain().await;
    assert_eq!(harness.active(), Some(harness.target.clone()));
    assert_eq!(harness.journal_state(), None);
}

pub(crate) async fn rejection_cutpoint(
    cutpoint: super::update_completion_support::CompletionCutpoint,
) {
    let harness = CompletionHarness::valid();
    harness.set_pending();
    harness.fail_once(cutpoint);
    let _ = reject_target_and_restore(&harness, harness.rejected()).await;
    harness.clear_failure();
    harness.drain().await;
    assert_eq!(harness.active(), Some(harness.previous.clone()));
    assert_eq!(harness.failed(), None);
    assert_eq!(harness.journal_state(), None);
}
