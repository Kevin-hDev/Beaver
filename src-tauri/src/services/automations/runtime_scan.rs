use super::runtime_store::{scan_and_advance_at, AutomationOccurrence, OccurrenceState};
use crate::models::AutomationDefinition;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn scan_and_advance(
    through: DateTime<Utc>,
    definitions: &[AutomationDefinition],
) -> Result<Vec<Uuid>, String> {
    scan_definitions_at(&crate::services::paths::data_dir(), through, definitions).await
}

pub(crate) async fn scan_definitions_at(
    root: &std::path::Path,
    through: DateTime<Utc>,
    definitions: &[AutomationDefinition],
) -> Result<Vec<Uuid>, String> {
    scan_and_advance_at(root, through, |last_checked| {
        collect(definitions, last_checked, through)
    })
    .await
}

fn collect(
    definitions: &[AutomationDefinition],
    last_checked: DateTime<Utc>,
    through: DateTime<Utc>,
) -> Result<Vec<AutomationOccurrence>, String> {
    let mut occurrences = Vec::new();
    for definition in definitions {
        let due =
            crate::services::scheduler::next_fire::due_between(definition, last_checked, through)
                .map_err(|_| "AUTOMATION_SCHEDULE_INVALID".to_string())?;
        if let Some(range) = due.missed {
            occurrences.push(AutomationOccurrence::missed(
                definition.id,
                range.first,
                range.last,
                u32::try_from(range.count)
                    .map_err(|_| "AUTOMATION_RUNTIME_UNAVAILABLE".to_string())?,
                through,
            ));
        }
        if let Some(range) = due.admissible {
            let mut pending = AutomationOccurrence::pending(definition.id, range.first);
            pending.coalesced_count = Some(
                u32::try_from(range.count)
                    .map_err(|_| "AUTOMATION_RUNTIME_UNAVAILABLE".to_string())?,
            );
            pending.last_scheduled_for = Some(range.last);
            occurrences.push(pending);
        }
    }
    Ok(occurrences)
}

pub(super) fn merge_pending(
    current: &mut AutomationOccurrence,
    additional: &AutomationOccurrence,
) -> Result<(), String> {
    if current.state != OccurrenceState::Pending
        || additional.state != OccurrenceState::Pending
        || current.automation_id != additional.automation_id
    {
        return Err("AUTOMATION_RUNTIME_UNAVAILABLE".into());
    }
    current.coalesced_count =
        Some(current.coalesced_count.unwrap_or(0) + additional.coalesced_count.unwrap_or(0));
    current.last_scheduled_for = additional.last_scheduled_for;
    current.updated_at = additional.updated_at;
    Ok(())
}

pub(super) fn apply(
    current: &mut Vec<AutomationOccurrence>,
    mut additional: AutomationOccurrence,
) -> Result<Uuid, String> {
    let automation_id = additional.automation_id;
    let has_running = current
        .iter()
        .any(|item| item.automation_id == automation_id && item.state == OccurrenceState::Running);
    let has_pending = current
        .iter()
        .any(|item| item.automation_id == automation_id && item.state == OccurrenceState::Pending);
    if additional.state == OccurrenceState::Terminal && (has_running || has_pending) {
        additional = pending_from_missed(additional)?;
    }
    if additional.state == OccurrenceState::Pending {
        if let Some(existing) = current.iter_mut().find(|item| {
            item.automation_id == automation_id && item.state == OccurrenceState::Pending
        }) {
            let id = existing.id;
            merge_pending(existing, &additional)?;
            return Ok(id);
        }
    }
    let id = additional.id;
    current.push(additional);
    Ok(id)
}

fn pending_from_missed(item: AutomationOccurrence) -> Result<AutomationOccurrence, String> {
    let result = item
        .result
        .ok_or_else(|| "AUTOMATION_RUNTIME_UNAVAILABLE".to_string())?;
    let mut pending = AutomationOccurrence::pending(item.automation_id, item.scheduled_for);
    pending.created_at = item.created_at;
    pending.updated_at = item.updated_at;
    pending.coalesced_count = result.missed_count;
    pending.last_scheduled_for = result.last_scheduled_for;
    Ok(pending)
}
