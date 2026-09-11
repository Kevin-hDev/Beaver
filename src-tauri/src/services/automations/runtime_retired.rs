use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

const MAX_RETIRED_IDS: usize = 128;

pub async fn reserved_automation_ids() -> Result<HashSet<Uuid>, super::AutomationError> {
    reserved_at(&crate::services::paths::data_dir())
        .await
        .map_err(|_| super::AutomationError::StoreUnavailable)
}

pub(crate) async fn reserved_at(root: &Path) -> Result<HashSet<Uuid>, String> {
    let _guard = super::store_lock().await;
    reserved_unlocked_at(root).await
}

pub(crate) async fn reserved_unlocked_at(root: &Path) -> Result<HashSet<Uuid>, String> {
    Ok(super::runtime_store::read_unlocked_at(root)
        .await?
        .map(|runtime| {
            runtime
                .occurrences
                .into_iter()
                .map(|item| item.automation_id)
                .chain(runtime.retired_automation_ids)
                .collect()
        })
        .unwrap_or_default())
}

pub(crate) async fn retire_if_referenced_unlocked_at(
    root: &Path,
    automation_id: Uuid,
) -> Result<(), super::AutomationError> {
    let Some(mut runtime) = super::runtime_store::read_unlocked_at(root)
        .await
        .map_err(|_| super::AutomationError::StoreUnavailable)?
    else {
        return Ok(());
    };
    if !runtime.occurrences.iter().any(|item| {
        item.automation_id == automation_id
            && item.state != super::runtime_store::OccurrenceState::Pending
    }) || runtime.retired_automation_ids.contains(&automation_id)
    {
        return Ok(());
    }
    if runtime.retired_automation_ids.len() == MAX_RETIRED_IDS {
        return Err(super::AutomationError::CapacityReached);
    }
    runtime.retired_automation_ids.push(automation_id);
    super::runtime_store::write_unlocked_at(root, &runtime)
        .await
        .map_err(|_| super::AutomationError::StoreUnavailable)
}

pub(crate) async fn release_unlocked_at(root: &Path, automation_id: Uuid) -> Result<(), String> {
    let Some(mut runtime) = super::runtime_store::read_unlocked_at(root).await? else {
        return Ok(());
    };
    let before = runtime.retired_automation_ids.len();
    runtime
        .retired_automation_ids
        .retain(|id| *id != automation_id);
    if runtime.retired_automation_ids.len() != before {
        super::runtime_store::write_unlocked_at(root, &runtime).await?;
    }
    Ok(())
}
