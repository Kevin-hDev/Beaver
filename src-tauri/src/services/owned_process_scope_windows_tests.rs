use super::*;
use std::process::Stdio;
use std::time::Duration;

fn sleeping_command() -> tokio::process::Command {
    let mut command = tokio::process::Command::new("cmd.exe");
    command
        .args(["/C", "ping -n 30 127.0.0.1 >NUL"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    command
}

#[tokio::test]
async fn dedicated_jobs_are_distinct_and_targeted() {
    let (mut first, first_scope) =
        OwnedProcessScope::spawn_tokio(&mut sleeping_command(), ProcessKind::ExtensionHost)
            .await
            .unwrap();
    let (mut second, second_scope) =
        OwnedProcessScope::spawn_tokio(&mut sleeping_command(), ProcessKind::ExtensionHost)
            .await
            .unwrap();
    let first_pid = first.id().unwrap();
    let second_pid = second.id().unwrap();
    assert!(first_scope.contains(first_pid));
    assert!(!first_scope.contains(second_pid));
    assert!(second_scope.contains(second_pid));
    assert_eq!(first_scope.identity(first_pid).unwrap().pid, first_pid);
    assert!(first_scope.identity(second_pid).is_err());
    assert!(second_scope.identity(first_pid).is_err());
    assert_eq!(second_scope.identity(second_pid).unwrap().pid, second_pid);
    assert!(super::super::OwnedProcess::identity(first_pid).is_err());

    assert!(first_scope.terminate());
    tokio::time::timeout(Duration::from_secs(3), first.wait())
        .await
        .expect("first dedicated job must stop")
        .unwrap();
    assert!(second.try_wait().unwrap().is_none());

    assert!(second_scope.terminate());
    let _ = second.wait().await;
}
