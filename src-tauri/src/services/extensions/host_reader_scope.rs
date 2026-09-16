use super::call_context::ExtensionCallContext;
use super::core_scope::CoreScopeRegistry;
use super::host_identity::HostIdentity;
use serde_json::Value;

pub(super) fn attach(
    context: ExtensionCallContext,
    params: &mut Option<Value>,
    scopes: &CoreScopeRegistry,
    identity: &HostIdentity,
    generation: u64,
) -> ExtensionCallContext {
    let Some(envelope) = params
        .as_mut()
        .and_then(Value::as_object_mut)
        .and_then(|object| object.remove(super::types::CORE_CONTEXT_ENVELOPE_FIELD))
    else {
        return context;
    };
    let Some(envelope) = envelope.as_object().filter(|value| value.len() == 3) else {
        return context.with_core_scope_error("core_context_invalid");
    };
    let (Some(id), Some(secret), Some(remaining)) = (
        envelope.get("id").and_then(Value::as_str),
        envelope.get("secret").and_then(Value::as_str),
        envelope.get("remainingMs").and_then(Value::as_u64),
    ) else {
        return context.with_core_scope_error("core_context_invalid");
    };
    if remaining == 0 || remaining > super::types::TOOL_CALL_TIMEOUT_MS as u64 {
        return context.with_core_scope_error("core_context_invalid");
    }
    match scopes.resolve(id, secret, identity, generation) {
        Ok(scope) => context.with_core_scope(scope),
        Err(reason) => context.with_core_scope_error(reason),
    }
}
