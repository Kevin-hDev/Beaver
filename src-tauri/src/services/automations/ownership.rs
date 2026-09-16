use super::{AutomationError, ExtensionActorIdentity};
use crate::models::{AutomationDefinition, AutomationExtensionOwner, AutomationExtensionOwnership};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct ApprovedContent<'a> {
    prompt: &'a str,
    schedule: &'a crate::models::AutomationSchedule,
    model: &'a str,
    provider: &'a str,
    target: &'a crate::models::AutomationTarget,
}

pub(super) fn new_owner(identity: &ExtensionActorIdentity) -> AutomationExtensionOwnership {
    AutomationExtensionOwnership::Valid(AutomationExtensionOwner {
        extension_id: identity.id.clone(),
        extension_version: identity.version.clone(),
        extension_fingerprint: identity.fingerprint.clone(),
        approved_content_sha256: None,
        approved_at: None,
    })
}

pub(super) fn require_owner<'a>(
    definition: &'a AutomationDefinition,
    identity: &ExtensionActorIdentity,
) -> Result<&'a AutomationExtensionOwner, AutomationError> {
    match definition.extension_owner.as_ref() {
        Some(AutomationExtensionOwnership::Valid(owner))
            if owner.extension_id == identity.id
                && owner.extension_version == identity.version
                && owner.extension_fingerprint == identity.fingerprint =>
        {
            Ok(owner)
        }
        _ => Err(AutomationError::NotFound),
    }
}

pub(super) fn belongs_to_extension(definition: &AutomationDefinition, extension_id: &str) -> bool {
    matches!(
        definition.extension_owner.as_ref(),
        Some(AutomationExtensionOwnership::Valid(owner)) if owner.extension_id == extension_id
    )
}

pub(super) fn approve(
    definition: &mut AutomationDefinition,
    identity: &ExtensionActorIdentity,
    now: DateTime<Utc>,
) -> Result<(), AutomationError> {
    let fingerprint = content_fingerprint(definition)?;
    let owner = match definition.extension_owner.as_mut() {
        Some(AutomationExtensionOwnership::Valid(owner))
            if owner.extension_id == identity.id
                && owner.extension_version == identity.version
                && owner.extension_fingerprint == identity.fingerprint =>
        {
            owner
        }
        _ => return Err(AutomationError::NotFound),
    };
    owner.approved_content_sha256 = Some(fingerprint);
    owner.approved_at = Some(now);
    Ok(())
}

pub(crate) fn consent_is_current(definition: &AutomationDefinition) -> bool {
    let Some(AutomationExtensionOwnership::Valid(owner)) = definition.extension_owner.as_ref()
    else {
        return definition.extension_owner.is_none();
    };
    owner.approved_content_sha256.as_deref() == content_fingerprint(definition).ok().as_deref()
        && owner.approved_at.is_some()
}

pub(super) fn revoke_consent(definition: &mut AutomationDefinition) {
    if let Some(AutomationExtensionOwnership::Valid(owner)) = definition.extension_owner.as_mut() {
        owner.approved_content_sha256 = None;
        owner.approved_at = None;
    }
    definition.status = crate::models::AutomationStatus::Disabled;
}

pub(super) fn invalidate_if_changed(
    before: &AutomationDefinition,
    after: &mut AutomationDefinition,
) -> Result<(), AutomationError> {
    if before.extension_owner.is_none()
        || content_fingerprint(before)? == content_fingerprint(after)?
    {
        return Ok(());
    }
    revoke_consent(after);
    Ok(())
}

pub(crate) fn content_fingerprint(
    definition: &AutomationDefinition,
) -> Result<String, AutomationError> {
    let bytes = serde_json::to_vec(&ApprovedContent {
        prompt: &definition.prompt,
        schedule: &definition.schedule,
        model: &definition.model,
        provider: &definition.provider,
        target: &definition.target,
    })
    .map_err(|_| AutomationError::StoreUnavailable)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
