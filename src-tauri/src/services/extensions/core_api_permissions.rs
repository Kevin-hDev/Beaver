use super::core_bridge::ExtensionBridgeError;
use super::types::ExtensionEffect;

pub(super) async fn authorize(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &serde_json::Value,
    effect: ExtensionEffect,
) -> Result<(), ExtensionBridgeError> {
    let Some(scope) = context.core_scope() else {
        return Ok(());
    };
    if scope.agent.session_id.is_empty() || scope.agent.request_id.is_empty() {
        return Err(ExtensionBridgeError::Denied);
    }
    if scope.agent.cancel.is_cancelled() || context.revoked().is_cancelled() {
        return Err(ExtensionBridgeError::Revoked);
    }
    if scope.agent.plan_active && effect != ExtensionEffect::ReadOnly {
        return Err(ExtensionBridgeError::Denied);
    }
    if scope
        .agent
        .profile
        .is_some_and(|profile| !profile.allows_extension(effect))
    {
        return Err(ExtensionBridgeError::Denied);
    }
    if scope.agent.permission_mode == "chat" {
        return Err(ExtensionBridgeError::Denied);
    }
    if method == "automations.setActive"
        && params.get("active").and_then(serde_json::Value::as_bool) == Some(true)
    {
        let arguments = super::core_automations::approval_arguments(context, params).await?;
        return request_confirmation(context, method, effect, &arguments).await;
    }
    if !crate::services::agent_local::permission_policy::uses_auto_bypass(
        &scope.agent.permission_mode,
    ) && needs_nested_confirmation(scope.tool_effect, effect)
    {
        if scope.agent.purpose != crate::services::llm::request_purpose::RequestPurpose::ManualChat
        {
            return Err(ExtensionBridgeError::Denied);
        }
        let arguments = serde_json::json!({"coreMethod": method});
        request_confirmation(context, method, effect, &arguments).await?;
    }
    Ok(())
}

async fn request_confirmation(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    effect: ExtensionEffect,
    arguments: &serde_json::Value,
) -> Result<(), ExtensionBridgeError> {
    let scope = context.core_scope().ok_or(ExtensionBridgeError::Denied)?;
    if scope.agent.purpose != crate::services::llm::request_purpose::RequestPurpose::ManualChat {
        return Err(ExtensionBridgeError::Denied);
    }
    if std::time::Instant::now() >= scope.deadline {
        return Err(ExtensionBridgeError::Timeout);
    }
    let decision = crate::services::agent_local::permission_gate::request_extension_core(
        &scope.agent.on_event,
        &scope.tool_name,
        method,
        effect,
        arguments,
        scope.agent.cancel.clone(),
        scope.deadline,
    )
    .await;
    if decision == crate::services::agent_local::permission_gate::PermissionDecision::Allow {
        Ok(())
    } else if scope.agent.cancel.is_cancelled() {
        Err(ExtensionBridgeError::Revoked)
    } else {
        Err(ExtensionBridgeError::Denied)
    }
}

fn needs_nested_confirmation(parent: ExtensionEffect, requested: ExtensionEffect) -> bool {
    crate::services::agent_local::permission_policy::extension_effect_policy(requested)
        .requires_confirmation
        && parent != requested
        && parent != ExtensionEffect::Unknown
}
