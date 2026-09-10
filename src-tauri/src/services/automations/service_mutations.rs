use super::*;
use crate::models::{AutomationSchedule, AutomationStatus};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

pub(crate) async fn create_at(
    root: &Path,
    actor: &AutomationActor,
    input: CreateAutomation,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    let fields = [
        "name",
        "description",
        "prompt",
        "target",
        "provider",
        "model",
        "schedule",
        "status",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let operation = super::audit_store::begin(root, actor, "create", None, fields).await?;
    let result = async {
        super::validation::validate_create(&input).await?;
        super::service_helpers::validate_resume_actor(actor, &input.target)?;
        let definition = super::service_helpers::new_definition(actor, input, now);
        super::validation::validate_definition(&definition)?;
        let guard = super::store_lock().await;
        let mut definitions = super::store::read_all_unlocked_at(root)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        if definitions.len() >= 64 {
            return Err(AutomationError::CapacityReached);
        }
        definitions.push(definition.clone());
        super::store::write_definitions_unlocked_at(root, definitions)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        drop(guard);
        super::service_helpers::detail(definition, now)
    }
    .await;
    let id = result.as_ref().ok().map(|detail| detail.definition.id);
    super::audit_store::complete(root, actor, operation, "create", id, result, None).await
}

pub(crate) async fn update_at(
    root: &Path,
    actor: &AutomationActor,
    id: Uuid,
    patch: UpdateAutomation,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    let operation = super::audit_store::begin(
        root,
        actor,
        "update",
        Some(id),
        super::service_helpers::changed_fields(&patch),
    )
    .await?;
    let result = async {
        let current = super::service_helpers::find(root, id).await?;
        super::validation::validate_update(&current, &patch).await?;
        let guard = super::store_lock().await;
        let mut definitions = super::store::read_all_unlocked_at(root)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        let current = definitions
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or(AutomationError::NotFound)?;
        super::validation::validate_update_locked(current, &patch)?;
        super::service_helpers::apply_patch(current, patch, now)?;
        let updated = current.clone();
        super::store::write_definitions_unlocked_at(root, definitions)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        if updated.status == AutomationStatus::Disabled {
            remove_pending(root, id).await?;
        }
        drop(guard);
        super::service_helpers::detail(updated, now)
    }
    .await;
    super::audit_store::complete(root, actor, operation, "update", Some(id), result, None).await
}

pub(crate) async fn delete_at(
    root: &Path,
    actor: &AutomationActor,
    id: Uuid,
) -> Result<(), AutomationError> {
    let operation = super::audit_store::begin(root, actor, "delete", Some(id), Vec::new()).await?;
    let result = async {
        let guard = super::store_lock().await;
        let mut definitions = super::store::read_all_unlocked_at(root)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        let before = definitions.len();
        definitions.retain(|item| item.id != id);
        if definitions.len() == before {
            return Err(AutomationError::NotFound);
        }
        super::store::write_definitions_unlocked_at(root, definitions)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
        remove_pending(root, id).await?;
        drop(guard);
        Ok(())
    }
    .await;
    super::audit_store::complete(root, actor, operation, "delete", Some(id), result, None).await
}

pub(crate) async fn record_completion_at(
    root: &Path,
    id: Uuid,
    finished_at: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut definitions = super::store::read_all_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    let Some(definition) = definitions.iter_mut().find(|item| item.id == id) else {
        return Ok(());
    };
    if definition.status != AutomationStatus::Active {
        return Ok(());
    }
    let changed = match definition.schedule {
        AutomationSchedule::Once { .. } => {
            definition.status = AutomationStatus::Completed;
            true
        }
        AutomationSchedule::AfterCompletion { .. } if definition.anchor_at != Some(finished_at) => {
            definition.anchor_at = Some(finished_at);
            true
        }
        _ => false,
    };
    if changed {
        definition.revision = definition.revision.saturating_add(1);
        super::store::write_definitions_unlocked_at(root, definitions)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
    }
    Ok(())
}

pub(super) async fn disable_missing_target_at(
    root: &Path,
    id: Uuid,
) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut definitions = super::store::read_all_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    let definition = definitions
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or(AutomationError::NotFound)?;
    definition.status = AutomationStatus::Disabled;
    definition.revision = definition.revision.saturating_add(1);
    super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    remove_pending(root, id).await
}

async fn remove_pending(root: &Path, id: Uuid) -> Result<(), AutomationError> {
    super::runtime_store::remove_pending_unlocked_at(root, id)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}
