use super::super::types_tools::ShellOutput;
use serde_json::json;
use tokio_util::sync::CancellationToken;

fn output(stdout: &str, stderr: &str, exit_code: i32, running: bool) -> ShellOutput {
    ShellOutput {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
        running,
        stopped: false,
        timed_out: false,
        tracking_incomplete: false,
        output_incomplete: false,
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
}

#[test]
fn requested_stop_is_not_reported_as_a_failure() {
    let mut shell_output = output("Processus arrêté.", "", -1, false);
    shell_output.stopped = true;

    let result = super::to_tool_result(shell_output);

    assert_eq!(result.content, "Processus arrêté.");
    assert!(!result.is_error);
}

#[test]
fn tracking_warning_is_separate_from_standard_output() {
    let mut shell_output = output("small", "", 0, false);
    shell_output.tracking_incomplete = true;

    let result = super::to_tool_result(shell_output);

    assert!(result.content.starts_with("small\n\n[Avertissement Beaver]"));
    assert!(!result.is_error);
}

#[test]
fn detached_output_warning_does_not_turn_success_into_failure() {
    let mut shell_output = output("done", "", 0, false);
    shell_output.output_incomplete = true;

    let result = super::to_tool_result(shell_output);

    assert!(result.content.contains("La commande a réussi"));
    assert!(!result.is_error);
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
        "bash_write",
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
    assert_eq!(result.display_summary.as_deref(), Some(command));
}
