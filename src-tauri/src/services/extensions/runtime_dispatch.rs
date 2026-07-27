use super::protocol::HostToolResult;
use super::types::HostState;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};

pub async fn dispatch_tool(name: &str, arguments: &Value) -> Option<ToolResult> {
    if !super::registry::is_dynamic_tool(name) {
        return None;
    }
    let runtime = match super::runtime::global() {
        Ok(runtime) => runtime,
        Err(_) => return Some(ToolResult::err("Extension indisponible.")),
    };
    let mut process = runtime.process.lock().await;
    let response = match process.as_mut() {
        Some(host) => host
            .request("tool.call", json!({"name": name, "arguments": arguments}))
            .await
            .and_then(super::runtime::parse::<HostToolResult>),
        None => return Some(ToolResult::err("Extension indisponible.")),
    };
    if response.is_err() {
        invalidate(runtime, &mut process).await;
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
    let Ok(mut process) = runtime.process.try_lock() else {
        return;
    };
    let Some(host) = process.as_mut() else {
        return;
    };
    if host
        .request("event.emit", json!({"event": name, "payload": payload}))
        .await
        .is_err()
    {
        invalidate(runtime, &mut process).await;
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

async fn invalidate(
    runtime: &super::runtime::ExtensionRuntime,
    slot: &mut Option<super::host_process::HostProcess>,
) {
    if let Some(host) = slot.take() {
        host.kill().await;
    }
    runtime.set_state(
        HostState::Error,
        Some("Hôte d'extensions indisponible.".to_string()),
        0,
    );
    super::runtime::mark_enabled_extensions_error();
}
