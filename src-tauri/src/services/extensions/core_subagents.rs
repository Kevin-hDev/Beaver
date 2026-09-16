use serde_json::{json, Value};

use super::core_bridge::{CoreResponse, ExtensionBridgeError};
use crate::services::agent_local::types_session::{AgentSession, SubagentExtensionOwner};

pub(super) async fn call(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let scope = context.core_scope().ok_or(ExtensionBridgeError::Denied)?;
    validate_keys(method, params)?;
    if !scope_allows_subagents(
        scope.agent.profile.is_none(),
        scope.agent.plan_active,
        &scope.agent.permission_mode,
    ) {
        return Err(ExtensionBridgeError::Denied);
    }
    let identity = super::registry_access::automation_identity(context)
        .map_err(|_| ExtensionBridgeError::Denied)?;
    let owner = SubagentExtensionOwner {
        extension_id: identity.id,
        extension_version: identity.version,
        extension_fingerprint: identity.fingerprint,
    };
    match method {
        "subagents.spawn" => spawn(scope, owner, params).await,
        "subagents.list" => list(scope, &owner, params).await,
        "subagents.get" => get(scope, &owner, params).await,
        "subagents.send" => send(scope, &owner, params).await,
        "subagents.cancel" => cancel(scope, &owner, params).await,
        _ => Err(ExtensionBridgeError::MethodUnavailable),
    }
}

async fn spawn(
    scope: &super::core_scope::AuthorizedCoreScope,
    owner: SubagentExtensionOwner,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let child = crate::services::agent_local::subagent_extension_api::spawn(
        &scope.agent.session_id,
        owner,
        required(params, "type")?,
        required(params, "prompt")?,
        scope.agent.cancel.clone(),
    )
    .await
    .map_err(map_error)?;
    Ok(CoreResponse::Json(public(&child)))
}

async fn list(
    scope: &super::core_scope::AuthorizedCoreScope,
    owner: &SubagentExtensionOwner,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let offset = params
        .get("cursor")
        .and_then(Value::as_str)
        .unwrap_or("0")
        .parse::<usize>()
        .map_err(|_| ExtensionBridgeError::Denied)?;
    let items = crate::services::agent_local::subagent_extension_api::list(
        &scope.agent.session_id,
        owner,
    )
    .await
    .map_err(map_error)?;
    let total = items.len();
    let page = items
        .into_iter()
        .skip(offset)
        .take(super::types::MAX_SDK_PAGE_RESULTS)
        .map(|child| public(&child))
        .collect::<Vec<_>>();
    let next = (offset.saturating_add(page.len()) < total).then(|| (offset + page.len()).to_string());
    Ok(CoreResponse::Json(json!({"items": page, "nextCursor": next})))
}

async fn get(
    scope: &super::core_scope::AuthorizedCoreScope,
    owner: &SubagentExtensionOwner,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let child = owned(scope, owner, params).await?;
    Ok(CoreResponse::Json(public(&child)))
}

async fn send(
    scope: &super::core_scope::AuthorizedCoreScope,
    owner: &SubagentExtensionOwner,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    crate::services::agent_local::subagent_extension_api::send(
        required(params, "subagentId")?,
        required(params, "prompt")?,
        &scope.agent.session_id,
        owner,
        scope.agent.cancel.clone(),
    )
    .await
    .map_err(map_error)?;
    Ok(CoreResponse::Json(json!({"accepted": true})))
}

async fn cancel(
    scope: &super::core_scope::AuthorizedCoreScope,
    owner: &SubagentExtensionOwner,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let stopped = crate::services::agent_local::subagent_extension_api::cancel(
        required(params, "subagentId")?,
        &scope.agent.session_id,
        owner,
    )
    .await
    .map_err(map_error)?;
    Ok(CoreResponse::Json(json!({"accepted": true, "stopped": stopped})))
}

async fn owned(
    scope: &super::core_scope::AuthorizedCoreScope,
    owner: &SubagentExtensionOwner,
    params: &Value,
) -> Result<AgentSession, ExtensionBridgeError> {
    crate::services::agent_local::subagent_extension_api::get(
        required(params, "subagentId")?,
        &scope.agent.session_id,
        owner,
    )
    .await
    .map_err(map_error)
}

fn required<'a>(params: &'a Value, key: &str) -> Result<&'a str, ExtensionBridgeError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or(ExtensionBridgeError::Denied)
}

pub(super) fn validate_keys(method: &str, params: &Value) -> Result<(), ExtensionBridgeError> {
    let allowed: &[&str] = match method {
        "subagents.spawn" => &["type", "prompt"],
        "subagents.list" => &["cursor"],
        "subagents.get" | "subagents.cancel" => &["subagentId"],
        "subagents.send" => &["subagentId", "prompt"],
        _ => return Err(ExtensionBridgeError::MethodUnavailable),
    };
    let object = params.as_object().ok_or(ExtensionBridgeError::Denied)?;
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(ExtensionBridgeError::Denied);
    }
    Ok(())
}

pub(super) fn scope_allows_subagents(
    is_parent: bool,
    plan_active: bool,
    permission_mode: &str,
) -> bool {
    is_parent && !plan_active && matches!(permission_mode, "manual" | "auto")
}

fn public(child: &AgentSession) -> Value {
    json!({"id": child.id, "type": child.subagent_type, "status": child.subagent_status,
        "report": child.subagent_summary})
}

fn map_error(_: crate::services::agent_local::types_tools::ToolResult) -> ExtensionBridgeError {
    ExtensionBridgeError::Failed
}
