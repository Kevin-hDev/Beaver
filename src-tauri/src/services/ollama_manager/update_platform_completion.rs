use super::super::super::error::OllamaErrorCode;
use super::super::CompletionRecovery;

pub(super) async fn recover() -> Result<CompletionRecovery, OllamaErrorCode> {
    match super::super::super::recovery_entry::recover_platform(
        super::super::super::recovery::RecoveryReason::Manual,
    )
    .await?
    {
        super::super::super::recovery::RecoveryOutcome::Ready => Ok(CompletionRecovery::Ready),
        super::super::super::recovery::RecoveryOutcome::ProgressMade => {
            Ok(CompletionRecovery::Progress)
        }
        super::super::super::recovery::RecoveryOutcome::Deferred { code } => {
            Ok(CompletionRecovery::Deferred { code })
        }
    }
}
