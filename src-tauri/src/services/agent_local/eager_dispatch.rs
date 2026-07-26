use crate::services::agent_local::tool_dispatcher;
use crate::services::agent_local::tool_executor_read_only::is_read_only;
use crate::services::agent_local::tool_hooks::{run_pre_hooks, PreHookDecision};
use crate::services::agent_local::types_tools::ToolResult;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

const MAX_EAGER: usize = 10;

pub async fn collect_eager_results(
    mut rx: mpsc::UnboundedReceiver<(usize, String, serde_json::Value)>,
    working_dir: PathBuf,
    session_id: String,
    request_id: String,
) -> HashMap<usize, ToolResult> {
    let mut handles: Vec<tokio::task::JoinHandle<(usize, ToolResult)>> = Vec::new();
    let mut count = 0;

    while let Some((idx, name, args)) = rx.recv().await {
        if !is_read_only(&name) || count >= MAX_EAGER {
            continue;
        }
        if matches!(run_pre_hooks(&name, &args), PreHookDecision::Deny(_)) {
            continue;
        }
        count += 1;
        let wd = working_dir.clone();
        let sid = session_id.clone();
        let rid = request_id.clone();
        let handle = tokio::spawn(async move {
            let arg_summary = super::diagnostic_args::summarize(&name, &args, &wd);
            super::stream_diagnostics::record_tool(
                &sid,
                &rid,
                &name,
                "started",
                arg_summary.clone(),
                false,
            )
            .await;
            let result = tool_dispatcher::dispatch(
                &name,
                &args,
                &wd,
                &sid,
                tokio_util::sync::CancellationToken::new(),
            )
            .await;
            super::stream_diagnostics::record_tool(
                &sid,
                &rid,
                &name,
                "completed",
                arg_summary,
                result.is_error,
            )
            .await;
            (idx, result)
        });
        handles.push(handle);
    }

    let mut results = HashMap::new();
    for handle in handles {
        if let Ok((idx, result)) = handle.await {
            results.insert(idx, result);
        }
    }
    results
}
