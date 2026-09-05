use super::types_tools::ToolResult;
use tokio_util::sync::CancellationToken;

pub(crate) async fn resolve_for_result(
    result: ToolResult,
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
) -> ToolResult {
    if result.is_error
        || (result.pending_artifacts().is_empty() && !result.has_pending_extension_resource())
    {
        return result;
    }
    let mut results = vec![Some(result)];
    super::tool_pending_artifact_batch::resolve_batch(&mut results, working_dir, cancel).await;
    results.pop().flatten().unwrap_or_else(invalid_result)
}

#[cfg(test)]
pub(crate) fn resolve_with_key(
    result: ToolResult,
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
    key: &[u8],
) -> ToolResult {
    let mut budget = super::tool_pending_artifact_batch::BatchArtifactBudget::new();
    super::tool_pending_artifact_batch::resolve_with_budget(
        vec![Some(result)],
        working_dir,
        cancel,
        &mut budget,
        Some(key),
    )
    .pop()
    .flatten()
    .unwrap_or_else(invalid_result)
}

fn invalid_result() -> ToolResult {
    ToolResult::unavailable(
        crate::services::extensions::error_codes::RESULT_INVALID,
        "Résultat d'extension indisponible.",
        false,
    )
}

#[cfg(test)]
#[path = "tool_pending_artifacts_tests.rs"]
mod tests;
