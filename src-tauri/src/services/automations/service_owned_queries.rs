use super::*;
use crate::models::AutomationDefinition;
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

pub(crate) async fn is_extension_owned_at(root: &Path, id: Uuid) -> Result<bool, AutomationError> {
    Ok(super::store::read_all_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .into_iter()
        .find(|definition| definition.id == id)
        .ok_or(AutomationError::NotFound)?
        .extension_owner
        .is_some())
}

pub(crate) async fn owned_definition_at(
    root: &Path,
    owner: &ExtensionActorIdentity,
    id: Uuid,
) -> Result<AutomationDefinition, AutomationError> {
    let definition = super::store::read_all_at(root)
        .await
        .map_err(|_| AutomationError::StoreUnavailable)?
        .into_iter()
        .find(|definition| definition.id == id)
        .ok_or(AutomationError::NotFound)?;
    super::ownership::require_owner(&definition, owner)?;
    Ok(definition)
}
