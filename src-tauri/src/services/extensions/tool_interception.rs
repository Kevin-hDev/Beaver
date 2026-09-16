pub(crate) use super::tool_interception_catalog::InterceptionSnapshot;
pub(super) use super::tool_interception_catalog::{InterceptorCatalog, InterceptorRegistration};

use super::tool_interception_result::{CallError, Outcome};
use super::types::ExtensionEffect;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::{json, Value};
use std::future::Future;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(crate) fn snapshot_for_model_request(mode: &str) -> InterceptionSnapshot {
    if mode == "chat" {
        return InterceptionSnapshot::default();
    }
    let Ok(runtime) = super::runtime::global() else {
        return InterceptionSnapshot::default();
    };
    let Ok(catalog) = super::registry_catalog() else {
        return InterceptionSnapshot::default();
    };
    runtime
        .tool_interceptors
        .snapshot(&catalog.ordered_plugin_ids)
}

pub(crate) async fn before_effect(
    snapshot: &InterceptionSnapshot,
    tool_name: &str,
    arguments: &Value,
    working_dir: &std::path::Path,
    mode: &str,
    cancel: &CancellationToken,
) -> Result<(), ToolResult> {
    if mode == "chat" || snapshot.is_empty() {
        return Ok(());
    }
    let call = json!({
        "toolName": tool_name,
        "effect": effect_for(tool_name),
        "mode": mode,
        "argumentSummary": crate::services::agent_local::diagnostic_args::summarize(
            tool_name,
            arguments,
            working_dir,
        ),
    });
    let (outcome, owner) =
        run_chain(
            &snapshot.entries,
            call,
            cancel,
            |entry, call, deadline| async move {
                call_interceptor(&entry, call, deadline, cancel).await
            },
        )
        .await;
    match outcome {
        Outcome::Continue => Ok(()),
        Outcome::Deny => Err(super::tool_interception_result::denied()),
        Outcome::ChainTimeout => {
            if let Some(entry) = owner.as_ref() {
                record_diagnostic(
                    entry,
                    super::types::DIAGNOSTIC_INTERCEPTION_BUDGET_EXHAUSTED,
                );
            }
            Err(super::tool_interception_result::chain_timeout())
        }
        Outcome::Disable(code) => {
            if let Some(entry) = owner.as_ref() {
                super::tool_interception_failure::disable(entry, code).await;
            }
            Err(super::tool_interception_result::individual_failure())
        }
        Outcome::Cancelled => Err(ToolResult::cancelled("Annulé.")),
    }
}

pub(super) async fn run_chain<F, Fut>(
    entries: &[InterceptorRegistration],
    call: Value,
    cancel: &CancellationToken,
    mut invoke: F,
) -> (Outcome, Option<InterceptorRegistration>)
where
    F: FnMut(InterceptorRegistration, Value, tokio::time::Instant) -> Fut,
    Fut: Future<Output = Result<Value, CallError>>,
{
    let chain_deadline = tokio::time::Instant::now()
        + Duration::from_millis(super::types::INTERCEPTOR_CHAIN_TIMEOUT_MS as u64);
    for entry in entries {
        if tokio::time::Instant::now() >= chain_deadline {
            return (Outcome::ChainTimeout, Some(entry.clone()));
        }
        let handler_deadline = tokio::time::Instant::now()
            + Duration::from_millis(super::types::INTERCEPTOR_HANDLER_TIMEOUT_MS as u64);
        let response = tokio::select! {
            _ = cancel.cancelled() => Err(CallError::Cancelled),
            response = invoke(entry.clone(), call.clone(), handler_deadline.min(chain_deadline)) => response,
        };
        match super::tool_interception_result::classify(response, handler_deadline, chain_deadline)
        {
            Outcome::Continue => {}
            outcome => return (outcome, Some(entry.clone())),
        }
    }
    (Outcome::Continue, None)
}

async fn call_interceptor(
    entry: &InterceptorRegistration,
    call: Value,
    deadline: tokio::time::Instant,
    cancel: &CancellationToken,
) -> Result<Value, CallError> {
    let request = async {
        let runtime = super::runtime::global().map_err(|_| CallError::Failed)?;
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let (identity, generation, host) = runtime
            .process_for_extension(&entry.extension_id, std::time::Instant::now() + remaining)
            .await
            .map_err(|_| CallError::Failed)?;
        if identity != entry.identity || generation != entry.generation {
            return Err(CallError::Failed);
        }
        host.request_until_tokio(
            "tool.intercept",
            json!({"extensionId": entry.extension_id, "call": call}),
            deadline,
        )
        .await
        .map_err(|error| {
            if error == super::error_codes::HOST_TIMEOUT {
                CallError::Timeout
            } else {
                CallError::Failed
            }
        })
    };
    tokio::select! {
        _ = cancel.cancelled() => Err(CallError::Cancelled),
        result = tokio::time::timeout_at(deadline, request) => {
            result.unwrap_or(Err(CallError::Timeout))
        },
    }
}

fn effect_for(tool_name: &str) -> ExtensionEffect {
    super::indexed_tool(tool_name)
        .map(|indexed| indexed.tool.effect)
        .unwrap_or_else(|| {
            if crate::services::agent_local::tool_executor_read_only::is_read_only(tool_name) {
                ExtensionEffect::ReadOnly
            } else {
                ExtensionEffect::Unknown
            }
        })
}

fn record_diagnostic(entry: &InterceptorRegistration, code: &'static str) {
    if let Ok(runtime) = super::runtime::global() {
        runtime.record_interceptor_diagnostic(&entry.extension_id, code);
    }
}
