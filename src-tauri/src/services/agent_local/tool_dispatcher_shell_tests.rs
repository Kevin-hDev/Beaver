use super::super::types_tools::ShellOutput;

fn output(stdout: &str, stderr: &str, exit_code: i32, running: bool) -> ShellOutput {
    ShellOutput {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
        running,
        timed_out: false,
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
