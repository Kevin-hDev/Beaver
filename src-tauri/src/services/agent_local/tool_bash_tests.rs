use super::*;

#[test]
fn test_cwd_marker_extraction() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = format!("hello world\n{marker}\n/tmp\n");
    let (output, new_cwd) = extract_cwd(&raw, marker);
    assert_eq!(output, "hello world");
    assert_eq!(new_cwd, Some("/tmp".to_string()));
}

#[test]
fn test_cwd_marker_extraction_no_marker() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = "hello world\n";
    let (output, new_cwd) = extract_cwd(raw, marker);
    assert_eq!(output, "hello world\n");
    assert!(new_cwd.is_none());
}

#[test]
fn test_cwd_marker_spoofing_nonexistent_path() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = format!("output\n{marker}\n/nonexistent/fake/path\n");
    let (output, new_cwd) = extract_cwd(&raw, marker);
    assert_eq!(output, "output");
    assert!(new_cwd.is_none(), "Faux chemin doit être rejeté");
}

#[test]
fn test_cwd_marker_spoofing_relative_path() {
    let marker = "<<CWD_MARKER_abc123def456>>";
    let raw = format!("output\n{marker}\nrelative/path\n");
    let (_output, new_cwd) = extract_cwd(&raw, marker);
    assert!(new_cwd.is_none(), "Chemin relatif doit être rejeté");
}

#[test]
fn test_generate_cwd_marker_unique() {
    let m1 = generate_cwd_marker();
    let m2 = generate_cwd_marker();
    assert_ne!(m1, m2, "Les markers doivent être uniques");
    assert!(m1.starts_with("<<CWD_MARKER_"));
    assert!(m1.ends_with(">>"));
}

#[test]
fn test_wrap_command_unix() {
    let marker = "<<CWD_MARKER_test>>";
    let wrapped = wrap_command_with_cwd("ls", marker);
    if !cfg!(target_os = "windows") {
        assert!(wrapped.contains("ls"));
        assert!(wrapped.contains(marker));
        assert!(wrapped.contains("pwd -P"));
    }
}

#[tokio::test]
async fn test_cwd_update_after_cd() {
    let out = execute_shell("cd /tmp", std::path::Path::new("/"), None).await;
    match out {
        Ok(shell_out) => {
            // Sur macOS, /tmp est un symlink vers /private/tmp (résolu par
            // canonicalize). Sur Linux, /tmp reste /tmp. On accepte les deux
            // pour que le test soit cross-platform.
            let cwd = shell_out.new_cwd.as_deref().unwrap_or("");
            assert!(
                cwd.ends_with("/tmp"),
                "cd /tmp doit détecter le nouveau cwd, obtenu : {cwd}"
            );
        }
        Err(e) => panic!("execute_shell a échoué : {e}"),
    }
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
