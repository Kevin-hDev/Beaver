#[cfg(unix)]
use super::test_support::canonical;
use super::*;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
#[tokio::test]
async fn short_commands_complete_without_changing_the_next_workdir() {
    let project = tempfile::tempdir().expect("project");
    let nested = project.path().join("nested");
    std::fs::create_dir(&nested).expect("nested");

    execute_shell("cd nested", project.path(), None)
        .await
        .expect("first bash");
    let next = execute_shell("pwd -P", project.path(), None)
        .await
        .expect("second bash");

    assert_eq!(next.exit_code, 0);
    assert_eq!(next.stdout.trim(), canonical(project.path()));
}

#[cfg(unix)]
#[tokio::test]
async fn explicit_external_workdir_is_allowed_for_one_call() {
    let project = tempfile::tempdir().expect("project");
    let external = tempfile::tempdir().expect("external");
    let resolved = resolve_workdir(external.path().to_str(), project.path()).expect("workdir");

    let output = execute_shell("pwd -P", &resolved, None)
        .await
        .expect("external bash");

    assert_eq!(resolved, external.path().canonicalize().expect("canonical"));
    assert_eq!(output.stdout.trim(), canonical(&resolved));
}

#[test]
fn invalid_workdirs_are_rejected() {
    let project = tempfile::tempdir().expect("project");

    assert!(resolve_workdir(Some("../outside"), project.path()).is_err());
    assert!(resolve_workdir(Some("/definitely/missing/beaver-workdir"), project.path()).is_err());
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn mkdir_is_fast_and_reports_the_changed_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let started = Instant::now();

    let output = execute_shell("mkdir created", dir.path(), None)
        .await
        .expect("mkdir");

    assert_eq!(output.exit_code, 0);
    assert!(started.elapsed() < Duration::from_secs(3));
    let created = dir
        .path()
        .canonicalize()
        .expect("canonical tempdir")
        .join("created")
        .to_string_lossy()
        .to_string();
    assert!(output.affected_paths.iter().any(|path| path == &created));
    assert!(output
        .file_changes
        .iter()
        .all(|change| change.diff.is_none()));
}
