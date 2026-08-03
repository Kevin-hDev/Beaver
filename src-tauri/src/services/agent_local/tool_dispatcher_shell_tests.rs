use super::super::types_tools::ShellOutput;
use super::super::tool_result_contract::ToolResultStatus;
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn output(stdout: &str, stderr: &str, exit_code: i32, running: bool) -> ShellOutput {
    ShellOutput {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
        running,
        stopped: false,
        cancelled: false,
        blocked: false,
        timed_out: false,
        tracking_incomplete: false,
        output_truncated: false,
        output_incomplete: false,
        sandbox_warning: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }
}

#[test]
fn keeps_standard_error_identifiable() {
    let result = super::to_tool_result(output("normal", "warning", 0, false));

    assert_eq!(result.content, "normal\n\n[stderr]\nwarning");
    assert!(!result.is_error);
}

#[test]
fn running_process_is_not_reported_as_a_failure() {
    let result = super::to_tool_result(output("still running", "", -1, true));

    assert!(!result.is_error);
    assert_eq!(result.status, ToolResultStatus::Running);
}

#[test]
fn requested_stop_is_not_reported_as_a_failure() {
    let mut shell_output = output("Processus arrêté.", "", -1, false);
    shell_output.stopped = true;

    let result = super::to_tool_result(shell_output);

    assert_eq!(result.content, "Processus arrêté.");
    assert!(!result.is_error);
    assert_eq!(result.status, ToolResultStatus::Stopped);
}

#[test]
fn tracking_warning_is_separate_from_standard_output() {
    let mut shell_output = output("small", "", 0, false);
    shell_output.tracking_incomplete = true;

    let result = super::to_tool_result(shell_output);

    assert_eq!(result.content, "small");
    assert_eq!(result.status, ToolResultStatus::Partial);
    assert_eq!(result.warnings.len(), 1);
    assert!(!result.is_error);
}

#[test]
fn detached_output_warning_does_not_turn_success_into_failure() {
    let mut shell_output = output("done", "", 0, false);
    shell_output.output_incomplete = true;

    let result = super::to_tool_result(shell_output);

    assert_eq!(result.content, "done");
    assert!(result.warnings[0].contains("La commande a réussi"));
    assert!(!result.is_error);
}

#[test]
fn nonzero_exit_keeps_output_and_exposes_the_exit_code() {
    let result = super::to_tool_result(output("done", "", 7, false));

    assert!(result.is_error);
    assert_eq!(result.error.as_ref().unwrap().code.as_ref(), "shell_exit_nonzero");
    assert_eq!(result.content, "done\n\n[Code de sortie: 7]");
}

#[test]
fn unknown_runtime_failure_does_not_encourage_a_blind_retry() {
    let result = super::to_tool_result(output("partial work", "", -1, false));

    let error = result.error.expect("structured error");
    assert_eq!(error.code.as_ref(), "shell_execution_failed");
    assert!(!error.retryable);
    assert!(error.hint.unwrap().contains("Vérifier l'état"));
}

#[test]
fn cancellation_has_a_distinct_status() {
    let mut shell_output = output("", "Commande annulée.", -1, false);
    shell_output.cancelled = true;

    let result = super::to_tool_result(shell_output);

    assert_eq!(result.status, ToolResultStatus::Cancelled);
    assert_eq!(result.error.as_ref().unwrap().code.as_ref(), "tool_cancelled");
}

#[test]
fn timed_out_command_requires_state_verification() {
    let mut shell_output = output("partial work", "Timeout.", -1, false);
    shell_output.timed_out = true;

    let result = super::to_tool_result(shell_output);
    let error = result.error.unwrap();

    assert_eq!(error.code.as_ref(), "shell_timeout");
    assert!(!error.retryable);
    assert!(error.hint.is_some());
}

#[test]
fn policy_block_is_not_reported_as_a_runtime_crash() {
    let mut shell_output = output("", "Commande bloquée.", -1, false);
    shell_output.blocked = true;

    let result = super::to_tool_result(shell_output);

    assert_eq!(
        result.error.as_ref().unwrap().code.as_ref(),
        "shell_command_blocked"
    );
    assert_eq!(
        result.error.unwrap().category,
        super::super::tool_result_contract::ToolErrorCategory::Permission
    );
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn stopped_session_returns_its_exact_command_as_display_summary() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let command = "sleep 30";
    let started = super::execute_command(
        &json!({"command": command, "yield_time_ms": 250}),
        dir.path(),
        &owner,
        CancellationToken::new(),
        None,
        None,
    )
    .await
    .expect("start process");
    let process_id = started
        .stdout
        .split("session_id=")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("process id");

    let result = super::dispatch(
        "bash_control",
        &json!({"session_id": process_id, "stop": true, "yield_time_ms": 1_000}),
        dir.path(),
        &owner,
        CancellationToken::new(),
        None,
        None,
    )
    .await;

    assert!(!result.is_error);
    assert_eq!(result.content, "Processus arrêté.");
    assert_eq!(result.display_summary(), Some(command));
}
