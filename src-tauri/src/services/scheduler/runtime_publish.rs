use crate::models::{WakeupRunErrorCode, WakeupRunStatus};
use crate::services::automations::{
    AutomationError, AutomationOccurrence, HistoryEntry, OccurrenceResult, OccurrenceResultStatus,
};
use std::path::Path;
use uuid::Uuid;

pub(crate) async fn publish_terminal_at(root: &Path, id: Uuid) -> Result<(), AutomationError> {
    let occurrence = crate::services::automations::read_terminal_at(root, id).await?;
    let result = occurrence
        .result
        .as_ref()
        .ok_or(AutomationError::StoreUnavailable)?;
    crate::services::automations::append_history_at(
        &root.join("logs/wakeups.jsonl"),
        history_entry(&occurrence, result),
    )
    .await
    .map_err(|_| AutomationError::StoreUnavailable)?;
    crate::services::automations::record_completion_at(
        root,
        occurrence.automation_id,
        result.finished_at,
    )
    .await?;
    crate::services::automations::remove_runtime_terminal_at(root, id).await?;
    super::notify_config_changed();
    Ok(())
}

fn history_entry(occurrence: &AutomationOccurrence, result: &OccurrenceResult) -> HistoryEntry {
    HistoryEntry {
        run_id: Some(occurrence.id),
        automation_id: occurrence.automation_id.to_string(),
        scheduled_for: occurrence.scheduled_for.to_rfc3339(),
        finished_at: result.finished_at.to_rfc3339(),
        status: status(&result.status),
        error_code: result.error_code.as_deref().map(error_code),
        session_id: result.session_id.clone(),
        tokens: result.tokens,
        missed_count: result.missed_count,
        first_scheduled_for: result.first_scheduled_for.map(|at| at.to_rfc3339()),
        last_scheduled_for: result.last_scheduled_for.map(|at| at.to_rfc3339()),
    }
}

fn status(status: &OccurrenceResultStatus) -> WakeupRunStatus {
    match status {
        OccurrenceResultStatus::Ok => WakeupRunStatus::Ok,
        OccurrenceResultStatus::Error => WakeupRunStatus::Error,
        OccurrenceResultStatus::Missed => WakeupRunStatus::Missed,
        OccurrenceResultStatus::Cancelled => WakeupRunStatus::Cancelled,
        OccurrenceResultStatus::Interrupted => WakeupRunStatus::Interrupted,
    }
}

fn error_code(code: &str) -> WakeupRunErrorCode {
    match code {
        "app_stopped" => WakeupRunErrorCode::AppStopped,
        "target_session_missing" => WakeupRunErrorCode::TargetSessionMissing,
        "provider_unavailable" => WakeupRunErrorCode::ProviderUnavailable,
        "model_unavailable" => WakeupRunErrorCode::ModelUnavailable,
        "authentication_failed" => WakeupRunErrorCode::AuthenticationFailed,
        "capacity_reached" => WakeupRunErrorCode::CapacityReached,
        _ => WakeupRunErrorCode::Failed,
    }
}
