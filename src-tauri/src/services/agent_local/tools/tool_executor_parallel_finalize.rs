use tokio_util::sync::CancellationToken;

use super::tool_pending_artifact_batch::resolve_batch;

pub(super) type IndexedResult<'a> = Option<(
    &'a str,
    crate::services::agent_local::types_tools::ToolResult,
)>;

pub(super) async fn publish_results(
    on_event: &super::stream_events::AgentEventEmitter,
    messages: &mut Vec<super::types_ollama::ChatMessage>,
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    indexed_results: Vec<IndexedResult<'_>>,
    emitted_results: &[bool],
    tool_call_ids: &[String],
) -> super::tool_execution_outcome::ToolExecutionOutcome {
    let tool_id = |idx| tool_call_ids.get(idx).map(String::as_str);
    let mut outcome = super::tool_execution_outcome::ToolExecutionOutcome::default();
    for (idx, slot) in indexed_results.into_iter().enumerate() {
        if let Some((name, mut result)) = slot {
            let artifacts = result.take_ephemeral_artifacts();
            let artifact_records = outcome
                .retain_artifacts(idx, tool_id(idx), artifacts)
                .unwrap_or_else(|()| {
                    result = crate::services::agent_local::types_tools::ToolResult::error(
                        "Résultat d'extension indisponible.",
                        crate::services::extensions::error_codes::RESULT_TOO_LARGE,
                        crate::services::agent_local::tool_result_contract::ToolErrorCategory::Unavailable,
                        false,
                    );
                    Vec::new()
                });
            let resolved_path = super::tool_executor_helpers::resolve_tool_path(
                name,
                &tool_calls[idx].1,
                working_dir,
            );
            let follow_up = if emitted_results[idx] {
                super::tool_executor_helpers::push_tool_message(
                    messages,
                    name,
                    result,
                    tool_id(idx),
                )
            } else {
                super::tool_executor_helpers::push_tool_result(
                    on_event,
                    messages,
                    name,
                    result,
                    idx,
                    tool_id(idx),
                    resolved_path,
                    artifact_records,
                )
            };
            outcome.record(follow_up);
        }
    }
    outcome
}

pub(super) async fn resolve_and_record_diagnostics<'a>(
    indexed_results: &mut [IndexedResult<'a>],
    tool_calls: &'a [(String, serde_json::Value)],
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    session_id: &str,
    request_id: &str,
    diagnostics_already_completed: &[bool],
) {
    let (names, mut results) = take_results(indexed_results);
    resolve_batch(&mut results, working_dir, cancel).await;
    restore_results(indexed_results, names, results);
    record_diagnostics(
        indexed_results,
        tool_calls,
        working_dir,
        session_id,
        request_id,
        diagnostics_already_completed,
    )
    .await;
}

fn take_results<'a>(
    indexed_results: &mut [IndexedResult<'a>],
) -> (
    Vec<Option<&'a str>>,
    Vec<Option<crate::services::agent_local::types_tools::ToolResult>>,
) {
    indexed_results
        .iter_mut()
        .map(|slot| match slot.take() {
            Some((name, result)) => (Some(name), Some(result)),
            None => (None, None),
        })
        .unzip()
}

fn restore_results<'a>(
    indexed_results: &mut [IndexedResult<'a>],
    names: Vec<Option<&'a str>>,
    results: Vec<Option<crate::services::agent_local::types_tools::ToolResult>>,
) {
    for ((slot, name), result) in indexed_results.iter_mut().zip(names).zip(results) {
        let (Some(name), Some(result)) = (name, result) else {
            continue;
        };
        *slot = Some((name, result));
    }
}

async fn record_diagnostics(
    indexed_results: &[IndexedResult<'_>],
    tool_calls: &[(String, serde_json::Value)],
    working_dir: &std::path::Path,
    session_id: &str,
    request_id: &str,
    diagnostics_already_completed: &[bool],
) {
    for (index, slot) in indexed_results.iter().enumerate() {
        let Some((name, result)) = slot else { continue };
        if diagnostics_already_completed[index] {
            continue;
        }
        let summary = super::diagnostic_args::summarize(name, &tool_calls[index].1, working_dir);
        super::tool_executor_diagnostics::completed(session_id, request_id, name, summary, result)
            .await;
    }
}

#[cfg(test)]
pub(super) fn resolve_with_test_key(
    indexed_results: &mut [IndexedResult<'_>],
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    key: &[u8],
) {
    let (names, results) = take_results(indexed_results);
    let mut budget = super::tool_pending_artifact_batch::BatchArtifactBudget::new();
    let results = super::tool_pending_artifact_batch::resolve_with_budget(
        results,
        working_dir,
        cancel,
        &mut budget,
        Some(key),
    );
    restore_results(indexed_results, names, results);
}

#[cfg(test)]
#[path = "tool_executor_parallel_finalize_tests.rs"]
mod tests;
