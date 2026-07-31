use super::*;
use std::time::{Duration, Instant};

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
    warm_file_watcher(dir.path()).await;
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
    assert!(output
        .affected_paths
        .iter()
        .any(|path| path == &created));
    assert!(output.file_changes.iter().all(|change| change.diff.is_none()));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn long_process_yields_then_can_be_stopped_with_its_children() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();
    let output = managed(
        "sleep 30 & child=$!; printf '%s\\n' \"$child\"; wait",
        dir.path(),
        &owner,
        None,
        Some(250),
        CancellationToken::new(),
    )
        .await
        .expect("start long command");
    let process_id = process_id(&output.stdout);
    let child_pid = output
        .stdout
        .lines()
        .next()
        .and_then(|line| line.parse::<i32>().ok())
        .expect("child pid");

    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(output.stdout.contains("Processus actif"));

    let stopped = control_shell_session(
        process_id,
        None,
        true,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("stop process");

    assert_ne!(stopped.exit_code, 0);
    assert!(stopped.stderr.contains("annulee"));
    let deadline = Instant::now() + Duration::from_secs(1);
    while process_exists(child_pid) && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(!process_exists(child_pid));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn inherited_background_pipe_does_not_delay_shell_completion() {
    let dir = tempfile::tempdir().expect("tempdir");
    let started = Instant::now();

    let output = execute_shell("sleep 30 &", dir.path(), None)
        .await
        .expect("background shell");

    assert_eq!(output.exit_code, 0);
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn running_process_accepts_input_through_bash_write() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let started = managed(
        "read value; printf 'received:%s' \"$value\"",
        dir.path(),
        &owner,
        None,
        Some(250),
        CancellationToken::new(),
    )
    .await
    .expect("start process");
    let process_id = process_id(&started.stdout);

    let completed = control_shell_session(
        process_id,
        Some("hello\n"),
        false,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("write input");

    assert_eq!(completed.exit_code, 0);
    assert!(completed.stdout.contains("received:hello"));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn running_process_rejects_other_sessions_and_invalid_input() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let other_owner = uuid::Uuid::new_v4().to_string();
    let started = managed(
        "read value; printf 'received:%s' \"$value\"",
        dir.path(),
        &owner,
        None,
        Some(250),
        CancellationToken::new(),
    )
    .await
    .expect("start process");
    let process_id = process_id(&started.stdout);

    assert!(control_shell_session(
        process_id,
        Some("intrusion\n"),
        false,
        &other_owner,
        Some(250),
        CancellationToken::new(),
        None,
    )
    .await
    .is_err());
    assert!(control_shell_session(
        process_id,
        Some("invalid\0input"),
        false,
        &owner,
        Some(250),
        CancellationToken::new(),
        None,
    )
    .await
    .is_err());

    let completed = control_shell_session(
        process_id,
        Some("valid\n"),
        false,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("finish process");
    assert!(completed.stdout.contains("received:valid"));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn hard_timeout_terminates_the_process_tree() {
    let dir = tempfile::tempdir().expect("tempdir");
    let started = Instant::now();

    let output = execute_shell("sleep 30", dir.path(), Some(1))
        .await
        .expect("timed shell");

    assert!(output.timed_out);
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn caller_cancellation_stops_the_running_command() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let cancel = CancellationToken::new();
    let command_cancel = cancel.clone();
    let path = dir.path().to_path_buf();
    let task = tokio::spawn(async move {
        managed("sleep 30", &path, &owner, None, Some(30_000), command_cancel).await
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    let started = Instant::now();
    cancel.cancel();
    let result = task.await.expect("task");

    match result {
        Ok(output) => assert_ne!(output.exit_code, 0),
        Err(error) => assert!(error.contains("annulee")),
    }
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn large_output_keeps_a_bounded_preview_and_a_complete_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = execute_shell("yes x | head -c 2000000", dir.path(), None)
        .await
        .expect("large output");
    let relative = output
        .stdout
        .split("[Résultat complet disponible : ")
        .nth(1)
        .and_then(|tail| tail.split(']').next())
        .expect("output path");
    let path = crate::services::paths::data_dir().join(relative);
    let full = tokio::fs::read(&path).await.expect("full output");

    assert_eq!(output.exit_code, 0);
    assert!(output.stdout.len() < 30_000);
    assert_eq!(full.len(), 2_000_000);
    tokio::fs::remove_file(path).await.expect("cleanup output");
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn short_output_does_not_create_a_result_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let output = managed(
        "printf small",
        dir.path(),
        &owner,
        None,
        None,
        CancellationToken::new(),
    )
    .await
    .expect("short output");
    let output_dir = crate::services::paths::data_dir()
        .join("tool-results")
        .join(owner);

    assert_eq!(output.stdout, "small");
    assert!(!output_dir.exists());
}

async fn managed(
    command: &str,
    working_dir: &Path,
    owner: &str,
    timeout: Option<u64>,
    yield_time_ms: Option<u64>,
    cancel: CancellationToken,
) -> Result<super::super::types_tools::ShellOutput, String> {
    execute_shell_managed(
        command,
        working_dir,
        ShellExecutionContext {
            owner_session_id: owner,
            hard_timeout_secs: timeout,
            yield_time_ms,
            cancel,
            progress: None,
        },
    )
    .await
}

fn process_id(output: &str) -> &str {
    output
        .split("session_id=")
        .nth(1)
        .and_then(|tail| tail.split(',').next())
        .expect("process id")
}

fn canonical(path: &Path) -> String {
    path.canonicalize()
        .expect("canonical path")
        .to_string_lossy()
        .to_string()
}

async fn warm_file_watcher(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        if super::super::tool_bash_changes::ChangeTracker::start(path)
            .await
            .is_ok()
        {
            return;
        }
        assert!(Instant::now() < deadline, "watcher did not initialize");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[cfg(unix)]
fn process_exists(pid: i32) -> bool {
    // SAFETY: signal 0 performs a read-only existence check for the captured child PID.
    let result = unsafe { libc::kill(pid, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}
