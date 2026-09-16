use crate::models::{AutomationDefinition, AutomationExtensionOwnership, AutomationTarget};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub(super) struct ExtensionAdmission {
    _guard: super::occurrence_cancellation::OccurrenceGuard,
    pub(super) cancel: CancellationToken,
}

pub(super) async fn admit(
    definition: &AutomationDefinition,
    occurrence_id: Uuid,
    parent_cancel: &CancellationToken,
) -> Result<ExtensionAdmission, &'static str> {
    let owner_id = admitted_owner(
        definition,
        crate::services::extensions::automation_owner_is_current,
    )?;
    validate_target_and_model(definition).await?;
    let (guard, cancel) = super::occurrence_cancellation::admit(
        occurrence_id,
        definition.id,
        owner_id,
        parent_cancel,
    )
    .map_err(|_| "automation_admission_failed")?;
    Ok(ExtensionAdmission {
        _guard: guard,
        cancel,
    })
}

pub(super) fn admitted_owner(
    definition: &AutomationDefinition,
    is_current: impl FnOnce(&crate::models::AutomationExtensionOwner) -> bool,
) -> Result<Option<&str>, &'static str> {
    match definition.extension_owner.as_ref() {
        None => Ok(None),
        Some(AutomationExtensionOwnership::Valid(owner))
            if crate::services::automations::extension_consent_is_current(definition)
                && is_current(owner) =>
        {
            Ok(Some(owner.extension_id.as_str()))
        }
        Some(_) => Err("extension_unavailable"),
    }
}

async fn validate_target_and_model(definition: &AutomationDefinition) -> Result<(), &'static str> {
    crate::services::automations::validate_runtime_model(&definition.provider, &definition.model)
        .await
        .map_err(|_| "model_unavailable")?;
    match &definition.target {
        AutomationTarget::NewSession {
            project_id: Some(project_id),
        } => crate::services::agent_local::directory_access::project_path(project_id)
            .await
            .map(|_| ())
            .map_err(|_| "project_unavailable"),
        AutomationTarget::ResumeSession { session_id } => {
            crate::services::agent_local::session_store::get(session_id)
                .await
                .map(|_| ())
                .map_err(|_| "target_session_missing")
        }
        AutomationTarget::NewSession { project_id: None } => Ok(()),
    }
}
