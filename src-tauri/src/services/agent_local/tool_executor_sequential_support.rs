#![expect(clippy::too_many_arguments, reason = "orchestration boundary keeps related runtime context explicit")]
use super::permission_gate::{self, PermissionDecision};
use super::stream_events::AgentEventEmitter;
use super::tool_execution_outcome::ToolExecutionOutcome;
use super::tool_executor_compression::ToolCompression;
use super::tool_executor_helpers::{push_tool_result, resolve_tool_path};
use super::types_ollama::ChatMessage;
use super::types_tools::ToolResult;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

pub(super) async fn initial_validation(
    session_id: &str,
    request_id: &str,
    name: &str,
    args: &Value,
    working_dir: &std::path::Path,
    plan_mode_active: bool,
    summary: Option<Value>,
) -> Result<Option<Value>, ToolResult> {
    if let Err(message) =
        super::subagent_tool_guard::validate_for_session(session_id, name, args, working_dir).await
    {
        let result = super::tool_executor_errors::permission(
            message,
            "tool_not_allowed_for_session",
        );
        super::tool_executor_diagnostics::completed(
            session_id, request_id, name, summary, &result,
        )
        .await;
        return Err(result);
    }
    if let Err(message) = super::tool_plan_guard::ensure_allowed_for_session(
        name,
        args,
        session_id,
        plan_mode_active,
    )
    .await
    {
        return Err(
            super::tool_executor_plan::denied_with_summary(
                session_id, request_id, name, message, summary,
            )
            .await,
        );
    }
    Ok(summary)
}

pub(super) async fn push_and_compress(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    name: &str,
    args: &serde_json::Value,
    working_dir: &std::path::Path,
    tr: ToolResult,
    idx: usize,
    tool_call_ids: &[String],
    compression: Option<&ToolCompression<'_>>,
) -> ToolExecutionOutcome {
    let resolved_path = resolve_tool_path(name, args, working_dir);
    let follow_up = push_tool_result(
        on_event,
        messages,
        name,
        tr,
        idx,
        tool_call_ids.get(idx).map(String::as_str),
        resolved_path,
    );
    let compressed = match compression {
        Some(compression) => compression.try_run(messages).await,
        None => false,
    };
    let mut outcome = ToolExecutionOutcome::with_compressed(compressed);
    outcome.record(follow_up);
    outcome
}

pub(super) async fn check_allowed(
    on_event: &AgentEventEmitter,
    name: &str,
    args: &serde_json::Value,
    session_id: &str,
    cancel: CancellationToken,
) -> bool {
    if crate::services::extensions::indexed_tool(name).is_some()
        && matches!(
            super::subagent_tool_guard::profile_for_session(session_id).await,
            Ok(Some(_))
        )
    {
        // Le garde enfant vient de valider le mode/cache exacts du parent. Refaire une
        // demande ici la livrerait au parent après un refus ou sous la mauvaise identité.
        return true;
    }
    if !permission_gate::requires_permission(name, args) {
        return true;
    }
    if permission_gate::is_allowed(session_id, name).await {
        return true;
    }
    match permission_gate::request(on_event, name, args, cancel).await {
        PermissionDecision::Allow => true,
        PermissionDecision::AllowSession => {
            permission_gate::mark_allowed(session_id, name).await;
            true
        }
        PermissionDecision::Deny => false,
    }
}
