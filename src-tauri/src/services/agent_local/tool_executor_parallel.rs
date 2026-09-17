#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_hooks::{run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use super::tool_execution_outcome::ToolExecutionOutcome;
use super::tool_executor_compression::ToolCompression;
use super::tool_executor_parallel_batch::{flush_read_batch, BatchEntry};
use super::tool_executor_parallel_finalize::{
    publish_results, resolve_and_record_diagnostics, IndexedResult,
};

pub async fn run_with_parallel_reads(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    mode: &str,
    cancel: CancellationToken,
    write_guard: &mut WriteGuard,
    mut eager_results: Option<&mut HashMap<usize, ToolResult>>,
    session_id: &str,
    request_id: &str,
    plan_mode_active: bool,
    tool_call_ids: &[String],
    compression: Option<&ToolCompression<'_>>,
    can_use_delegate_batch: bool,
    interception: &crate::services::extensions::InterceptionSnapshot,
) -> ToolExecutionOutcome {
    let mut read_batch: Vec<BatchEntry> = Vec::new();
    let mut indexed_results: Vec<IndexedResult<'_>> = vec![None; tool_calls.len()];
    let mut emitted_results = vec![false; tool_calls.len()];
    let mut diagnostics_already_completed = vec![false; tool_calls.len()];
    let mut i = 0;
    while i <= tool_calls.len() {
        let is_last = i == tool_calls.len();
        let is_write =
            !is_last && !super::tool_executor_read_only::is_read_only(tool_calls[i].0.as_str());
        if is_last || is_write {
            if !read_batch.is_empty() {
                let batch: Vec<_> = std::mem::take(&mut read_batch);
                flush_read_batch(
                    on_event,
                    &batch,
                    &mut indexed_results,
                    working_dir,
                    &cancel,
                    write_guard,
                    &mut eager_results,
                    session_id,
                    request_id,
                    mode,
                    plan_mode_active,
                )
                .await;
            }
            if is_last {
                break;
            }
            if mode != "chat"
                && can_use_delegate_batch
                && tool_calls[i].0 == super::tool_executor_delegate_batch::DELEGATE_TOOL
            {
                let mut delegate_items = Vec::new();
                while i < tool_calls.len()
                    && tool_calls[i].0 == super::tool_executor_delegate_batch::DELEGATE_TOOL
                {
                    delegate_items.push(super::tool_executor_delegate_batch::DelegateBatchItem {
                        index: i,
                        args: &tool_calls[i].1,
                    });
                    i += 1;
                }
                let results = super::tool_executor_delegate_batch::run_delegate_batch(
                    on_event,
                    &delegate_items,
                    session_id,
                    request_id,
                    working_dir,
                    cancel.clone(),
                    plan_mode_active,
                    tool_call_ids,
                    mode,
                    interception,
                )
                .await;
                for output in results {
                    emitted_results[output.index] = true;
                    diagnostics_already_completed[output.index] = true;
                    indexed_results[output.index] = Some((
                        super::tool_executor_delegate_batch::DELEGATE_TOOL,
                        output.result,
                    ));
                }
                continue;
            }
            let (name, args) = &tool_calls[i];
            let tr = super::tool_executor_parallel_write::execute_tracked_write(
                on_event,
                name,
                args,
                super::tool_executor_parallel_write::WriteExecContext {
                    working_dir,
                    mode,
                    write_guard,
                    session_id,
                    request_id,
                    cancel: cancel.clone(),
                    plan_mode_active,
                    tool_call_index: i,
                    tool_call_id: tool_call_ids.get(i).map(String::as_str),
                    interception,
                },
            )
            .await;
            indexed_results[i] = Some((name.as_str(), tr));
            i += 1;
        } else {
            let (name, args) = &tool_calls[i];
            let plan_check = super::tool_plan_guard::ensure_allowed_for_session(
                name,
                args,
                session_id,
                plan_mode_active,
            )
            .await;
            if let Err(msg) = plan_check {
                let tr = super::tool_executor_plan::denied_from_args(
                    session_id,
                    request_id,
                    name,
                    msg,
                    args,
                    working_dir,
                )
                .await;
                indexed_results[i] = Some((name.as_str(), tr));
                diagnostics_already_completed[i] = true;
                i += 1;
                continue;
            }
            match run_pre_hooks(name, args) {
                PreHookDecision::Deny(msg) => {
                    let tr = super::tool_executor_errors::permission(msg, "tool_hook_denied");
                    indexed_results[i] = Some((name.as_str(), tr));
                }
                PreHookDecision::Allow => {
                    match crate::services::extensions::before_tool_effect(
                        interception,
                        name,
                        args,
                        working_dir,
                        mode,
                        &cancel,
                    )
                    .await
                    {
                        Ok(()) => read_batch.push(BatchEntry {
                            global_idx: i,
                            name: name.as_str(),
                            effective_args: args,
                            tool_call_id: tool_call_ids.get(i).map(String::as_str),
                        }),
                        Err(result) => indexed_results[i] = Some((name.as_str(), result)),
                    }
                }
            }
            i += 1;
        }
    }

    resolve_and_record_diagnostics(
        &mut indexed_results,
        tool_calls,
        working_dir,
        &cancel,
        session_id,
        request_id,
        &diagnostics_already_completed,
    )
    .await;

    publish_results(
        on_event,
        messages,
        tool_calls,
        working_dir,
        indexed_results,
        &emitted_results,
        tool_call_ids,
        compression,
    )
    .await
}
