use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(crate) fn enrich(result: ToolResult, tool_name: &str) -> ToolResult {
    if !result.is_error {
        return result;
    }
    let code = result.error.as_ref().map(|error| error.code.as_ref());
    let generic_error = code == Some("tool_execution_failed");
    let shell_exit = is_shell_tool(tool_name) && code == Some("shell_exit_nonzero");
    if !generic_error && !shell_exit {
        return result;
    }
    let lower = result.content.to_lowercase();
    if tool_name == "edit_file" && lower.contains("non trouv") {
        return result.with_error_info(
            "edit_match_not_found",
            ToolErrorCategory::NotFound,
            false,
        );
    }
    if tool_name == "edit_file" && lower.contains("fois") {
        return result
            .with_error_info("edit_match_ambiguous", ToolErrorCategory::Conflict, false)
            .with_error_hint(
                "old_string apparaît plusieurs fois. Ajouter des lignes avant ou après pour rendre la correspondance unique.",
            );
    }
    if is_shell_tool(tool_name) && lower.contains("command not found") {
        return result
            .with_error_info("shell_command_not_found", ToolErrorCategory::NotFound, false)
            .with_error_hint(
                "Vérifier le nom de la commande ou installer le programme nécessaire.",
            );
    }
    let permission_denied = lower.contains("permission denied")
        || lower.contains("access denied")
        || lower.contains("accès refusé")
        || (generic_error
            && contains_any(
                &lower,
                &[
                    "permission refusée",
                    "interdit",
                    "non autoris",
                    "désactivé",
                    "désactivée",
                ],
            ));
    if permission_denied {
        return result.with_error_info(
            "resource_permission_denied",
            ToolErrorCategory::Permission,
            false,
        );
    }
    let not_found = lower.contains("no such file")
        || lower.contains("cannot find path")
        || (generic_error
            && contains_any(
                &lower,
                &["introuvable", "n'existe pas", "not found"],
            ));
    if not_found {
        return result.with_error_info(
            "resource_not_found",
            ToolErrorCategory::NotFound,
            false,
        );
    }
    if shell_exit {
        return result;
    }
    if contains_any(&lower, &["annulé", "annule", "cancelled", "canceled"]) {
        return result.into_cancelled();
    }
    if lower.contains("timeout") || lower.contains("timed out") {
        return result
            .with_error_info(
                "tool_timeout",
                ToolErrorCategory::Timeout,
                safe_to_retry(tool_name),
            )
            .with_error_hint(
                "Vérifier l'état produit avant une nouvelle tentative, puis corriger ou fractionner l'action.",
            );
    }
    if result_is_invalid(&lower) {
        return result.with_error_info(
            "tool_result_invalid",
            ToolErrorCategory::Internal,
            safe_to_retry(tool_name),
        );
    }
    if input_is_invalid(&lower) {
        return result
            .with_error_info(
                "invalid_tool_input",
                ToolErrorCategory::Validation,
                false,
            )
            .with_error_hint("Corriger les arguments de l'outil avant de le relancer.");
    }
    if contains_any(
        &lower,
        &[
            "déjà ",
            "already ",
            "limite atteinte",
            "limite d'",
            "pleine",
            "encore actif",
            "archivé",
        ],
    ) {
        return result.with_error_info(
            "tool_state_conflict",
            ToolErrorCategory::Conflict,
            false,
        );
    }
    if lower.contains("erreur interne")
        || lower.contains("internal error")
        || lower.contains("non initialis")
    {
        return result.with_error_info(
            "tool_internal_error",
            ToolErrorCategory::Internal,
            safe_to_retry(tool_name),
        );
    }
    if lower.contains("indisponible") || lower.contains("unavailable") {
        return result.with_error_info(
            "tool_unavailable",
            ToolErrorCategory::Unavailable,
            safe_to_retry(tool_name),
        );
    }
    result
}

fn safe_to_retry(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "read_file"
            | "grep"
            | "glob"
            | "list_dir"
            | "web_search"
            | "web_fetch"
            | "read_spreadsheet"
            | "read_document"
            | "read_image"
            | "forecast_read"
            | "agent_diagnostics"
            | "todo_history"
            | "list_subagents"
            | "get_subagent"
            | "inspect_subagent_changes"
    )
}

fn is_shell_tool(tool_name: &str) -> bool {
    matches!(tool_name, "bash" | "bash_write")
}

fn input_is_invalid(message: &str) -> bool {
    contains_any(
        message,
        &[
            "paramètre",
            "parametres",
            "paramètres",
            "argument",
            "invalide",
            "invalid",
            "non supporté",
            "non supportée",
            "unsupported",
            "trop long",
            "trop grande",
            "hors limites",
            "outil inconnu",
            "opération inconnue",
            "mode invalide",
            "doit être",
        ],
    )
}

fn result_is_invalid(message: &str) -> bool {
    contains_any(message, &["résultat", "result", "réponse", "response"])
        && contains_any(
            message,
            &["invalide", "invalid", "indisponible", "unavailable", "vide"],
        )
}

fn contains_any(message: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| message.contains(pattern))
}

pub(crate) fn extension_unavailable() -> ToolResult {
    ToolResult::error(
        "Extension indisponible.",
        "extension_unavailable",
        ToolErrorCategory::Unavailable,
        true,
    )
}

pub(crate) fn skill_load(message: String) -> ToolResult {
    if message.contains("Identifiant") {
        ToolResult::error(
            message,
            "invalid_skill_id",
            ToolErrorCategory::Validation,
            false,
        )
    } else {
        ToolResult::error(
            message,
            "skill_unavailable",
            ToolErrorCategory::Unavailable,
            true,
        )
    }
}
