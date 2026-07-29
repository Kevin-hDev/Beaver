use super::*;

#[test]
fn test_prepare_command_preserves_unix_command() {
    let prepared = prepare_command("ls");
    if !cfg!(target_os = "windows") {
        assert_eq!(prepared, "ls");
    }
}

#[tokio::test]
async fn cd_does_not_change_the_next_bash_working_directory() {
    let project = tempfile::tempdir().expect("project");
    let nested = project.path().join("nested");
    std::fs::create_dir(&nested).expect("nested");

    execute_shell("cd nested", project.path(), None)
        .await
        .expect("first bash");
    let next = execute_shell("pwd -P", project.path(), None)
        .await
        .expect("second bash");

    assert_eq!(
        next.stdout,
        project.path().canonicalize().expect("canonical").display().to_string()
    );
}

#[tokio::test]
async fn explicit_external_workdir_is_allowed_for_one_call() {
    let project = tempfile::tempdir().expect("project");
    let external = tempfile::tempdir().expect("external");

    let resolved = resolve_workdir(external.path().to_str(), project.path())
        .expect("external workdir");
    let output = execute_shell("pwd -P", &resolved, None)
        .await
        .expect("external bash");

    assert_eq!(
        resolved,
        external.path().canonicalize().expect("canonical external")
    );
    assert_eq!(output.stdout, resolved.display().to_string());
}

#[test]
fn relative_or_missing_workdir_is_rejected() {
    let project = tempfile::tempdir().expect("project");

    assert!(resolve_workdir(Some("../outside"), project.path()).is_err());
    assert!(resolve_workdir(Some("/definitely/missing/beaver-workdir"), project.path()).is_err());
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn test_execute_shell_reports_affected_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = execute_shell(
        "printf 'hello\\n' > created.md && printf 'tsx\\n' > component.tsx",
        dir.path(),
        None,
    )
    .await
    .expect("execute shell");

    let mut paths = out.affected_paths;
    paths.sort();

    let expected = vec![
        dir.path()
            .join("component.tsx")
            .canonicalize()
            .expect("component"),
        dir.path()
            .join("created.md")
            .canonicalize()
            .expect("created"),
    ]
    .into_iter()
    .map(|path| path.to_string_lossy().to_string())
    .collect::<Vec<_>>();

    assert_eq!(out.exit_code, 0);
    assert_eq!(paths, expected);
    assert_eq!(out.file_changes.len(), 2);
    assert!(out.file_changes.iter().all(|change| change.diff.is_some()));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn test_execute_shell_reports_delete_before_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let deleted = dir.path().join("deleted.md");
    std::fs::write(&deleted, "one\ntwo\n").expect("initial write");

    let out = execute_shell("rm deleted.md && false", dir.path(), None)
        .await
        .expect("execute shell");

    assert_ne!(out.exit_code, 0);
    assert_eq!(out.file_changes.len(), 1);
    let change = &out.file_changes[0];
    assert!(matches!(
        change.status,
        super::super::types_tools::ToolFileChangeStatus::Deleted
    ));
    assert_eq!((change.additions, change.deletions), (0, 2));
    assert!(change.diff.is_some());
}

#[test]
fn test_dev_server_command_detected_as_background() {
    assert!(super::super::tool_bash_long::should_run_in_background(
        "npm run dev -- --host 127.0.0.1"
    ));
    assert!(super::super::tool_bash_long::should_run_in_background(
        "cargo watch -x check"
    ));
    assert!(!super::super::tool_bash_long::should_run_in_background(
        "cargo check"
    ));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn test_background_command_returns_when_ready() {
    let command = "printf 'Local: http://127.0.0.1:5173\\n'; while true; do sleep 1; done";
    let started = std::time::Instant::now();
    let out = execute_shell(command, std::path::Path::new("/tmp"), Some(10)).await;
    super::super::tool_bash_background::abort_all_for_test();

    let shell_out = out.expect("commande longue devrait réussir");
    assert_eq!(shell_out.exit_code, 0);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(3),
        "la commande ne doit pas attendre le timeout complet"
    );
    assert!(shell_out.stdout.contains("Commande longue active"));
}
