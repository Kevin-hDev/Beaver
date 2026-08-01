use crate::services::agent_local::tool_dispatcher::enrich_error;
use crate::services::agent_local::tool_dispatcher_error::skill_load;
use crate::services::agent_local::tool_result_contract::{
    ToolErrorCategory,
    ToolResultStatus,
};
use crate::services::agent_local::types_tools::ToolResult;

#[test]
fn edit_not_found_has_a_stable_code_without_guessing_a_fix() {
    let result = enrich_error(ToolResult::err("Chaîne non trouvée"), "edit_file");
    let error = result.error.expect("structured error");

    assert_eq!(error.code.as_ref(), "edit_match_not_found");
    assert_eq!(error.category, ToolErrorCategory::NotFound);
    assert!(error.hint.is_none());
}

#[test]
fn ambiguous_edit_explains_how_to_make_the_match_unique() {
    let result = enrich_error(ToolResult::err("La chaîne apparaît 3 fois"), "edit_file");
    let error = result.error.expect("structured error");

    assert_eq!(error.code.as_ref(), "edit_match_ambiguous");
    assert_eq!(error.category, ToolErrorCategory::Conflict);
    assert!(error.hint.as_deref().unwrap().contains("old_string"));
}

#[test]
fn bash_command_not_found_is_not_reported_as_a_generic_failure() {
    let result = enrich_error(
        ToolResult::error(
            "zsh: foo: command not found\n\n[Code de sortie: 127]",
            "shell_exit_nonzero",
            ToolErrorCategory::Execution,
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
fn mutating_timeouts_require_verification_and_keep_output_separate_from_the_hint() {
    let result = enrich_error(ToolResult::err("Timeout: commande dépassée"), "bash");
    let error = result.error.expect("structured error");

    assert_eq!(result.content, "Timeout: commande dépassée");
    assert_eq!(error.code.as_ref(), "tool_timeout");
    assert_eq!(error.category, ToolErrorCategory::Timeout);
    assert!(!error.retryable);
    assert!(error.hint.is_some());
}

#[test]
fn arbitrary_shell_output_does_not_override_the_real_exit_failure() {
    let result = enrich_error(
        ToolResult::error(
            "test named timeout failed\n\n[Code de sortie: 1]",
            "shell_exit_nonzero",
            ToolErrorCategory::Execution,
            false,
        ),
        "bash",
    );

    assert_eq!(result.error.unwrap().code.as_ref(), "shell_exit_nonzero");
}

#[test]
fn successful_results_are_untouched() {
    let result = enrich_error(ToolResult::ok("ok"), "bash");

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.content, "ok");
    assert!(result.error.is_none());
}

#[test]
fn generic_validation_failures_are_actionable() {
    let result = enrich_error(ToolResult::err("Paramètre analysis_id requis"), "forecast");
    let error = result.error.expect("structured error");

    assert_eq!(error.code.as_ref(), "invalid_tool_input");
    assert_eq!(error.category, ToolErrorCategory::Validation);
    assert!(!error.retryable);
    assert!(error.hint.is_some());
}

#[test]
fn invalid_service_results_only_allow_safe_read_retries() {
    let mutating = enrich_error(
        ToolResult::err("Résultat de comparaison indisponible"),
        "forecast",
    );
    let read = enrich_error(
        ToolResult::err("Résultat de lecture indisponible"),
        "forecast_read",
    );
    let mutating_error = mutating.error.expect("structured error");
    let read_error = read.error.expect("structured error");

    assert_eq!(mutating_error.code.as_ref(), "tool_result_invalid");
    assert_eq!(mutating_error.category, ToolErrorCategory::Internal);
    assert!(!mutating_error.retryable);
    assert!(read_error.retryable);
}

#[test]
fn legacy_cancellation_text_gets_a_cancelled_status() {
    let result = enrich_error(ToolResult::err("Opération annulée"), "forecast");

    assert_eq!(result.status, ToolResultStatus::Cancelled);
    assert_eq!(result.error.unwrap().category, ToolErrorCategory::Cancelled);
}

#[test]
fn explicit_codes_are_never_overwritten_by_message_heuristics() {
    let result = enrich_error(
        ToolResult::error(
            "Paramètre invalide",
            "domain_specific_error",
            ToolErrorCategory::External,
            true,
        ),
        "extension",
    );

    assert_eq!(
        result.error.unwrap().code.as_ref(),
        "domain_specific_error"
    );
}

#[test]
fn skill_identifier_and_availability_failures_are_distinct() {
    let invalid = skill_load("Identifiant de skill invalide".to_string());
    let unavailable = skill_load("Skill indisponible".to_string());

    assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
    assert_eq!(
        unavailable.error.unwrap().category,
        ToolErrorCategory::Unavailable
    );
}
