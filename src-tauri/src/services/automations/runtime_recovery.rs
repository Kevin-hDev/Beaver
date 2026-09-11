use super::runtime_store::{
    read_unlocked_at, write_unlocked_at, AutomationOccurrence, AutomationRuntime, OccurrenceResult,
    OccurrenceResultStatus, OccurrenceState, AUTOMATION_RUNTIME_SCHEMA_VERSION,
};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

pub async fn recover_startup(now: DateTime<Utc>) -> Result<Vec<Uuid>, String> {
    recover_startup_at(&crate::services::paths::data_dir(), now).await
}

pub(crate) async fn recover_startup_at(
    root: &Path,
    now: DateTime<Utc>,
) -> Result<Vec<Uuid>, String> {
    let _guard = super::store_lock().await;
    let Some(mut runtime) = read_unlocked_at(root).await? else {
        write_unlocked_at(
            root,
            &AutomationRuntime {
                schema_version: AUTOMATION_RUNTIME_SCHEMA_VERSION,
                last_checked_at: now,
                occurrences: Vec::new(),
                retired_automation_ids: Vec::new(),
            },
        )
        .await?;
        return Ok(Vec::new());
    };
    for occurrence in &mut runtime.occurrences {
        match occurrence.state {
            OccurrenceState::Pending => terminalize_pending(occurrence, now),
            OccurrenceState::Running => terminalize_running(occurrence, now),
            OccurrenceState::Terminal => {}
        }
    }
    let ids = runtime
        .occurrences
        .iter()
        .filter(|item| item.state == OccurrenceState::Terminal)
        .map(|item| item.id)
        .collect();
    write_unlocked_at(root, &runtime).await?;
    Ok(ids)
}

fn terminalize_pending(item: &mut AutomationOccurrence, now: DateTime<Utc>) {
    item.state = OccurrenceState::Terminal;
    item.updated_at = now;
    item.started_at = None;
    item.result = Some(OccurrenceResult {
        status: OccurrenceResultStatus::Missed,
        finished_at: now,
        error_code: None,
        session_id: None,
        tokens: None,
        missed_count: item.coalesced_count,
        first_scheduled_for: Some(item.scheduled_for),
        last_scheduled_for: item.last_scheduled_for,
    });
    item.coalesced_count = None;
    item.last_scheduled_for = None;
}

fn terminalize_running(item: &mut AutomationOccurrence, now: DateTime<Utc>) {
    item.state = OccurrenceState::Terminal;
    item.updated_at = now;
    item.result = Some(OccurrenceResult {
        status: OccurrenceResultStatus::Interrupted,
        finished_at: now,
        error_code: Some("app_stopped".into()),
        session_id: None,
        tokens: None,
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    });
}
