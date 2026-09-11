use super::runtime_store::AUTOMATION_RUNTIME_SCHEMA_VERSION;
use super::{AutomationError, AutomationOccurrence};
use super::{AutomationRuntime, OccurrenceResult, OccurrenceState};
#[cfg(test)]
use crate::models::AutomationDefinition;
#[cfg(test)]
use chrono::Duration;
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

#[cfg(test)]
const MAX_OCCURRENCES: usize = 128;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAdmission {
    Ready { occurrence_id: Uuid },
    Pending { occurrence_id: Uuid },
    Coalesced { occurrence_id: Uuid },
    Missed { occurrence_id: Uuid },
}

#[cfg(test)]
pub async fn admit_at(
    root: &Path,
    definition: &AutomationDefinition,
    scheduled_for: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<RuntimeAdmission, AutomationError> {
    if definition.status != crate::models::AutomationStatus::Active || scheduled_for > now {
        return Err(AutomationError::InvalidInput);
    }
    let _guard = super::store_lock().await;
    let mut runtime = load(root, now).await?;
    let admission = admit_into(&mut runtime, definition.id, scheduled_for, now)?;
    super::runtime_store::write_unlocked_at(root, &runtime)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    Ok(admission)
}

#[cfg(test)]
fn admit_into(
    runtime: &mut AutomationRuntime,
    automation_id: Uuid,
    scheduled_for: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<RuntimeAdmission, AutomationError> {
    if let Some(index) = pending_position(runtime, automation_id) {
        return merge_at(runtime, index, scheduled_for, now);
    }
    add_occurrence(runtime, automation_id, scheduled_for, now)
}

#[cfg(test)]
fn pending_position(runtime: &AutomationRuntime, automation_id: Uuid) -> Option<usize> {
    runtime.occurrences.iter().position(|item| {
        item.automation_id == automation_id && item.state == OccurrenceState::Pending
    })
}

#[cfg(test)]
fn merge_at(
    runtime: &mut AutomationRuntime,
    index: usize,
    scheduled_for: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<RuntimeAdmission, AutomationError> {
    let occurrence = &mut runtime.occurrences[index];
    let count = occurrence
        .coalesced_count
        .unwrap_or(0)
        .checked_add(1)
        .ok_or(AutomationError::CapacityReached)?;
    occurrence.coalesced_count = Some(count);
    occurrence.last_scheduled_for = Some(
        occurrence
            .last_scheduled_for
            .unwrap_or(occurrence.scheduled_for)
            .max(scheduled_for),
    );
    occurrence.updated_at = now;
    Ok(RuntimeAdmission::Coalesced {
        occurrence_id: occurrence.id,
    })
}

#[cfg(test)]
fn add_occurrence(
    runtime: &mut AutomationRuntime,
    automation_id: Uuid,
    scheduled_for: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<RuntimeAdmission, AutomationError> {
    if runtime.occurrences.len() >= MAX_OCCURRENCES {
        return Err(AutomationError::CapacityReached);
    }
    let has_running = runtime
        .occurrences
        .iter()
        .any(|item| item.automation_id == automation_id && item.state == OccurrenceState::Running);
    let occurrence = if !has_running
        && now - scheduled_for > Duration::minutes(super::next_fire::MISSED_GRACE_MINUTES)
    {
        AutomationOccurrence::missed(automation_id, scheduled_for, scheduled_for, 1, now)
    } else {
        let mut pending = AutomationOccurrence::pending(automation_id, scheduled_for);
        pending.created_at = now;
        pending.updated_at = now;
        pending
    };
    let id = occurrence.id;
    let admission = match occurrence.state {
        OccurrenceState::Terminal => RuntimeAdmission::Missed { occurrence_id: id },
        OccurrenceState::Pending if has_running => RuntimeAdmission::Pending { occurrence_id: id },
        OccurrenceState::Pending => RuntimeAdmission::Ready { occurrence_id: id },
        OccurrenceState::Running => return Err(AutomationError::StoreUnavailable),
    };
    runtime.occurrences.push(occurrence);
    Ok(admission)
}

async fn load(root: &Path, now: DateTime<Utc>) -> Result<AutomationRuntime, AutomationError> {
    Ok(super::runtime_store::read_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .unwrap_or(AutomationRuntime {
            schema_version: AUTOMATION_RUNTIME_SCHEMA_VERSION,
            last_checked_at: now,
            occurrences: Vec::new(),
            retired_automation_ids: Vec::new(),
        }))
}

pub async fn mark_running_at(
    root: &Path,
    id: Uuid,
    started_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut runtime = load(root, started_at).await?;
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
    let mut runtime = load(root, result.finished_at).await?;
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
    let mut runtime = load(root, Utc::now()).await?;
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
