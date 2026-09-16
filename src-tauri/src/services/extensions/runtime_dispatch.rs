use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::protocol::HostToolResult;
use super::types::MAX_WORKING_DIRECTORY_CHARS;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub async fn dispatch_tool(
    name: &str,
    arguments: &Value,
    agent_scope: super::core_scope::AgentCoreScope,
) -> Option<ToolResult> {
    let extension_id = super::registry_index::plugin_id_for_tool(name)?;
    let runtime = match super::runtime::global() {
        Ok(runtime) => Arc::clone(runtime),
        Err(_) => return Some(super::tool_result::unavailable()),
    };
    let name = name.to_string();
    let arguments = arguments.clone();
    let work = runtime.work.clone();
    let caller_cancel = agent_scope.cancel.clone();
    let result = work
        .run_operation(move |runtime_cancel| async move {
            tokio::select! {
                _ = caller_cancel.cancelled() => ToolResult::cancelled("Annulé."),
                _ = runtime_cancel.cancelled() => super::tool_result::unavailable(),
                result = dispatch_tracked(&runtime, &extension_id, &name, &arguments, agent_scope) => result,
            }
        })
        .await;
    Some(result.unwrap_or_else(|_| super::tool_result::unavailable()))
}

async fn dispatch_tracked(
    runtime: &Arc<super::runtime::ExtensionRuntime>,
    extension_id: &str,
    name: &str,
    arguments: &Value,
    agent_scope: super::core_scope::AgentCoreScope,
) -> ToolResult {
    let deadline =
        Instant::now() + Duration::from_millis(super::types::TOOL_CALL_TIMEOUT_MS as u64);
    if super::runtime_lifecycle::ensure_running(extension_id, deadline)
        .await
        .is_err()
    {
        return super::tool_result::unavailable();
    }
    let (identity, generation, host) =
        match runtime.process_for_extension(extension_id, deadline).await {
            Ok(channel) => channel,
            Err(_) => return super::tool_result::unavailable(),
        };
    let Some(working_directory) = agent_scope.working_directory.to_str().map(str::to_string) else {
        return super::runtime_dispatch_result::extension_context_unavailable();
    };
    if working_directory.encode_utf16().count() > MAX_WORKING_DIRECTORY_CHARS {
        return super::runtime_dispatch_result::extension_context_unavailable();
    }
    let tool_effect = super::registry_index::indexed_tool(name)
        .map(|indexed| indexed.tool.effect)
        .unwrap_or(super::types::ExtensionEffect::Unknown);
    let lease = match runtime.work.core_scopes().admit(
        identity,
        generation,
        agent_scope,
        name.to_string(),
        tool_effect,
        deadline,
    ) {
        Ok(lease) => lease,
        Err(_) => return core_saturated(),
    };
    let response = host
        .request_until(
            "tool.call",
            json!({
                "name": name,
                "arguments": arguments,
                "context": {"workingDirectory": working_directory},
                "scope": lease.envelope(),
            }),
            deadline,
        )
        .await
        .and_then(super::runtime::parse::<HostToolResult>);
    if response.is_err() {
        if let Ok((identity, _, current)) = runtime
            .process_for_extension(extension_id, super::runtime_lifecycle::new_stop_deadline())
            .await
        {
            if Arc::ptr_eq(&current, &host) {
                invalidate(
                    runtime,
                    identity,
                    host,
                    super::runtime_lifecycle::new_stop_deadline(),
                )
                .await;
            }
        }
    }
    super::runtime_dispatch_result::to_tool_result(response)
}

fn core_saturated() -> ToolResult {
    ToolResult::error(
        "Services d'extension temporairement occupés.",
        super::types::backend_error_codes::CORE_SATURATED,
        crate::services::agent_local::tool_result_contract::ToolErrorCategory::Unavailable,
        true,
    )
}

async fn invalidate(
    runtime: &super::runtime::ExtensionRuntime,
    identity: HostIdentity,
    failed: Arc<HostProcess>,
    deadline: std::time::Instant,
) {
    let should_invalidate = runtime
        .hosts
        .lock()
        .await
        .channel(&identity)
        .is_some_and(|channel| {
            Arc::ptr_eq(&channel.process, &failed)
                && should_invalidate_generation(&channel.generation)
        });
    if !should_invalidate {
        return;
    }
    if runtime
        .stop_host_if_current(&identity, Some(&failed), deadline, false)
        .await
        != super::runtime::StopHostOutcome::Unconfirmed
    {
        let _ = super::registry_sync::mark_identity_error(&identity);
    }
}

fn should_invalidate_generation(generation: &super::runtime_hosts::HostGeneration) -> bool {
    !generation.is_stopping()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopping_generations_are_owned_by_the_stop_path() {
        let generation = super::super::runtime_hosts::HostGeneration::new(1);
        assert!(should_invalidate_generation(&generation));
        generation.begin_stop(true);
        assert!(!should_invalidate_generation(&generation));
    }
}
