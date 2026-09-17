use crate::models::{AutomationStatus, AutomationTarget};
use crate::services::automations::{
    AutomationActor, AutomationOrigin, CreateAutomation, ExtensionActorIdentity, UpdateAutomation,
};
use serde_json::{json, Value};

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

pub(super) async fn approval_arguments(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
) -> Result<Value, ExtensionBridgeError> {
    let owner = super::registry_access::automation_identity(context)
        .map_err(|_| ExtensionBridgeError::Denied)?;
    let definition = crate::services::automations::service_owned_api::definition_for_approval(
        &owner,
        super::core_automations_params::id(params)?,
    )
    .await
    .map_err(super::core_automations_params::map_error)?;
    Ok(approval_arguments_from_definition(definition))
}

pub(super) fn approval_arguments_from_definition(
    definition: crate::models::AutomationDefinition,
) -> Value {
    json!({
        "coreMethod": "automations.setActive",
        "name": definition.name,
        "prompt": definition.prompt,
        "schedule": definition.schedule,
        "provider": definition.provider,
        "model": definition.model,
        "target": definition.target,
        "futureAgentExecution": true,
    })
}

pub(super) async fn call(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let owner = super::registry_access::automation_identity(context)
        .map_err(|_| ExtensionBridgeError::Denied)?;
    let actor = actor(context, &owner, method, params).await?;
    match method {
        "automations.list" => list(&actor, &owner, params).await,
        "automations.create" => create(context, &actor, &owner, params).await,
        "automations.update" => update(&actor, &owner, params).await,
        "automations.setActive" => set_active(&actor, &owner, params).await,
        "automations.delete" => delete(&actor, &owner, params).await,
        _ => Err(ExtensionBridgeError::MethodUnavailable),
    }
}

async fn list(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let offset = super::core_automations_params::cursor(params)?;
    let items = crate::services::automations::service_owned_api::list(actor, owner)
        .await
        .map_err(super::core_automations_params::map_error)?;
    let total = items.len();
    let page = items
        .into_iter()
        .skip(offset)
        .take(super::types::MAX_SDK_PAGE_RESULTS)
        .map(super::core_automations_params::public_automation)
        .collect::<Vec<_>>();
    let next =
        (offset.saturating_add(page.len()) < total).then(|| (offset + page.len()).to_string());
    Ok(CoreResponse::Json(
        json!({"items": page, "nextCursor": next}),
    ))
}

async fn create(
    context: &super::call_context::ExtensionCallContext,
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let scope = context.core_scope().ok_or(ExtensionBridgeError::Denied)?;
    let session = crate::services::agent_local::session_store::get(&scope.agent.session_id)
        .await
        .map_err(|_| ExtensionBridgeError::Failed)?;
    let input = CreateAutomation {
        name: super::core_automations_params::required(params, "name")?.to_string(),
        description: super::core_automations_params::optional(params, "description")?
            .map(str::to_string),
        prompt: super::core_automations_params::required(params, "prompt")?.to_string(),
        target: AutomationTarget::NewSession {
            project_id: session.project_id,
        },
        provider: session.provider,
        model: session.model,
        schedule: super::core_automations_params::schedule(params.get("schedule"))?,
        status: AutomationStatus::Disabled,
    };
    let created = crate::services::automations::service_owned_api::create(actor, owner, input)
        .await
        .map_err(super::core_automations_params::map_error)?;
    Ok(CoreResponse::Json(
        super::core_automations_params::public_automation(created.definition),
    ))
}

async fn update(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let patch = UpdateAutomation {
        name: super::core_automations_params::optional(params, "name")?.map(str::to_string),
        description: super::core_automations_params::optional(params, "description")?
            .map(|value| Some(value.to_string())),
        prompt: super::core_automations_params::optional(params, "prompt")?.map(str::to_string),
        schedule: params
            .get("schedule")
            .map(|value| super::core_automations_params::schedule(Some(value)))
            .transpose()?,
        ..Default::default()
    };
    let updated = crate::services::automations::service_owned_api::update(
        actor,
        owner,
        super::core_automations_params::id(params)?,
        super::core_automations_params::revision(params)?,
        patch,
    )
    .await
    .map_err(super::core_automations_params::map_error)?;
    Ok(CoreResponse::Json(
        super::core_automations_params::public_automation(updated.definition),
    ))
}

async fn set_active(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let active = params
        .get("active")
        .and_then(Value::as_bool)
        .ok_or(ExtensionBridgeError::Denied)?;
    let updated = crate::services::automations::service_owned_api::set_active(
        actor,
        owner,
        super::core_automations_params::id(params)?,
        super::core_automations_params::revision(params)?,
        active,
    )
    .await
    .map_err(super::core_automations_params::map_error)?;
    Ok(CoreResponse::Json(
        super::core_automations_params::public_automation(updated.definition),
    ))
}

async fn delete(
    actor: &AutomationActor,
    owner: &ExtensionActorIdentity,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let id = super::core_automations_params::id(params)?;
    crate::services::automations::service_owned_api::delete(
        actor,
        owner,
        id,
        super::core_automations_params::revision(params)?,
    )
    .await
    .map_err(super::core_automations_params::map_error)?;
    Ok(CoreResponse::Json(json!({"deleted": true})))
}

async fn actor(
    context: &super::call_context::ExtensionCallContext,
    owner: &ExtensionActorIdentity,
    method: &str,
    params: &Value,
) -> Result<AutomationActor, ExtensionBridgeError> {
    let scope = context.core_scope().ok_or(ExtensionBridgeError::Denied)?;
    let base = crate::services::automations::actor_context::actor_for(
        &scope.agent.session_id,
        false,
        None,
        Some(&scope.agent.request_id),
    )
    .map_err(super::core_automations_params::map_error)?;
    let session = crate::services::agent_local::session_store::get(&scope.agent.session_id)
        .await
        .map_err(|_| ExtensionBridgeError::Failed)?;
    if restricted_execution(
        method,
        params,
        base.current_automation_id.is_some(),
        session.parent_session_id.is_some(),
    ) {
        return Err(ExtensionBridgeError::Denied);
    }
    Ok(AutomationActor {
        origin: AutomationOrigin::Extension,
        session_or_channel_id: owner.id.clone(),
        current_automation_id: base.current_automation_id,
    })
}

pub(super) fn restricted_execution(
    method: &str,
    params: &Value,
    is_automation: bool,
    is_subagent: bool,
) -> bool {
    if !is_automation && !is_subagent {
        return false;
    }
    matches!(method, "automations.create" | "automations.update")
        || (method == "automations.setActive"
            && params.get("active").and_then(Value::as_bool) == Some(true))
}
