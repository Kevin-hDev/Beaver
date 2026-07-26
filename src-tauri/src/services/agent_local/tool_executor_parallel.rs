use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_hooks::{run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::agent_local::write_guard::WriteGuard;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

use super::tool_executor_compression::ToolCompression;
use super::tool_executor_helpers::{push_tool_message, push_tool_result, resolve_tool_path};
use super::tool_executor_parallel_batch::{flush_read_batch, BatchEntry};

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
) -> bool {
    let mut read_batch: Vec<BatchEntry> = Vec::new();
    let mut indexed_results: Vec<Option<(&str, ToolResult)>> = vec![None; tool_calls.len()];
    let mut emitted_results = vec![false; tool_calls.len()];
    let mut i = 0;
    while i <= tool_calls.len() {
        let is_last = i == tool_calls.len();
        let is_write =
            !is_last && !super::tool_executor_read_only::is_read_only(tool_calls[i].0.as_str());
        if is_last || is_write {
            if !read_batch.is_empty() {
                let batch: Vec<_> = std::mem::take(&mut read_batch);
                flush_read_batch(
                    &batch,
                    &mut indexed_results,
                    working_dir,
                    &cancel,
                    write_guard,
                    &mut eager_results,
                    session_id,
                    request_id,
                )
                .await;
            }
            if is_last {
                break;
            }
            if can_use_delegate_batch
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
                )
                .await;
                for output in results {
                    emitted_results[output.index] = true;
                    indexed_results[output.index] = Some((
                        super::tool_executor_delegate_batch::DELEGATE_TOOL,
                        output.result,
                    ));
                }
                continue;
            }
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
                i += 1;
                continue;
            }
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
                i += 1;
                continue;
            }
            match run_pre_hooks(name, args) {
                PreHookDecision::Deny(msg) => {
                    let tr = tool_dispatcher::enrich_error(ToolResult::err(msg), name);
                    let summary = super::diagnostic_args::summarize(name, args, working_dir);
                    super::tool_executor_diagnostics::completed(
                        session_id, request_id, name, summary, true,
                    )
                    .await;
                    indexed_results[i] = Some((name.as_str(), tr));
                }
                PreHookDecision::Allow => {
                    read_batch.push(BatchEntry {
                        global_idx: i,
                        name: name.as_str(),
                        effective_args: args,
                    });
                }
            }
            i += 1;
        }
    }

    let mut compressed = false;
    for (idx, slot) in indexed_results.into_iter().enumerate() {
        if let Some((name, tr)) = slot {
            let resolved_path = resolve_tool_path(name, &tool_calls[idx].1, working_dir);
            if emitted_results[idx] {
                push_tool_message(messages, name, tr, tool_id(tool_call_ids, idx));
            } else {
                push_tool_result(
                    on_event,
                    messages,
                    name,
                    tr,
                    idx,
                    tool_id(tool_call_ids, idx),
                    resolved_path,
                );
            }
            if let Some(compression) = compression {
                compressed |= compression.try_run(messages).await;
            }
        }
    }
    compressed
}

fn tool_id(ids: &[String], idx: usize) -> Option<&str> {
    ids.get(idx).map(String::as_str)
}
