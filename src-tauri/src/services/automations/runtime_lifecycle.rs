use super::{AutomationError, AutomationOccurrence};
use super::{AutomationRuntime, OccurrenceResult, OccurrenceState};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

async fn load(root: &Path) -> Result<AutomationRuntime, AutomationError> {
    super::runtime_store::read_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .ok_or(AutomationError::NotFound)
}

pub async fn mark_running_at(
    root: &Path,
    id: Uuid,
    started_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut runtime = load(root).await?;
    let index = runtime
        .occurrences
        .iter()
        .position(|item| item.id == id && item.state == OccurrenceState::Pending)
        .ok_or(AutomationError::NotFound)?;
    let automation_id = runtime.occurrences[index].automation_id;
    let already_running = runtime
        .occurrences
        .iter()
        .any(|item| item.automation_id == automation_id && item.state == OccurrenceState::Running);
    if already_running {
        return Err(AutomationError::InvalidInput);
    }
    let occurrence = &mut runtime.occurrences[index];
    occurrence.state = OccurrenceState::Running;
    occurrence.updated_at = started_at;
    occurrence.coalesced_count = None;
    occurrence.last_scheduled_for = None;
    occurrence.started_at = Some(started_at);
    super::runtime_store::write_unlocked_at(root, &runtime)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}

pub async fn mark_terminal_at(
    root: &Path,
    id: Uuid,
    result: OccurrenceResult,
) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut runtime = load(root).await?;
    let occurrence = runtime
        .occurrences
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or(AutomationError::NotFound)?;
    if occurrence.state == OccurrenceState::Terminal {
        if occurrence.result.as_ref() == Some(&result) {
            return Ok(());
        }
        return Err(AutomationError::InvalidInput);
    }
    occurrence.state = OccurrenceState::Terminal;
    occurrence.updated_at = result.finished_at;
    occurrence.coalesced_count = None;
    occurrence.last_scheduled_for = None;
    occurrence.result = Some(result);
    super::runtime_store::write_unlocked_at(root, &runtime)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}

pub async fn runtime_at(root: &Path) -> Result<AutomationRuntime, AutomationError> {
    let _guard = super::store_lock().await;
    super::runtime_store::read_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .ok_or(AutomationError::NotFound)
}

pub(crate) async fn terminal_unlocked_at(
    root: &Path,
    id: Uuid,
) -> Result<AutomationOccurrence, AutomationError> {
    let runtime = super::runtime_store::read_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .ok_or(AutomationError::NotFound)?;
    runtime
        .occurrences
        .into_iter()
        .find(|item| item.id == id && item.state == OccurrenceState::Terminal)
        .ok_or(AutomationError::NotFound)
}

pub(crate) async fn remove_terminal_unlocked_at(
    root: &Path,
    id: Uuid,
) -> Result<(), AutomationError> {
    let mut runtime = load(root).await?;
    let before = runtime.occurrences.len();
    runtime
        .occurrences
        .retain(|item| item.id != id || item.state != OccurrenceState::Terminal);
    if runtime.occurrences.len() == before {
        return Err(AutomationError::NotFound);
    }
    super::runtime_store::write_unlocked_at(root, &runtime)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}
