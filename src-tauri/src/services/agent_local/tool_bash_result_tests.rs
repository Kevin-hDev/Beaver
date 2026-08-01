use super::*;

#[test]
fn timeout_preserves_process_stderr_and_adds_the_completion_cause() {
    let mut stdout = String::new();
    let mut stderr = "compiler diagnostic".to_string();

    let code = completion_exit_code(
        Some(CompletionKind::TimedOut),
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, -1);
    assert!(stderr.starts_with("compiler diagnostic"));
    assert!(stderr.contains("Timeout de la commande atteint."));
}

#[test]
fn cancellation_and_internal_failure_keep_distinct_messages() {
    let mut stdout = String::new();
    let mut cancelled = String::new();
    let mut failed = String::new();

    completion_exit_code(
        Some(CompletionKind::Cancelled),
        &mut stdout,
        &mut cancelled,
    );
    completion_exit_code(Some(CompletionKind::Failed), &mut stdout, &mut failed);

    assert!(cancelled.contains("Commande annulee."));
    assert!(failed.contains("Execution shell interrompue."));
}
