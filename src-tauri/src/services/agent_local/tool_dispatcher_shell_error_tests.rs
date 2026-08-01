use super::*;

#[test]
fn setup_errors_have_distinct_recovery_semantics() {
    let missing = from_message("Session shell introuvable.".to_string());
    let capacity = from_message("Trop de processus shell actifs.".to_string());
    let internal = from_message("Sortie shell indisponible.".to_string());

    assert_eq!(missing.error.unwrap().code.as_ref(), "shell_session_not_found");
    let capacity_error = capacity.error.as_ref().unwrap();
    let internal_error = internal.error.as_ref().unwrap();
    assert_eq!(capacity_error.category, ToolErrorCategory::Conflict);
    assert!(!capacity_error.retryable);
    assert!(capacity_error.hint.is_some());
    assert_eq!(internal_error.category, ToolErrorCategory::Internal);
    assert!(!internal_error.retryable);
    assert!(internal_error.hint.is_some());
}

#[test]
fn cancellation_timeout_and_invalid_input_are_not_conflated() {
    let cancelled = from_message("Commande annulee.".to_string());
    let timeout = from_message("Délai d'écriture dépassé.".to_string());
    let invalid = from_message("Commande shell invalide.".to_string());

    assert_eq!(cancelled.status, super::super::tool_result_contract::ToolResultStatus::Cancelled);
    assert_eq!(timeout.error.unwrap().category, ToolErrorCategory::Timeout);
    assert_eq!(invalid.error.unwrap().category, ToolErrorCategory::Validation);
}

#[test]
fn process_start_and_input_failures_keep_their_actual_stage() {
    let denied = from_message("Lancement du shell refusé.".to_string());
    let input_timeout = from_message("Délai d'écriture vers le shell dépassé.".to_string());
    let exited = from_message("Processus shell termine.".to_string());

    assert_eq!(denied.error.unwrap().code.as_ref(), "shell_start_denied");
    assert_eq!(
        input_timeout.error.unwrap().code.as_ref(),
        "shell_input_timeout"
    );
    assert_eq!(exited.error.unwrap().code.as_ref(), "shell_process_exited");
}

#[test]
fn explorer_process_setup_failure_is_not_a_user_command_exit() {
    let result = from_message("Commande d'exploration indisponible.".to_string());

    assert_eq!(
        result.error.unwrap().code.as_ref(),
        "explorer_command_unavailable"
    );
}

#[test]
fn unknown_dispatch_failure_requires_state_verification() {
    let result = from_message("Échec shell inattendu.".to_string());
    let error = result.error.as_ref().unwrap();

    assert_eq!(error.code.as_ref(), "shell_dispatch_failed");
    assert!(!error.retryable);
    assert!(error.hint.is_some());
}
