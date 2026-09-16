use super::*;
use crate::models::{AutomationDefinition, AutomationStatus, AutomationTarget};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

pub(super) async fn list_at(
    root: &Path,
    owner: &ExtensionActorIdentity,
) -> Result<Vec<AutomationDefinition>, AutomationError> {
    Ok(super::store::read_all_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .into_iter()
        .filter(|item| super::ownership::require_owner(item, owner).is_ok())
        .collect())
}

pub(super) async fn create_at(
    root: &Path,
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    mut input: CreateAutomation,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    input.status = AutomationStatus::Disabled;
    super::validation::validate_create(&input, now).await?;
    if !matches!(input.target, AutomationTarget::NewSession { .. }) {
        return Err(AutomationError::InvalidInput);
    }
    let mut definition = super::service_helpers::new_definition(actor, input, now);
    definition.extension_owner = Some(super::ownership::new_owner(owner));
    let _guard = super::store_lock().await;
    let mut definitions = super::store::read_all_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    if definitions.len() >= super::store::MAX_AUTOMATIONS
        || definitions
            .iter()
            .filter(|item| super::ownership::require_owner(item, owner).is_ok())
            .count()
            >= super::types::MAX_AUTOMATIONS_PER_EXTENSION
    {
        return Err(AutomationError::CapacityReached);
    }
    definitions.push(definition.clone());
    super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    super::service_helpers::detail(definition, now)
}

pub(super) async fn update_at(
    root: &Path,
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
    patch: UpdateAutomation,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    let _guard = super::store_lock().await;
    let mut definitions = mutable_definitions(root).await?;
    let current = owned_mut(&mut definitions, owner, id, revision)?;
    super::validation::validate_update_locked(current, &patch)?;
    let before = current.clone();
    super::service_helpers::apply_patch(current, patch, now)?;
    super::ownership::invalidate_if_changed(&before, current)?;
    let updated = current.clone();
    super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    super::service_helpers::detail(updated, now)
}

pub(super) async fn set_active_at(
    root: &Path,
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
    active: bool,
    globally_paused: bool,
    now: DateTime<Utc>,
) -> Result<AutomationDetail, AutomationError> {
    if active && globally_paused {
        return Err(AutomationError::GloballyPaused);
    }
    let _guard = super::store_lock().await;
    let mut definitions = mutable_definitions(root).await?;
    let current = owned_mut(&mut definitions, owner, id, revision)?;
    current.status = if active {
        AutomationStatus::Active
    } else {
        AutomationStatus::Disabled
    };
    if active {
        super::ownership::approve(current, owner, now)?;
    }
    current.revision = current.revision.saturating_add(1);
    let updated = current.clone();
    super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    if !active {
        super::runtime_store::remove_pending_unlocked_at(root, id)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
    }
    super::service_helpers::detail(updated, now)
}

pub(super) async fn delete_at(
    root: &Path,
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut definitions = mutable_definitions(root).await?;
    owned_mut(&mut definitions, owner, id, revision)?;
    super::retire_if_referenced_unlocked_at(root, id).await?;
    definitions.retain(|item| item.id != id);
    super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?;
    super::runtime_store::remove_pending_unlocked_at(root, id)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}

async fn mutable_definitions(root: &Path) -> Result<Vec<AutomationDefinition>, AutomationError> {
    super::store::read_all_unlocked_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}

fn owned_mut<'a>(
    definitions: &'a mut [AutomationDefinition],
    owner: &ExtensionActorIdentity,
    id: Uuid,
    revision: u64,
) -> Result<&'a mut AutomationDefinition, AutomationError> {
    let current = definitions
        .iter_mut()
        .find(|item| item.id == id)
        .ok_or(AutomationError::NotFound)?;
    super::ownership::require_owner(current, owner)?;
    if current.revision != revision {
        return Err(AutomationError::RevisionConflict);
    }
    Ok(current)
}
