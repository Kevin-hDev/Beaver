use super::{action_result, change_failure, id_arg, APPLY_ERROR};
use crate::services::agent_local::tool_result_contract::{
    ToolErrorCategory, ToolResultStatus,
};
use crate::services::agent_local::types_subagent_change::SubagentChangeMeta;

#[test]
fn apply_failure_explains_that_manual_resolution_requires_cleanup() {
    let result = action_result(
        Err::<SubagentChangeMeta, _>("conflit".to_string()),
        APPLY_ERROR,
    );

    assert!(result.is_error);
    assert!(result.content.contains("reste non résolu"));
    assert!(result.content.contains("discard_subagent_changes"));
}

#[test]
fn change_failures_preserve_the_actionable_cause() {
    let conflict = change_failure("Le changement entre en conflit".into(), APPLY_ERROR, false);
    let dirty = change_failure("Dépôt parent non prêt".into(), APPLY_ERROR, false);
    let unknown = change_failure("échec git inattendu".into(), APPLY_ERROR, false);

    assert_eq!(conflict.status, ToolResultStatus::Error);
    assert_eq!(conflict.error.unwrap().code.as_ref(), "subagent_change_conflict");
    assert_eq!(dirty.error.unwrap().category, ToolErrorCategory::Conflict);
    assert!(!unknown.error.unwrap().retryable);
}

#[test]
fn unavailable_dependency_is_retryable_only_for_inspection() {
    let inspect = change_failure("Git indisponible".into(), "inspection impossible", true);
    let apply = change_failure("Git indisponible".into(), APPLY_ERROR, false);

    assert!(inspect.error.unwrap().retryable);
    let apply_error = apply.error.unwrap();
    assert!(!apply_error.retryable);
    assert!(apply_error.hint.is_some());
}

#[test]
fn invalid_change_ids_are_validation_errors() {
    let args = serde_json::json!({"subagent_id": "not-a-uuid"});
    let result = id_arg(&args, "subagent_id").unwrap_err();

    assert_eq!(result.error.unwrap().category, ToolErrorCategory::Validation);
}
