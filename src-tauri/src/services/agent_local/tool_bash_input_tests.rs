use super::test_support::{managed, process_id};
use super::*;
use tokio_util::sync::CancellationToken;

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
async fn bash_write_blocks_destructive_input_before_it_reaches_the_process() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let started = managed(
        "cat",
        dir.path(),
        &owner,
        None,
        Some(250),
        CancellationToken::new(),
    )
    .await
    .expect("start process");
    let process_id = process_id(&started.stdout);

    let blocked = control_shell_session(
        process_id,
        Some("sudo rm harmless\n"),
        false,
        false,
        &owner,
        Some(250),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("blocked input");

    assert_eq!(blocked.exit_code, -1);
    assert!(blocked.stderr.contains("bloquée"));
    let _ = control_shell_session(
        process_id,
        None,
        false,
        true,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await;
}

#[cfg(not(target_os = "windows"))]
#[tokio::test]
async fn bash_write_can_send_input_then_close_stdin() {
    let dir = tempfile::tempdir().expect("tempdir");
    let owner = uuid::Uuid::new_v4().to_string();
    let started = managed(
        "cat",
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
        Some("payload"),
        true,
        false,
        &owner,
        Some(1_000),
        CancellationToken::new(),
        None,
    )
    .await
    .expect("send eof");

    assert!(!completed.running);
    assert_eq!(completed.exit_code, 0);
    assert_eq!(completed.stdout, "payload");
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
