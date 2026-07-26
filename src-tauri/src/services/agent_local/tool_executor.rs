use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use super::tool_executor_compression::ToolCompression;
use super::tool_executor_parallel::run_with_parallel_reads;
use super::tool_executor_sequential::run_sequential;

pub async fn run_tools(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    plan_mode_active: bool,
    tool_call_ids: &[String],
    compression: Option<&ToolCompression<'_>>,
) -> bool {
    run_tools_with_eager(
        on_event,
        messages,
        tool_calls,
        working_dir,
        mode,
        session_id,
        request_id,
        cancel,
        write_guard,
        plan_mode_active,
        None,
        tool_call_ids,
        compression,
    )
    .await
}

pub async fn run_tools_with_eager(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    plan_mode_active: bool,
    mut eager_results: Option<HashMap<usize, ToolResult>>,
    tool_call_ids: &[String],
    compression: Option<&ToolCompression<'_>>,
) -> bool {
    let can_use_delegate_batch = matches!(
        super::subagent_tool_guard::profile_for_session(session_id).await,
        Ok(None)
    );
    if can_use_delegate_batch
        && !tool_calls.is_empty()
        && tool_calls
            .iter()
            .all(|(name, _)| name == super::tool_executor_delegate_batch::DELEGATE_TOOL)
    {
        return super::tool_executor_delegate_batch::run_delegate_only_tools(
            on_event,
            messages,
            tool_calls,
            working_dir,
            session_id,
            request_id,
            cancel,
            plan_mode_active,
            tool_call_ids,
            compression,
        )
        .await;
    }

    if mode == "manual" {
        run_sequential(
            on_event,
            messages,
            tool_calls,
            working_dir,
            session_id,
            request_id,
            cancel,
            write_guard,
            plan_mode_active,
            tool_call_ids,
            compression,
        )
        .await
    } else {
        run_with_parallel_reads(
            on_event,
            messages,
            tool_calls,
            working_dir,
            mode,
            cancel,
            write_guard,
            eager_results.as_mut(),
            session_id,
            request_id,
            plan_mode_active,
            tool_call_ids,
            compression,
            can_use_delegate_batch,
        )
        .await
    }
}
