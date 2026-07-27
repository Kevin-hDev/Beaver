use super::host_process::HostProcess;
use super::protocol::HostToolResult;
use super::types::HostState;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn dispatch_tool(name: &str, arguments: &Value) -> Option<ToolResult> {
    if !super::registry_index::is_dynamic_tool(name) {
        return None;
    }
    let host = match super::runtime::ensure_running().await {
        Ok(host) => host,
        Err(_) => return Some(ToolResult::err("Extension indisponible.")),
    };
    let response = host
        .request("tool.call", json!({"name": name, "arguments": arguments}))
        .await
        .and_then(super::runtime::parse::<HostToolResult>);
    if response.is_err() {
        invalidate(&host).await;
    }
    Some(to_tool_result(response))
}

pub async fn emit_event(name: &str, payload: Value) {
    if super::validation::identifier(name).is_err() || super::validation::message(&payload).is_err()
    {
        return;
    }
    let Ok(runtime) = super::runtime::global() else {
        return;
    };
    let host = {
        let Ok(process) = runtime.process.try_lock() else {
            return;
        };
        process.as_ref().cloned()
    };
    let Some(host) = host else {
        return;
    };
    if host
        .request("event.emit", json!({"event": name, "payload": payload}))
        .await
        .is_err()
    {
        invalidate(&host).await;
    }
}

fn to_tool_result(result: Result<HostToolResult, String>) -> ToolResult {
    match result {
        Ok(result) => {
            let mut tool_result = if result.is_error {
                ToolResult::err(result.content)
            } else {
                ToolResult::ok(result.content)
            };
            tool_result.display_summary = result.display_summary;
            tool_result
        }
        Err(_) => ToolResult::err("L'extension n'a pas pu exécuter cet outil."),
    }
}

async fn invalidate(failed: &Arc<HostProcess>) {
    let Ok(runtime) = super::runtime::global() else {
        return;
    };
    let removed = {
        let mut slot = runtime.process.lock().await;
        if slot
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, failed))
        {
            slot.take()
        } else {
            None
        }
    };
    if let Some(host) = removed {
        host.kill().await;
        runtime.set_state(
            HostState::Error,
            Some("Hôte d'extensions indisponible.".to_string()),
            0,
        );
        super::runtime::mark_enabled_extensions_error();
    }
}
