use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::tool_hooks::run_post_hooks;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub const DELEGATE_TOOL: &str = "delegate_task";

pub struct DelegateBatchItem<'a> {
    pub index: usize,
    pub args: &'a Value,
}

pub struct DelegateBatchOutput {
    pub index: usize,
    pub result: ToolResult,
}

struct PendingOutput {
    index: usize,
    summary: Option<Value>,
    args: Value,
    pending: super::tool_dispatcher_delegate::PendingDelegate,
}

pub async fn run_delegate_batch(
    on_event: &AgentEventEmitter,
    items: &[DelegateBatchItem<'_>],
    session_id: &str,
    request_id: &str,
    working_dir: &std::path::Path,
    cancel: CancellationToken,
    plan_mode_active: bool,
) -> Vec<DelegateBatchOutput> {
    let mut outputs = Vec::new();
    let mut pending = Vec::new();

    for item in items {
        let summary = super::tool_executor_diagnostics::started(
            session_id,
            request_id,
            DELEGATE_TOOL,
            item.args,
            working_dir,
        )
        .await;
        match super::tool_executor_delegate_launch::launch(
            item.args,
            session_id,
            plan_mode_active,
            cancel.clone(),
        )
        .await
        {
            Ok(delegate) => pending.push(PendingOutput {
                index: item.index,
                summary,
                args: item.args.clone(),
                pending: delegate,
            }),
            Err(result) => {
                finish_diagnostics(session_id, request_id, summary, result.is_error).await;
                emit_result(on_event, item.index, &result);
                outputs.push(DelegateBatchOutput {
                    index: item.index,
                    result,
                });
            }
        }
    }

    let (tx, mut rx) = mpsc::unbounded_channel();
    for item in pending {
        let tx = tx.clone();
        tauri::async_runtime::spawn(async move {
            let result = run_post_hooks(DELEGATE_TOOL, &item.args, item.pending.wait().await);
            let _ = tx.send((item.index, item.summary, result));
        });
    }
    drop(tx);

    while let Some((index, summary, result)) = rx.recv().await {
        finish_diagnostics(session_id, request_id, summary, result.is_error).await;
        emit_result(on_event, index, &result);
        outputs.push(DelegateBatchOutput { index, result });
        if cancel.is_cancelled() {
            break;
        }
    }

    sort_outputs_by_index(&mut outputs);
    outputs
}

fn sort_outputs_by_index(outputs: &mut [DelegateBatchOutput]) {
    outputs.sort_by_key(|output| output.index);
}

pub async fn run_delegate_only_tools(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    tool_calls: &[(String, Value)],
    working_dir: &std::path::Path,
    session_id: &str,
    request_id: &str,
    cancel: CancellationToken,
    plan_mode_active: bool,
    tool_call_ids: &[String],
    compression: Option<&super::tool_executor_compression::ToolCompression<'_>>,
) -> bool {
    let items: Vec<_> = tool_calls
        .iter()
        .enumerate()
        .map(|(index, (_, args))| DelegateBatchItem { index, args })
        .collect();
    let outputs = run_delegate_batch(
        on_event,
        &items,
        session_id,
        request_id,
        working_dir,
        cancel,
        plan_mode_active,
    )
    .await;
    let mut compressed = false;
    for output in outputs {
        super::tool_executor_helpers::push_tool_message(
            messages,
            DELEGATE_TOOL,
            output.result,
            tool_call_ids.get(output.index).map(String::as_str),
        );
        if let Some(compression) = compression {
            compressed |= compression.try_run(messages).await;
        }
    }
    compressed
}

async fn finish_diagnostics(
    session_id: &str,
    request_id: &str,
    summary: Option<Value>,
    is_error: bool,
) {
    super::tool_executor_diagnostics::completed(
        session_id,
        request_id,
        DELEGATE_TOOL,
        summary,
        is_error,
    )
    .await;
}

fn emit_result(on_event: &AgentEventEmitter, index: usize, result: &ToolResult) {
    super::tool_executor_helpers::emit_tool_result(on_event, DELEGATE_TOOL, result, index, None);
}

#[cfg(test)]
mod tests {
    use super::{sort_outputs_by_index, DelegateBatchOutput};
    use crate::services::agent_local::types_tools::ToolResult;

    #[test]
    fn keeps_parent_tool_context_in_original_order() {
        let mut outputs = vec![
            DelegateBatchOutput {
                index: 2,
                result: ToolResult::ok("third"),
            },
            DelegateBatchOutput {
                index: 0,
                result: ToolResult::ok("first"),
            },
            DelegateBatchOutput {
                index: 1,
                result: ToolResult::ok("second"),
            },
        ];

        sort_outputs_by_index(&mut outputs);

        let contents = outputs
            .into_iter()
            .map(|output| output.result.content)
            .collect::<Vec<_>>();
        assert_eq!(contents, ["first", "second", "third"]);
    }
}
