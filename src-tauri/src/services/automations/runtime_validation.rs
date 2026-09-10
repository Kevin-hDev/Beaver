use super::runtime_store::{
    AutomationRuntime, OccurrenceResult, OccurrenceResultStatus, OccurrenceState,
    AUTOMATION_RUNTIME_SCHEMA_VERSION,
};
use std::collections::HashSet;

pub(super) fn validate(runtime: &AutomationRuntime) -> Result<(), String> {
    let unique = runtime
        .occurrences
        .iter()
        .map(|item| item.id)
        .collect::<HashSet<_>>();
    if runtime.schema_version != AUTOMATION_RUNTIME_SCHEMA_VERSION
        || runtime.occurrences.len() > 128
        || unique.len() != runtime.occurrences.len()
    {
        return Err(error());
    }
    let mut pending = HashSet::new();
    let mut running = HashSet::new();
    for item in &runtime.occurrences {
        match item.state {
            OccurrenceState::Pending
                if item.coalesced_count.unwrap_or(0) > 0
                    && item.last_scheduled_for >= Some(item.scheduled_for)
                    && item.started_at.is_none()
                    && item.result.is_none()
                    && pending.insert(item.automation_id) => {}
            OccurrenceState::Running
                if item.coalesced_count.is_none()
                    && item.last_scheduled_for.is_none()
                    && item.started_at.is_some()
                    && item.result.is_none()
                    && running.insert(item.automation_id) => {}
            OccurrenceState::Terminal
                if item.coalesced_count.is_none()
                    && item.last_scheduled_for.is_none()
                    && valid_terminal_result(item.result.as_ref()) => {}
            _ => return Err(error()),
        }
    }
    Ok(())
}

fn valid_terminal_result(result: Option<&OccurrenceResult>) -> bool {
    let Some(result) = result else {
        return false;
    };
    match result.status {
        OccurrenceResultStatus::Missed => {
            result.missed_count.unwrap_or(0) > 0
                && result.first_scheduled_for.is_some()
                && result.last_scheduled_for >= result.first_scheduled_for
        }
        _ => {
            result.missed_count.is_none()
                && result.first_scheduled_for.is_none()
                && result.last_scheduled_for.is_none()
        }
    }
}

fn error() -> String {
    "AUTOMATION_RUNTIME_UNAVAILABLE".into()
}
