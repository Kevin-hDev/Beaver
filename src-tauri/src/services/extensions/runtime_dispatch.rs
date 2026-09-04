use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::protocol::HostToolResult;
use super::types::MAX_WORKING_DIRECTORY_CHARS;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn dispatch_tool(
    name: &str,
    arguments: &Value,
    working_directory: &Path,
    caller_cancel: CancellationToken,
) -> Option<ToolResult> {
    let extension_id = super::registry_index::plugin_id_for_tool(name)?;
    let runtime = match super::runtime::global() {
        Ok(runtime) => Arc::clone(runtime),
        Err(_) => return Some(super::tool_result::unavailable()),
    };
    let name = name.to_string();
    let arguments = arguments.clone();
    let working_directory = working_directory.to_path_buf();
    let work = runtime.work.clone();
    let caller_cancel = caller_cancel.clone();
    let result = work
        .run_operation(move |runtime_cancel| async move {
            tokio::select! {
                _ = caller_cancel.cancelled() => ToolResult::cancelled("Annulé."),
                _ = runtime_cancel.cancelled() => super::tool_result::unavailable(),
                result = dispatch_tracked(&runtime, &extension_id, &name, &arguments, &working_directory) => result,
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
    working_directory: &Path,
) -> ToolResult {
    let deadline = super::runtime_lifecycle::new_stop_deadline();
    let host = match super::runtime_lifecycle::ensure_running(extension_id, deadline).await {
        Ok(host) => host,
        Err(_) => return super::tool_result::unavailable(),
    };
    let Some(working_directory) = working_directory.to_str() else {
        return super::runtime_dispatch_result::extension_context_unavailable();
    };
    if working_directory.encode_utf16().count() > MAX_WORKING_DIRECTORY_CHARS {
        return super::runtime_dispatch_result::extension_context_unavailable();
    }
    let response = host
        .request(
            "tool.call",
            json!({
                "name": name,
                "arguments": arguments,
                "context": {"workingDirectory": working_directory},
            }),
        )
        .await
        .and_then(super::runtime::parse::<HostToolResult>);
    if response.is_err() {
        if let Ok((identity, _, current)) =
            runtime.process_for_extension(extension_id, deadline).await
        {
            if Arc::ptr_eq(&current, &host) {
                invalidate(runtime, identity, host, deadline).await;
            }
        }
    }
    super::runtime_dispatch_result::to_tool_result(response)
}

pub async fn emit_event(name: &str, payload: Value) {
    if super::validation::identifier(name).is_err() || super::validation::message(&payload).is_err()
    {
        return;
    }
    let Ok(runtime) = super::runtime::global().map(Arc::clone) else {
        return;
    };
    let name = name.to_string();
    let work = runtime.work.clone();
    let _ = work
        .run_operation(move |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => {},
                _ = emit_tracked(runtime, name, payload) => {},
            }
        })
        .await;
}

async fn emit_tracked(
    runtime: Arc<super::runtime::ExtensionRuntime>,
    name: String,
    payload: Value,
) {
    let snapshots = runtime.hosts.lock().await.usable_snapshots();
    let mut calls = tokio::task::JoinSet::new();
    for (identity, _, process) in snapshots {
        let event = name.clone();
        let body = payload.clone();
        calls.spawn(async move {
            let result = process
                .request("event.emit", json!({"event": event, "payload": body}))
                .await;
            (identity, process, result.is_ok())
        });
    }
    while let Some(Ok((identity, process, succeeded))) = calls.join_next().await {
        if !succeeded {
            invalidate(
                &runtime,
                identity,
                process,
                super::runtime_lifecycle::new_stop_deadline(),
            )
            .await;
        }
    }
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
