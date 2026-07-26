use super::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn dispatch(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    if super::tool_catalog::is_optional_tool(tool_name)
        && !super::agent_settings::is_tool_enabled(tool_name).await
    {
        return ToolResult::err("Outil désactivé dans les paramètres.");
    }
    let profile = match super::subagent_tool_guard::validate_for_session(
        session_id,
        tool_name,
        args,
        working_dir,
    )
    .await
    {
        Ok(profile) => profile,
        Err(msg) => return ToolResult::err(msg),
    };
    let args = match super::tool_validate::validate(tool_name, args) {
        Ok(cleaned) => cleaned,
        Err(msg) => return ToolResult::err(format!("[{tool_name}] {msg}")),
    };
    let before = super::tool_file_changes::direct_snapshot(tool_name, &args, working_dir);
    let mut result = match super::memory_tool::dispatch_if_memory(
        tool_name,
        &args,
        working_dir,
        session_id,
    )
    .await
    {
        Some(result) => result,
        None => {
            super::tool_dispatcher::dispatch_inner(
                tool_name,
                &args,
                working_dir,
                session_id,
                cancel,
                profile,
            )
            .await
        }
    };
    if let Some(change) = before.and_then(super::tool_file_changes::direct_change) {
        if result.affected_paths.is_empty() {
            result.affected_paths.push(change.path.clone());
        }
        result.file_changes.push(change);
    }
    let result = super::tool_result_truncate::truncate_result(result, tool_name, session_id);
    enrich_error(result, tool_name)
}

pub(crate) fn enrich_error(mut result: ToolResult, tool_name: &str) -> ToolResult {
    if !result.is_error {
        return result;
    }
    let hint = match tool_name {
        "edit_file" if result.content.contains("non trouvée") => "",
        "edit_file" if result.content.contains("fois") => {
            "\n\n[HINT: old_string apparaît plusieurs fois. Ajouter plus de contexte (lignes avant/après) pour rendre la correspondance unique]"
        }
        "bash" if result.content.contains("command not found") => {
            "\n\n[HINT: Commande introuvable. Vérifier l'orthographe ou installer le paquet nécessaire]"
        }
        "bash" if result.content.contains("Timeout") => {
            "\n\n[HINT: Timeout dépassé. Augmenter le paramètre timeout ou utiliser une approche plus efficace]"
        }
        _ => "",
    };
    if !hint.is_empty() {
        result.content.push_str(hint);
    }
    result
}
