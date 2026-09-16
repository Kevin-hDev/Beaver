use crate::services::automations::{OccurrenceResult, OccurrenceResultStatus};
use chrono::Utc;
use uuid::Uuid;

pub(super) async fn finish_error(
    occurrence_id: Uuid,
    automation_id: Uuid,
    event_session_id: &str,
    request_id: &str,
    code: &str,
    result_session_id: Option<String>,
) {
    finish(
        occurrence_id,
        automation_id,
        event_session_id,
        request_id,
        error_result(code, result_session_id),
    )
    .await;
}

pub(super) fn error_result(code: &str, session_id: Option<String>) -> OccurrenceResult {
    result(OccurrenceResultStatus::Error, code, session_id)
}

pub(super) fn cancelled_result(session_id: Option<String>) -> OccurrenceResult {
    result(OccurrenceResultStatus::Cancelled, "cancelled", session_id)
}

fn result(
    status: OccurrenceResultStatus,
    code: &str,
    session_id: Option<String>,
) -> OccurrenceResult {
    OccurrenceResult {
        status,
        finished_at: Utc::now(),
        error_code: Some(code.into()),
        session_id,
        tokens: None,
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
}

pub(super) async fn finish(
    occurrence_id: Uuid,
    automation_id: Uuid,
    session_id: &str,
    request_id: &str,
    result: OccurrenceResult,
) {
    let status = match result.status {
        OccurrenceResultStatus::Ok => "completed",
        OccurrenceResultStatus::Cancelled => "cancelled",
        OccurrenceResultStatus::Interrupted => "interrupted",
        OccurrenceResultStatus::Error | OccurrenceResultStatus::Missed => "failed",
    };
    let error_code = result.error_code.clone();
    if super::runtime::mark_terminal(occurrence_id, result)
        .await
        .is_ok()
    {
        let _ = crate::services::extensions::automation_event(
            session_id,
            request_id,
            &automation_id.to_string(),
            false,
            status,
            error_code.as_deref(),
        );
        if super::runtime::publish_terminal(occurrence_id)
            .await
            .is_err()
        {
            ::log::warn!("[scheduler] publication terminale différée");
        }
    }
}

pub(super) async fn reject_before_start(occurrence_id: Uuid, automation_id: Uuid, code: &str) {
    let _ = super::runtime::mark_terminal(occurrence_id, error_result(code, None)).await;
    if disables_extension_definition(code) {
        let _ = crate::services::automations::disable_extension_unavailable(automation_id).await;
    }
    if super::runtime::publish_terminal(occurrence_id)
        .await
        .is_err()
    {
        ::log::warn!("[scheduler] publication terminale différée");
    }
}

pub(super) fn disables_extension_definition(code: &str) -> bool {
    code == "extension_unavailable"
}

pub(super) fn error_code(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.contains("auth") || lower.contains("401") || lower.contains("403") {
        "authentication_failed"
    } else if lower.contains("model") {
        "model_unavailable"
    } else if lower.contains("provider") || lower.contains("ollama") {
        "provider_unavailable"
    } else {
        "failed"
    }
}
