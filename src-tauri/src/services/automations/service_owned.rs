use super::*;
use crate::models::{AutomationDefinition, AutomationStatus, AutomationTarget};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

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
            .filter(|item| super::ownership::belongs_to_extension(item, &owner.id))
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
    if current.status == AutomationStatus::Disabled {
        crate::services::scheduler::cancel_automation_occurrences(id);
    }
    let updated = current.clone();
    if super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .is_err()
    {
        if updated.status == AutomationStatus::Disabled {
            crate::services::scheduler::block_automation_admission(id);
        }
        return Err(AutomationError::StoreUnavailable);
    }
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
    if active && !crate::services::scheduler::extension_automation_mutations_allowed(&owner.id) {
        return Err(AutomationError::ConsentRequired);
    }
    let _guard = super::store_lock().await;
    let mut definitions = mutable_definitions(root).await?;
    let current = owned_mut(&mut definitions, owner, id, revision)?;
    if !active {
        crate::services::scheduler::cancel_automation_occurrences(id);
    }
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
    if super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .is_err()
    {
        if !active {
            crate::services::scheduler::block_automation_admission(id);
        }
        return Err(AutomationError::StoreUnavailable);
    }
    if active {
        crate::services::scheduler::allow_automation_admission(id);
    }
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
    crate::services::scheduler::cancel_automation_occurrences(id);
    if super::retire_if_referenced_unlocked_at(root, id)
        .await
        .is_err()
    {
        crate::services::scheduler::block_automation_admission(id);
        return Err(AutomationError::StoreUnavailable);
    }
    definitions.retain(|item| item.id != id);
    if super::store::write_definitions_unlocked_at(root, definitions)
        .await
        .is_err()
    {
        crate::services::scheduler::block_automation_admission(id);
        return Err(AutomationError::StoreUnavailable);
    }
    crate::services::scheduler::allow_automation_admission(id);
    super::runtime_store::remove_pending_unlocked_at(root, id)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)
}

pub(crate) async fn revoke_owner_at(
    root: &Path,
    extension_id: &str,
) -> Result<usize, AutomationError> {
    let _guard = super::store_lock().await;
    let mut definitions = mutable_definitions(root).await?;
    let mut changed = 0usize;
    for definition in &mut definitions {
        let owned = matches!(
            definition.extension_owner.as_ref(),
            Some(crate::models::AutomationExtensionOwnership::Valid(owner))
                if owner.extension_id == extension_id
        );
        if owned
            && (definition.status != AutomationStatus::Disabled
                || super::ownership::consent_is_current(definition))
        {
            super::ownership::revoke_consent(definition);
            definition.revision = definition.revision.saturating_add(1);
            changed = changed.saturating_add(1);
        }
    }
    if changed > 0 {
        super::store::write_definitions_unlocked_at(root, definitions)
            .await
            .map_err(|_| AutomationError::StoreUnavailable)?;
    }
    Ok(changed)
}

pub(crate) async fn disable_unavailable_at(root: &Path, id: Uuid) -> Result<(), AutomationError> {
    let _guard = super::store_lock().await;
    let mut definitions = mutable_definitions(root).await?;
    let definition = definitions
        .iter_mut()
        .find(|definition| definition.id == id)
        .ok_or(AutomationError::NotFound)?;
    if definition.extension_owner.is_none() {
        return Ok(());
    }
    crate::services::scheduler::cancel_automation_occurrences(id);
    super::ownership::revoke_consent(definition);
    definition.revision = definition.revision.saturating_add(1);
    super::store::write_definitions_unlocked_at(root, definitions)
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
