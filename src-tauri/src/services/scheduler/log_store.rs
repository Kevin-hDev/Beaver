use crate::models::WakeupRun;
use crate::services::automations::HistoryEntry;
use std::path::Path;

pub(super) async fn list_runs_at(
    path: &Path,
    wakeup_id: Option<&str>,
) -> Result<Vec<WakeupRun>, String> {
    Ok(
        crate::services::automations::all_history_at(path, wakeup_id)
            .await?
            .into_iter()
            .map(into_legacy)
            .collect(),
    )
}

fn into_legacy(entry: HistoryEntry) -> WakeupRun {
    WakeupRun {
        wakeup_id: entry.automation_id,
        scheduled_for: entry.scheduled_for,
        fired_at: entry.finished_at,
        status: entry.status,
        error_code: entry.error_code,
        _legacy_error: None,
        session_id: entry.session_id,
        tokens: entry.tokens,
    }
}
