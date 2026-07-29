use super::*;

#[test]
fn successful_external_write_adds_a_non_blocking_notice() {
    let workspace = tempfile::tempdir().expect("workspace");
    let external = tempfile::tempdir().expect("external");
    let changed = external.path().join("created.txt");
    std::fs::write(&changed, "ok").expect("write");
    let result = ToolResult::ok("created")
        .with_affected_paths(vec![changed.to_string_lossy().to_string()]);

    let result = append(result, workspace.path());

    assert!(!result.is_error);
    assert!(result.content.contains("WORKSPACE NOTICE"));
    assert!(result
        .content
        .contains(workspace.path().canonicalize().unwrap().to_string_lossy().as_ref()));
}

#[test]
fn write_inside_workspace_does_not_add_a_notice() {
    let workspace = tempfile::tempdir().expect("workspace");
    let changed = workspace.path().join("created.txt");
    std::fs::write(&changed, "ok").expect("write");
    let result = ToolResult::ok("created")
        .with_affected_paths(vec![changed.to_string_lossy().to_string()]);

    let result = append(result, workspace.path());

    assert_eq!(result.content, "created");
}

#[test]
fn a_failed_tool_is_not_reframed_as_workspace_drift() {
    let workspace = tempfile::tempdir().expect("workspace");
    let result = ToolResult::err("failed")
        .with_affected_paths(vec!["/outside/failed.txt".to_string()]);

    let result = append(result, workspace.path());

    assert_eq!(result.content, "failed");
}
