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
    assert_eq!(result.content, "created");
    assert!(result
        .warnings
        .first()
        .expect("workspace warning")
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
    assert!(result.warnings.is_empty());
}

#[test]
fn a_failed_tool_keeps_its_error_and_reports_external_changes_separately() {
    let workspace = tempfile::tempdir().expect("workspace");
    let external = tempfile::tempdir().expect("external");
    let changed = external.path().join("partial.txt");
    std::fs::write(&changed, "partial").expect("write partial result");
    let result = ToolResult::execution("test_failure", "failed", false)
        .with_affected_paths(vec![changed.to_string_lossy().to_string()]);

    let result = append(result, workspace.path());

    assert_eq!(result.content, "failed");
    assert!(result.is_error);
    assert!(result.warnings[0].contains("WORKSPACE NOTICE"));
}

#[test]
fn configured_outputs_are_part_of_the_active_workspace() {
    let workspace = tempfile::tempdir().expect("workspace");
    let outputs = tempfile::tempdir().expect("outputs");

    let roots = allowed_roots(workspace.path(), Some(outputs.path().to_path_buf()));

    assert!(roots.contains(&outputs.path().canonicalize().unwrap()));
}

#[cfg(unix)]
#[test]
fn configured_outputs_reached_through_a_symlink_do_not_add_a_notice() {
    use std::os::unix::fs::symlink;

    let workspace = tempfile::tempdir().expect("workspace");
    let parent = tempfile::tempdir().expect("outputs parent");
    let outputs = parent.path().join("real-outputs");
    let shortcut = parent.path().join("outputs-shortcut");
    std::fs::create_dir(&outputs).expect("create outputs");
    symlink(&outputs, &shortcut).expect("create outputs shortcut");
    let changed = outputs.join("report.md");
    std::fs::write(&changed, "ok").expect("write report");
    let result =
        ToolResult::ok("created").with_affected_paths(vec![changed.to_string_lossy().to_string()]);

    let result = append_with_outputs(result, workspace.path(), Some(shortcut));

    assert_eq!(result.content, "created");
}
