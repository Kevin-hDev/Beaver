use crate::models::WakeupRun;
use crate::services::automations::HistoryEntry;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

#[cfg(test)]
pub(super) const MAX_LINES: usize = crate::services::automations::history_max_lines();
#[cfg(test)]
pub(super) const ROTATED_LINES: usize = MAX_LINES / 2;
pub(super) const MAX_ID_CHARS: usize = 128;
#[cfg(test)]
pub(super) const MAX_LOG_LINE_BYTES: usize = 2_048;

pub(super) async fn append_at(path: &Path, entry: WakeupRun) -> Result<(), String> {
    crate::services::automations::append_history_at(path, into_history(entry)).await
}

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

#[cfg(test)]
pub(super) fn parse_runs(content: &str, wakeup_id: Option<&str>) -> Vec<WakeupRun> {
    crate::services::automations::parse_history(content)
        .into_iter()
        .filter(|entry| {
            wakeup_id
                .map(|id| entry.automation_id == id)
                .unwrap_or(true)
        })
        .map(into_legacy)
        .collect()
}

#[cfg(test)]
pub(super) async fn append_at_with_atomic_writer<Writer, Future>(
    path: &Path,
    entry: WakeupRun,
    writer: Writer,
) -> Result<(), String>
where
    Writer: FnOnce(PathBuf, Vec<u8>) -> Future,
    Future: std::future::Future<Output = Result<(), String>>,
{
    crate::services::automations::append_history_at_with_atomic_writer(
        path,
        into_history(entry),
        writer,
    )
    .await
}

#[cfg(test)]
pub(super) async fn append_at_with_read_observer<Observer>(
    path: &Path,
    entry: WakeupRun,
    observer: Observer,
) -> Result<(), String>
where
    Observer: FnMut(),
{
    crate::services::automations::append_history_at_with_read_observer(
        path,
        into_history(entry),
        observer,
    )
    .await
}

fn into_history(entry: WakeupRun) -> HistoryEntry {
    HistoryEntry {
        run_id: None,
        automation_id: entry.wakeup_id,
        scheduled_for: entry.scheduled_for,
        finished_at: entry.fired_at,
        status: entry.status,
        error_code: entry.error_code,
        session_id: entry.session_id,
        tokens: entry.tokens,
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
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
