use super::update_completion_support::CompletionCutpoint;
use super::update_completion_tests::{rejection_cutpoint, success_cutpoint};

const SUCCESS_CUTPOINTS: &[CompletionCutpoint] = &[
    CompletionCutpoint::BackupMoveBefore,
    CompletionCutpoint::BackupMoveAfter,
    CompletionCutpoint::BackupDeleteBefore,
    CompletionCutpoint::BackupDeleteAfter,
    CompletionCutpoint::JournalRemoveBefore,
    CompletionCutpoint::JournalRemoveAfter,
];

const REJECTION_CUTPOINTS: &[CompletionCutpoint] = &[
    CompletionCutpoint::FailedMoveBefore,
    CompletionCutpoint::FailedMoveAfter,
    CompletionCutpoint::RestoreBefore,
    CompletionCutpoint::RestoreAfter,
    CompletionCutpoint::RollbackJournalBefore,
    CompletionCutpoint::RollbackJournalAfter,
    CompletionCutpoint::FailedDeleteMoveBefore,
    CompletionCutpoint::FailedDeleteMoveAfter,
    CompletionCutpoint::FailedDeleteBefore,
    CompletionCutpoint::FailedDeleteAfter,
    CompletionCutpoint::JournalRemoveBefore,
    CompletionCutpoint::JournalRemoveAfter,
];

#[tokio::test]
async fn every_success_cutpoint_converges_without_losing_active_bundle() {
    for cutpoint in SUCCESS_CUTPOINTS {
        success_cutpoint(*cutpoint).await;
    }
}

#[tokio::test]
async fn every_rejection_cutpoint_converges_with_previous_bundle() {
    for cutpoint in REJECTION_CUTPOINTS {
        rejection_cutpoint(*cutpoint).await;
    }
}
