use super::tool_result_contract::ToolErrorCategory;
use super::types_tools::ToolResult;

pub(crate) fn enrich(result: ToolResult, tool_name: &str) -> ToolResult {
    if !result.is_error || !matches!(tool_name, "bash" | "bash_control") {
        return result;
    }
    let shell_exit = result
        .error
        .as_ref()
        .is_some_and(|error| error.code.as_ref() == "shell_exit_nonzero");
    if shell_exit && result.content.to_ascii_lowercase().contains("command not found") {
        return result
            .with_error_info(
                "shell_command_not_found",
                ToolErrorCategory::NotFound,
                false,
            )
            .with_error_hint(
                "Vérifier le nom de la commande ou installer le programme nécessaire.",
            );
    }
    result
}

pub(crate) fn skill_load(error: super::tool_skill_loader::SkillLoadError) -> ToolResult {
    match error {
        super::tool_skill_loader::SkillLoadError::InvalidId => ToolResult::validation(
            "invalid_skill_id",
            error.message(),
        ),
        super::tool_skill_loader::SkillLoadError::NotFound => ToolResult::not_found(
            "skill_not_found",
            error.message(),
        )
        .with_error_hint("Relire la liste des skills disponibles avant de choisir un autre ID."),
        super::tool_skill_loader::SkillLoadError::Unavailable => ToolResult::unavailable(
            "skill_unavailable",
            error.message(),
            true,
        ),
    }
}
