use super::host_process::HostProcess;
use super::protocol::HostToolResult;
use super::types::{HostState, MAX_WORKING_DIRECTORY_CHARS};
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

pub async fn dispatch_tool(
    name: &str,
    arguments: &Value,
    working_directory: &Path,
) -> Option<ToolResult> {
    if !super::registry_index::is_dynamic_tool(name) {
        return None;
    }
    let runtime = match super::runtime::global() {
        Ok(runtime) => Arc::clone(runtime),
        Err(_) => return Some(super::tool_result::unavailable()),
    };
    let name = name.to_string();
    let arguments = arguments.clone();
    let working_directory = working_directory.to_path_buf();
    let work = runtime.work.clone();
    let result = work
        .run_operation(move |cancel| async move {
            tokio::select! {
                _ = cancel.cancelled() => super::tool_result::unavailable(),
                result = dispatch_tracked(&name, &arguments, &working_directory) => result,
            }
        })
        .await;
    Some(result.unwrap_or_else(|_| super::tool_result::unavailable()))
}

async fn dispatch_tracked(name: &str, arguments: &Value, working_directory: &Path) -> ToolResult {
    let host = match super::runtime_lifecycle::ensure_running().await {
        Ok(host) => host,
        Err(_) => return super::tool_result::unavailable(),
    };
    let Some(working_directory) = working_directory.to_str() else {
        return extension_context_unavailable();
    };
    if working_directory.encode_utf16().count() > MAX_WORKING_DIRECTORY_CHARS {
        return extension_context_unavailable();
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
        invalidate(&host).await;
    }
    to_tool_result(response)
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
                _ = emit_tracked(&runtime, &name, payload) => {},
            }
        })
        .await;
}

async fn emit_tracked(runtime: &Arc<super::runtime::ExtensionRuntime>, name: &str, payload: Value) {
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
                ToolResult::error(
                    result.content,
                    "extension_tool_error",
                    ToolErrorCategory::External,
                    false,
                )
            } else {
                ToolResult::ok(result.content)
            };
            tool_result.mark_truncated(result.truncated);
            if let Some(summary) = result.display_summary {
                tool_result = tool_result.with_display_summary(summary);
            }
            tool_result
        }
        Err(_) => ToolResult::error(
            "L'extension n'a pas pu confirmer le résultat de cet outil.",
            "extension_host_failed",
            ToolErrorCategory::External,
            false,
        )
        .with_error_hint(
            "Vérifier l'état du projet ou du service avant de relancer : l'action a pu être exécutée.",
        ),
    }
}

fn extension_context_unavailable() -> ToolResult {
    ToolResult::error(
        "Contexte d'extension indisponible.",
        "extension_context_unavailable",
        ToolErrorCategory::Unavailable,
        false,
    )
}

async fn invalidate(failed: &Arc<HostProcess>) {
    let Ok(runtime) = super::runtime::global() else {
        return;
    };
    let outcome = super::runtime_lifecycle::stop_host_slot(
        &runtime.process,
        Some(failed),
        super::host_process::stop_deadline(),
    )
    .await;
    if outcome != super::runtime_lifecycle::StopHostOutcome::NotCurrent {
        runtime.set_state(
            HostState::Error,
            Some("Hôte d'extensions indisponible.".to_string()),
            0,
        );
        super::runtime::mark_enabled_extensions_error();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncertain_host_failure_never_recommends_a_blind_retry() {
        let result = to_tool_result(Err("host disconnected".to_string()));
        let error = result.error.expect("structured extension error");

        assert_eq!(error.code.as_ref(), "extension_host_failed");
        assert!(!error.retryable);
        assert!(error.hint.is_some());
    }

    #[test]
    fn extension_reported_error_stays_an_error() {
        let result = to_tool_result(Ok(HostToolResult {
            content: "invalid input".to_string(),
            is_error: true,
            truncated: false,
            display_summary: None,
        }));

        assert!(result.is_error);
        assert_eq!(result.error.unwrap().code.as_ref(), "extension_tool_error");
    }
}
