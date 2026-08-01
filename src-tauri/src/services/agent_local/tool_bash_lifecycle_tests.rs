use super::test_support::{managed, process_exists, process_id};
use super::*;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

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
    assert!(output.running);
    assert_eq!(output.exit_code, -1);

    let stopped = control_shell_session(
        process_id,
        None,
        false,
        true,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("stop process");

    assert!(stopped.stopped);
    assert_ne!(stopped.exit_code, 0);
    assert!(stopped.stderr.is_empty());
    assert!(stopped.stdout.contains("Processus arrêté."));
    let deadline = Instant::now() + Duration::from_secs(1);
    while process_exists(child_pid) && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(!process_exists(child_pid));
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn background_jobs_remain_managed_until_stopped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let started = Instant::now();

    let output = managed(
        "sleep 30 &",
        dir.path(),
        &owner,
        None,
        Some(250),
        CancellationToken::new(),
    )
    .await
    .expect("background shell");
    let process_id = process_id(&output.stdout);

    assert!(output.running);
    assert_eq!(output.exit_code, -1);
    assert!(started.elapsed() < Duration::from_secs(2));

    let stopped = control_shell_session(
        process_id,
        None,
        false,
        true,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("stop background shell");
    assert!(!stopped.running);
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn closed_output_pipes_do_not_block_process_completion() {
    let dir = tempfile::tempdir().expect("tempdir");
    let started = Instant::now();

    let output = execute_shell("exec >/dev/null 2>&1; sleep 1", dir.path(), None)
        .await
        .expect("silent process");

    assert_eq!(output.exit_code, 0);
    assert!(started.elapsed() < Duration::from_secs(3));
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
