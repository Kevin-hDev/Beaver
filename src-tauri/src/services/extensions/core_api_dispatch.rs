use super::core_bridge::ExtensionBridgeError;
use super::types::{CoreApiMethodContract, ExtensionEffect};
use std::time::Duration;

pub(super) struct CoreMethodPolicy {
    pub(super) effect: ExtensionEffect,
    pub(super) budget: Duration,
}

pub(super) fn policy(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
) -> Result<CoreMethodPolicy, ExtensionBridgeError> {
    if let Some(reason) = context.core_scope_error() {
        return Err(ExtensionBridgeError::Context(reason));
    }
    if let Some(contract) = super::types::CORE_API_METHODS
        .iter()
        .find(|contract| contract.name == method)
    {
        return typed_policy(context, contract);
    }
    legacy_policy(context, method)
}

fn typed_policy(
    context: &super::call_context::ExtensionCallContext,
    contract: &CoreApiMethodContract,
) -> Result<CoreMethodPolicy, ExtensionBridgeError> {
    if !context.has_capability(contract.capability) {
        return Err(ExtensionBridgeError::MethodUnavailable);
    }
    if contract.requires_context {
        require_scope(context)?;
    }
    Ok(CoreMethodPolicy {
        effect: strongest_effect(contract.effects),
        budget: declared_budget(contract.name),
    })
}

fn legacy_policy(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
) -> Result<CoreMethodPolicy, ExtensionBridgeError> {
    let (_, level, kind, budget) = super::types::HOST_TO_CORE_METHODS
        .iter()
        .find(|(declared, _, _, _)| *declared == method)
        .ok_or(ExtensionBridgeError::MethodUnavailable)?;
    if !method_is_allowed(context.api_level(), level, kind) {
        return Err(ExtensionBridgeError::MethodUnavailable);
    }
    Ok(CoreMethodPolicy {
        effect: legacy_effect(method),
        budget: Duration::from_millis(
            budget
                .filter(|value| *value > 0)
                .unwrap_or(super::types::CORE_REQUEST_TIMEOUT_MS) as u64,
        ),
    })
}

pub(super) fn method_is_allowed(
    api_level: &super::types::ExtensionApiLevel,
    declared_level: &str,
    kind: &str,
) -> bool {
    kind == "request"
        && match declared_level {
            "stable" => true,
            "advanced" => *api_level == super::types::ExtensionApiLevel::Advanced,
            _ => false,
        }
}

fn require_scope(
    context: &super::call_context::ExtensionCallContext,
) -> Result<(), ExtensionBridgeError> {
    if let Some(reason) = context.core_scope_error() {
        return Err(ExtensionBridgeError::Context(reason));
    }
    context
        .core_scope()
        .map(|_| ())
        .ok_or(ExtensionBridgeError::Context("core_context_required"))
}

fn declared_budget(method: &str) -> Duration {
    let milliseconds = super::types::HOST_TO_CORE_METHODS
        .iter()
        .find(|(name, _, _, _)| *name == method)
        .and_then(|(_, _, _, budget)| *budget)
        .filter(|value| *value > 0)
        .unwrap_or(super::types::CORE_REQUEST_TIMEOUT_MS);
    Duration::from_millis(milliseconds as u64)
}

fn strongest_effect(effects: &[&str]) -> ExtensionEffect {
    effects
        .iter()
        .map(|effect| match *effect {
            "read-only" => ExtensionEffect::ReadOnly,
            "local-write" => ExtensionEffect::LocalWrite,
            "external-read" => ExtensionEffect::ExternalRead,
            "external-write" => ExtensionEffect::ExternalWrite,
            "process" => ExtensionEffect::Process,
            "secret" => ExtensionEffect::Secret,
            _ => ExtensionEffect::Unknown,
        })
        .max_by_key(|effect| effect_rank(*effect))
        .unwrap_or(ExtensionEffect::Unknown)
}

fn legacy_effect(method: &str) -> ExtensionEffect {
    match method {
        "mcp.tool.call" => ExtensionEffect::ExternalWrite,
        "secrets.provider.get"
        | "secrets.mcp.oauth.get"
        | "secrets.mcp.env.get"
        | "secrets.channel.get" => ExtensionEffect::Secret,
        _ => ExtensionEffect::ReadOnly,
    }
}

fn effect_rank(effect: ExtensionEffect) -> u8 {
    match effect {
        ExtensionEffect::ReadOnly => 0,
        ExtensionEffect::ExternalRead => 1,
        ExtensionEffect::LocalWrite => 2,
        ExtensionEffect::ExternalWrite => 3,
        ExtensionEffect::Process => 4,
        ExtensionEffect::Secret => 5,
        ExtensionEffect::Unknown => 6,
    }
}
