use crate::services::agent_local::tool_dispatcher::enrich_error;
use crate::services::agent_local::tool_dispatcher_error::skill_load;
use crate::services::agent_local::tool_result_contract::{
    ToolErrorCategory, ToolResultStatus,
};
use crate::services::agent_local::types_tools::ToolResult;

#[test]
fn explicit_domain_errors_are_not_reclassified_from_their_text() {
    let result = enrich_error(
        ToolResult::not_found("edit_match_not_found", "Paramètre invalide et annulé"),
        "edit_file",
    );
    let error = result.error.expect("structured error");

    assert_eq!(error.code.as_ref(), "edit_match_not_found");
    assert_eq!(error.category, ToolErrorCategory::NotFound);
}

#[test]
fn bash_command_not_found_gets_a_specific_shell_code() {
    let result = enrich_error(
        ToolResult::execution(
            "shell_exit_nonzero",
            "zsh: foo: command not found\n\n[Code de sortie: 127]",
            false,
        ),
        "bash",
    );
    let error = result.error.expect("structured error");

    assert_eq!(error.code.as_ref(), "shell_command_not_found");
    assert_eq!(error.category, ToolErrorCategory::NotFound);
    assert!(!error.retryable);
}

#[test]
fn arbitrary_shell_output_does_not_override_the_exit_failure() {
    let result = enrich_error(
        ToolResult::execution(
            "shell_exit_nonzero",
            "test named timeout failed\n\n[Code de sortie: 1]",
            false,
        ),
        "bash",
    );

    assert_eq!(result.error.unwrap().code.as_ref(), "shell_exit_nonzero");
}

#[test]
fn non_shell_command_not_found_text_is_not_reclassified() {
    let result = enrich_error(
        ToolResult::external(
            "extension_tool_error",
            "command not found",
            false,
        ),
        "extension.tool",
    );

    assert_eq!(result.error.unwrap().code.as_ref(), "extension_tool_error");
}

#[test]
fn successful_results_are_untouched() {
    let result = enrich_error(ToolResult::ok("ok"), "bash");

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.content, "ok");
    assert!(result.error.is_none());
}

#[test]
fn skill_identifier_and_availability_failures_are_distinct() {
    let invalid = skill_load(super::tool_skill_loader::SkillLoadError::InvalidId);
    let missing = skill_load(super::tool_skill_loader::SkillLoadError::NotFound);
    let unavailable = skill_load(super::tool_skill_loader::SkillLoadError::Unavailable);

    assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
    assert_eq!(missing.error.unwrap().category, ToolErrorCategory::NotFound);
    assert_eq!(
        unavailable.error.unwrap().category,
        ToolErrorCategory::Unavailable
    );
}
