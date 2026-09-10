use super::{AutomationError, HistoryEntry};
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub(super) struct Cursor {
    run_id: Option<Uuid>,
    automation_id: String,
    scheduled_for: String,
}

pub(super) fn encode(entry: &HistoryEntry) -> Result<String, AutomationError> {
    let cursor = Cursor {
        run_id: entry.run_id,
        automation_id: entry.automation_id.clone(),
        scheduled_for: entry.scheduled_for.clone(),
    };
    let bytes = serde_json::to_vec(&cursor).map_err(|_| AutomationError::CursorExpired)?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

pub(super) fn decode(raw: &str) -> Result<Cursor, AutomationError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw)
        .map_err(|_| AutomationError::CursorExpired)?;
    serde_json::from_slice(&bytes).map_err(|_| AutomationError::CursorExpired)
}

pub(super) fn matches(entry: &HistoryEntry, cursor: &Cursor) -> bool {
    entry.run_id == cursor.run_id
        && entry.automation_id == cursor.automation_id
        && entry.scheduled_for == cursor.scheduled_for
}
