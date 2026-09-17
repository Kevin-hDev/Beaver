use super::types_tools::ToolResult;
use std::path::Path;

pub async fn finalize(
    result: ToolResult,
    tool_name: &str,
    session_id: &str,
    working_dir: &Path,
) -> ToolResult {
    let mut result = super::tool_dispatcher_error::enrich(result, tool_name);
    if let Some((total, stored)) = result.bound_file_changes() {
        result = result.with_warning(format!(
            "Le détail des fichiers modifiés a été réduit à un échantillon : {stored} sur {total}."
        ));
    }
    if let Some((total, stored)) = result.bound_affected_paths() {
        result = result.with_warning(format!(
            "La liste des chemins modifiés a été réduite à un échantillon : {stored} sur {total}."
        ));
    }
    let result = super::tool_result_truncate::truncate_result(result, tool_name, session_id).await;
    super::tool_workspace_notice::append(result, working_dir)
}
