use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(super) fn from_message(message: String) -> ToolResult {
    let lower = message.to_lowercase();
    if lower.contains("annul") || lower.contains("cancel") {
        return ToolResult::cancelled(message);
    }
    if lower.contains("délai d'écriture") || lower.contains("delai d'ecriture") {
        return ToolResult::error(
            message,
            "shell_input_timeout",
            ToolErrorCategory::Timeout,
            false,
        );
    }
    if lower.contains("timeout") || lower.contains("délai") {
        return ToolResult::error(
            message,
            "shell_timeout",
            ToolErrorCategory::Timeout,
            false,
        )
        .with_error_hint(
            "Vérifier l'état du projet avant de relancer : la commande a pu effectuer une partie de son travail.",
        );
    }
    if lower.contains("session shell introuvable") {
        return ToolResult::error(
            message,
            "shell_session_not_found",
            ToolErrorCategory::NotFound,
            false,
        );
    }
    if lower.contains("trop de processus shell actifs") {
        return ToolResult::error(
            message,
            "shell_session_limit",
            ToolErrorCategory::Conflict,
            false,
        )
        .with_error_hint(
            "Vérifier l'état du projet avant de relancer : le processus a pu démarrer brièvement.",
        );
    }
    if lower.contains("workdir bash") {
        let category = if lower.contains("inaccessible") {
            ToolErrorCategory::NotFound
        } else {
            ToolErrorCategory::Validation
        };
        return ToolResult::error(message, "shell_workdir_invalid", category, false);
    }
    if lower.contains("shell utilisateur indisponible") {
        return ToolResult::error(
            message,
            "shell_unavailable",
            ToolErrorCategory::Unavailable,
            false,
        );
    }
    if lower.contains("commande d'exploration indisponible") {
        return ToolResult::error(
            message,
            "explorer_command_unavailable",
            ToolErrorCategory::Unavailable,
            true,
        );
    }
    if lower.contains("lancement du shell refusé") {
        return ToolResult::error(
            message,
            "shell_start_denied",
            ToolErrorCategory::Permission,
            false,
        );
    }
    if lower.contains("lancement du shell impossible") {
        return ToolResult::error(
            message,
            "shell_start_failed",
            ToolErrorCategory::Execution,
            true,
        );
    }
    if lower.contains("sortie shell indisponible") {
        return ToolResult::error(
            message,
            "shell_output_failed",
            ToolErrorCategory::Internal,
            false,
        )
        .with_error_hint(
            "Vérifier l'état du projet avant de relancer : la commande a pu être exécutée.",
        );
    }
    if lower.contains("commande shell invalide") || lower.contains("entree shell invalide") {
        return ToolResult::error(
            message,
            "invalid_shell_input",
            ToolErrorCategory::Validation,
            false,
        );
    }
    if lower.contains("processus shell termine") {
        return ToolResult::error(
            message,
            "shell_process_exited",
            ToolErrorCategory::Conflict,
            false,
        );
    }
    if lower.contains("ecriture vers le shell impossible") {
        return ToolResult::error(
            message,
            "shell_input_failed",
            ToolErrorCategory::Execution,
            false,
        );
    }
    ToolResult::error(
        message,
        "shell_dispatch_failed",
        ToolErrorCategory::Execution,
        false,
    )
    .with_error_hint(
        "Vérifier l'état du projet avant de relancer : la commande a pu être exécutée.",
    )
}

#[cfg(test)]
#[path = "tool_dispatcher_shell_error_tests.rs"]
mod tests;
