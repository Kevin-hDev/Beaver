#[path = "log_store.rs"]
mod store;

use crate::models::{WakeupRun, WakeupRunErrorCode, WakeupRunStatus};
use crate::services::work_registry::ServiceWorkAdmissionError;
use chrono::{DateTime, Local, Utc};
use std::path::PathBuf;
use store::{append_at, list_runs_at};
#[cfg(test)]
use store::{
    append_at_with_atomic_writer, parse_runs, MAX_ID_CHARS, MAX_LINES, MAX_LOG_LINE_BYTES,
};

fn log_path() -> PathBuf {
    crate::services::paths::data_dir()
        .join("logs")
        .join("wakeups.jsonl")
}

pub async fn log_ok(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    session_id: &str,
    tokens: u32,
) -> Result<(), String> {
    append(WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Ok,
        error_code: None,
        _legacy_error: None,
        session_id: Some(safe_id(session_id)),
        tokens: Some(tokens),
    })
    .await
}

pub async fn log_err(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    error: &str,
) -> Result<(), String> {
    append(error_entry(
        wakeup_id,
        scheduled_for,
        generic_error_code(error),
    ))
    .await
}

pub async fn log_refused(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    error: ServiceWorkAdmissionError,
) -> Result<(), String> {
    append(error_entry(
        wakeup_id,
        scheduled_for,
        refusal_error_code(error),
    ))
    .await
}

pub async fn log_missed(wakeup_id: &str, scheduled_for: DateTime<Local>) -> Result<(), String> {
    append(missed_entry(wakeup_id, scheduled_for)).await
}

fn missed_entry(wakeup_id: &str, scheduled_for: DateTime<Local>) -> WakeupRun {
    WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Missed,
        error_code: Some(WakeupRunErrorCode::MissedUnavailable),
        _legacy_error: None,
        session_id: None,
        tokens: None,
    }
}

pub async fn log_cancelled(wakeup_id: &str, scheduled_for: DateTime<Local>) -> Result<(), String> {
    append(cancelled_entry(wakeup_id, scheduled_for)).await
}

pub async fn list_runs(wakeup_id: Option<&str>) -> Result<Vec<WakeupRun>, String> {
    list_runs_at(&log_path(), wakeup_id).await
}

fn cancelled_entry(wakeup_id: &str, scheduled_for: DateTime<Local>) -> WakeupRun {
    WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Cancelled,
        error_code: None,
        _legacy_error: None,
        session_id: None,
        tokens: None,
    }
}

fn error_entry(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    code: WakeupRunErrorCode,
) -> WakeupRun {
    WakeupRun {
        wakeup_id: safe_id(wakeup_id),
        scheduled_for: scheduled_for.to_rfc3339(),
        fired_at: Utc::now().to_rfc3339(),
        status: WakeupRunStatus::Error,
        error_code: Some(code),
        _legacy_error: None,
        session_id: None,
        tokens: None,
    }
}

fn generic_error_code(error: &str) -> WakeupRunErrorCode {
    let lower = error.to_lowercase();
    if lower.contains("rate limit") {
        WakeupRunErrorCode::RateLimited
    } else if lower.contains("clé api") || lower.contains("unauthorized") || lower.contains("auth")
    {
        WakeupRunErrorCode::AuthenticationFailed
    } else if lower.contains("ollama") {
        WakeupRunErrorCode::OllamaUnavailable
    } else {
        WakeupRunErrorCode::Failed
    }
}

fn refusal_error_code(error: ServiceWorkAdmissionError) -> WakeupRunErrorCode {
    match error {
        ServiceWorkAdmissionError::AppClosing | ServiceWorkAdmissionError::Closing => {
            WakeupRunErrorCode::SchedulerStopping
        }
        ServiceWorkAdmissionError::AppCapacity | ServiceWorkAdmissionError::Capacity => {
            WakeupRunErrorCode::CapacityReached
        }
    }
}

fn safe_id(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(store::MAX_ID_CHARS)
        .collect()
}

async fn append(entry: WakeupRun) -> Result<(), String> {
    append_at(&log_path(), entry).await
}

#[cfg(test)]
pub(super) fn cancelled_entry_for_test(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
) -> WakeupRun {
    cancelled_entry(wakeup_id, scheduled_for)
}

#[cfg(test)]
pub(super) fn error_entry_for_test(
    wakeup_id: &str,
    scheduled_for: DateTime<Local>,
    error: &str,
) -> WakeupRun {
    error_entry(wakeup_id, scheduled_for, generic_error_code(error))
}

#[cfg(test)]
#[path = "log_tests.rs"]
mod tests;
