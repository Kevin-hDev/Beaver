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
    dispatch_for_mode(tool_name, args, working_dir, session_id, cancel, false).await
}

pub async fn dispatch_for_mode(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
    chat_mode: bool,
) -> ToolResult {
    if chat_mode && !is_chat_tool(tool_name) {
        return ToolResult::err("Outil indisponible dans ce mode.");
    }
    let registered_dynamic =
        !chat_mode && crate::services::extensions::is_dynamic_tool(tool_name);
    let replacement = crate::services::extensions::is_replacement(tool_name);
    let active_dynamic = if registered_dynamic {
        match super::extension_session_plugins::is_tool_active(session_id, tool_name).await {
            Ok(active) => active,
            Err(_) => return ToolResult::err("Extension indisponible."),
        }
    } else {
        false
    };
    let dynamic_tool = match dynamic_route(registered_dynamic, active_dynamic, replacement) {
        Ok(dynamic) => dynamic,
        Err(message) => return ToolResult::err(message),
    };
    let enabled_by_settings = !super::tool_catalog::is_optional_tool(tool_name)
        || super::agent_settings::is_tool_enabled(tool_name).await;
    if !super::tool_availability::available(
        enabled_by_settings,
        dynamic_tool,
        replacement,
    ) {
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
    let args = match validate_arguments(dynamic_tool, tool_name, args) {
        Ok(cleaned) => cleaned,
        Err(msg) => return ToolResult::err(format!("[{tool_name}] {msg}")),
    };
    let before = super::tool_file_changes::direct_snapshot(tool_name, &args, working_dir);
    let mut result = if dynamic_tool {
        if crate::services::extensions::record_tool_invocation(tool_name).is_err() {
            eprintln!("[extensions] usage counter unavailable");
        }
        crate::services::extensions::dispatch_tool(tool_name, &args, working_dir)
            .await
            .unwrap_or_else(|| ToolResult::err("Extension indisponible."))
    } else {
        match super::memory_tool::dispatch_if_memory(tool_name, &args, working_dir, session_id).await
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
        }
    };
    if let Some(change) = before.and_then(super::tool_file_changes::direct_change) {
        if result.affected_paths.is_empty() {
            result.affected_paths.push(change.path.clone());
        }
        result.file_changes.push(change);
    }
    let result = super::tool_result_truncate::truncate_result(result, tool_name, session_id);
    let result = super::tool_workspace_notice::append(result, working_dir);
    enrich_error(result, tool_name)
}

fn dynamic_route(
    registered_dynamic: bool,
    active_dynamic: bool,
    replacement: bool,
) -> Result<bool, &'static str> {
    if active_dynamic {
        Ok(true)
    } else if registered_dynamic && !replacement {
        Err("Extension indisponible.")
    } else {
        Ok(false)
    }
}

pub fn is_chat_tool(tool_name: &str) -> bool {
    matches!(tool_name, "web_search" | "web_fetch")
}

fn validate_arguments(dynamic_tool: bool, tool_name: &str, args: &Value) -> Result<Value, String> {
    if dynamic_tool {
        crate::services::extensions::validate_arguments(tool_name, args)
    } else {
        super::tool_validate::validate(tool_name, args)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn chat_policy_has_exactly_two_native_tools() {
        assert!(is_chat_tool("web_search"));
        assert!(is_chat_tool("web_fetch"));
        assert!(!is_chat_tool("bash"));
        assert!(!is_chat_tool("search_extension_tools"));
    }

    #[test]
    fn inactive_replacements_fall_back_to_core_but_other_plugins_fail_closed() {
        assert_eq!(dynamic_route(true, false, true), Ok(false));
        assert_eq!(dynamic_route(true, true, true), Ok(true));
        assert_eq!(
            dynamic_route(true, false, false),
            Err("Extension indisponible.")
        );
    }

    #[tokio::test]
    async fn chat_rejects_an_agentic_call_before_dispatch() {
        let result = dispatch_for_mode(
            "bash",
            &json!({"command": "pwd"}),
            std::path::Path::new("."),
            "test-session",
            CancellationToken::new(),
            true,
        )
        .await;

        assert!(result.is_error);
        assert_eq!(result.content, "Outil indisponible dans ce mode.");
    }
}
