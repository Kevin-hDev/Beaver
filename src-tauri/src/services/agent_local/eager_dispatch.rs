use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_executor_read_only::is_read_only;
use crate::services::agent_local::tool_hooks::{run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_tools::ToolResult;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const MAX_EAGER: usize = 10;

pub async fn collect_eager_results(
    rx: mpsc::UnboundedReceiver<(usize, String, serde_json::Value)>,
    working_dir: PathBuf,
    session_id: String,
    request_id: String,
    chat_mode: bool,
    cancel: CancellationToken,
) -> HashMap<usize, ToolResult> {
    collect_eager_results_with(
        rx,
        working_dir,
        session_id,
        request_id,
        chat_mode,
        cancel,
        |name, args, working_dir, session_id, cancel, chat_mode| async move {
            tool_dispatcher::dispatch_for_mode(
                &name,
                &args,
                &working_dir,
                &session_id,
                cancel,
                chat_mode,
            )
            .await
        },
    )
    .await
}

async fn collect_eager_results_with<Dispatch, DispatchFuture>(
    mut rx: mpsc::UnboundedReceiver<(usize, String, serde_json::Value)>,
    working_dir: PathBuf,
    session_id: String,
    request_id: String,
    chat_mode: bool,
    cancel: CancellationToken,
    dispatch: Dispatch,
) -> HashMap<usize, ToolResult>
where
    Dispatch: Fn(String, serde_json::Value, PathBuf, String, CancellationToken, bool) -> DispatchFuture
        + Clone
        + Send
        + 'static,
    DispatchFuture: Future<Output = ToolResult> + Send + 'static,
{
    let mut tasks = tokio::task::JoinSet::new();
    let mut count = 0;

    while let Some((idx, name, args)) = rx.recv().await {
        if !is_read_only(&name) || extension_requires_confirmation(&name) || count >= MAX_EAGER {
            continue;
        }
        if matches!(run_pre_hooks(&name, &args), PreHookDecision::Deny(_)) {
            continue;
        }
        count += 1;
        let wd = working_dir.clone();
        let sid = session_id.clone();
        let rid = request_id.clone();
        let task_cancel = cancel.clone();
        let task_dispatch = dispatch.clone();
        tasks.spawn(async move {
            let arg_summary = super::diagnostic_args::summarize(&name, &args, &wd);
            super::stream_diagnostics::record_tool(
                &sid,
                &rid,
                &name,
                "started",
                arg_summary.clone(),
                None,
            )
            .await;
            let result =
                task_dispatch(name.clone(), args, wd, sid.clone(), task_cancel, chat_mode).await;
            super::stream_diagnostics::record_tool(
                &sid,
                &rid,
                &name,
                "completed",
                arg_summary,
                Some(&result),
            )
            .await;
            (idx, result)
        });
    }

    let mut results = HashMap::new();
    while let Some(outcome) = tasks.join_next().await {
        if let Ok((idx, result)) = outcome {
            results.insert(idx, result);
        }
    }
    results
}

fn extension_requires_confirmation(name: &str) -> bool {
    crate::services::extensions::indexed_tool(name).is_some_and(|indexed| {
        super::permission_policy::extension_effect_policy(indexed.tool.effect)
            .requires_confirmation
    })
}

#[cfg(test)]
#[path = "eager_dispatch_tests.rs"]
mod tests;
