use crate::services::agent_local::memory_paths::{MemoryLayout, MemoryScope};
use serde_json::Value;

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

pub(super) async fn call(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    call_with_layout(context, method, params, &MemoryLayout::production()).await
}

async fn call_with_layout(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &Value,
    layout: &MemoryLayout,
) -> Result<CoreResponse, ExtensionBridgeError> {
    validate_keys(method, params)?;
    let scope = resolve_scope(context, params, layout).await?;
    match method {
        "memory.list" => super::core_memory_reads::list(context, params, &scope).await,
        "memory.read" => super::core_memory_reads::read(context, params, &scope).await,
        "memory.write" => {
            super::core_memory_mutations::write(context, params, &scope).await
        }
        "memory.archive" => {
            super::core_memory_mutations::archive(context, params, &scope).await
        }
        _ => Err(ExtensionBridgeError::MethodUnavailable),
    }
}

async fn resolve_scope(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
    layout: &MemoryLayout,
) -> Result<MemoryScope, ExtensionBridgeError> {
    let scope = params
        .get("scope")
        .and_then(Value::as_str)
        .ok_or(ExtensionBridgeError::Denied)?;
    match scope {
        "global" => Ok(layout.global_scope()),
        "project" => {
            let working_directory = context
                .core_scope()
                .ok_or(ExtensionBridgeError::Denied)?
                .agent
                .working_directory
                .canonicalize()
                .map_err(|_| ExtensionBridgeError::Denied)?;
            layout
                .project_scope_ready(&working_directory)
                .await
                .map_err(|_| ExtensionBridgeError::Denied)
        }
        _ => Err(ExtensionBridgeError::Denied),
    }
}

fn validate_keys(method: &str, params: &Value) -> Result<(), ExtensionBridgeError> {
    let allowed: &[&str] = match method {
        "memory.list" => &["scope", "cursor"],
        "memory.read" | "memory.archive" => &["scope", "topicId"],
        "memory.write" => &["scope", "topicId", "expectedUpdatedAt", "content"],
        _ => return Err(ExtensionBridgeError::MethodUnavailable),
    };
    let object = params.as_object().ok_or(ExtensionBridgeError::Denied)?;
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(ExtensionBridgeError::Denied);
    }
    Ok(())
}

#[cfg(test)]
#[path = "core_memory_tests.rs"]
mod tests;
