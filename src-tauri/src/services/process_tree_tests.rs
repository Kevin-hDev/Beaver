#[cfg(unix)]
use super::unix::UnixProcessIdentity;
#[cfg(unix)]
use sysinfo::Pid;

#[cfg(unix)]
#[test]
fn stale_descendant_identity_is_rejected_before_signal() {
    let identity = UnixProcessIdentity::new(Pid::from_u32(42), 100);

    assert!(!identity.matches(Pid::from_u32(42), 101));
    assert!(identity.matches(Pid::from_u32(42), 100));
}

#[test]
fn after_parent_cleanup_refuses_a_reused_root_pid() {
    let mut termination_attempted = false;
    let result = super::after_parent::kill_pipe_holders_with(
        42,
        super::ProcessKind::Searxng,
        |_| true,
        |_, _| {
            termination_attempted = true;
            true
        },
        |_| {},
    );

    assert!(!result);
    assert!(!termination_attempted);
}

#[cfg(unix)]
#[test]
fn stale_group_member_is_rechecked_before_any_signal() {
    let member = UnixProcessIdentity::new(Pid::from_u32(42), 100);
    let mut signalled = Vec::new();

    super::after_parent::signal_members_with(
        &[member],
        7,
        libc::SIGTERM,
        |_, _| false,
        |pid, signal| signalled.push((pid, signal)),
    );

    assert!(signalled.is_empty());
}

#[cfg(unix)]
#[test]
fn terminate_reaps_child_without_three_second_delay() {
    let mut command = std::process::Command::new("/bin/sleep");
    command.arg("30");
    super::configure(&mut command);
    let mut child = command.spawn().unwrap();
    let started = std::time::Instant::now();

    super::terminate(&mut child, super::ProcessKind::ForecastRuntime);

    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    assert!(child.try_wait().unwrap().is_some());
}

#[tokio::test]
async fn scoped_termination_confirms_a_descendant_holding_stdout() {
    let node = which::which("node").expect("Node fixture runtime");
    let mut command = tokio::process::Command::new(node);
    command
        .args([
            "-e",
            "const {spawn}=require('node:child_process'); spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:['ignore',process.stdout,'ignore']}); setInterval(()=>{},1000)",
        ])
        .stdout(std::process::Stdio::piped());
    let (mut child, scope) = crate::services::owned_process::OwnedProcess::spawn_tokio_scoped(
        &mut command,
        super::ProcessKind::ExtensionHost,
    )
    .await
    .unwrap();
    let root_pid = child.id().unwrap();

    assert!(
        super::terminate_tokio_scoped(
            &mut child,
            super::ProcessKind::ExtensionHost,
            &scope,
            root_pid,
            std::time::Instant::now() + std::time::Duration::from_secs(5),
        )
        .await
    );
}

#[tokio::test]
async fn a_live_scoped_descendant_prevents_confirmation() {
    let mut command = tokio::process::Command::new(which::which("node").unwrap());
    command.args([
        "-e",
        "require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)']); setInterval(()=>{},1000)",
    ]);
    let (mut child, scope) = crate::services::owned_process::OwnedProcess::spawn_tokio_scoped(
        &mut command,
        super::ProcessKind::ExtensionHost,
    )
    .await
    .unwrap();
    let root_pid = child.id().unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(!super::scope::confirm_scope_absent(&scope, root_pid, std::time::Instant::now()).await);
    assert!(
        super::terminate_tokio_scoped(
            &mut child,
            super::ProcessKind::ExtensionHost,
            &scope,
            root_pid,
            std::time::Instant::now() + std::time::Duration::from_secs(5),
        )
        .await
    );
}

#[cfg(unix)]
#[tokio::test]
async fn scoped_cleanup_reaps_pipe_holder_after_parent_exit() {
    let mut command = tokio::process::Command::new(which::which("node").unwrap());
    command
        .args([
            "-e",
            "require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:['ignore',process.stdout,'ignore']}); setTimeout(()=>process.exit(0),50)",
        ])
        .stdout(std::process::Stdio::piped());
    let (mut child, scope) = crate::services::owned_process::OwnedProcess::spawn_tokio_scoped(
        &mut command,
        super::ProcessKind::ExtensionHost,
    )
    .await
    .unwrap();
    let root_pid = child.id().unwrap();
    child.wait().await.unwrap();

    assert!(
        super::terminate_tokio_scoped(
            &mut child,
            super::ProcessKind::ExtensionHost,
            &scope,
            root_pid,
            std::time::Instant::now() + std::time::Duration::from_millis(500),
        )
        .await
    );
}

#[cfg(unix)]
#[tokio::test]
async fn reused_root_pid_blocks_old_group_cleanup() {
    let mut command = tokio::process::Command::new(which::which("node").unwrap());
    command.args(["-e", "setTimeout(() => process.exit(0), 10)"]);
    let (mut child, scope) = crate::services::owned_process::OwnedProcess::spawn_tokio_scoped(
        &mut command,
        super::ProcessKind::ExtensionHost,
    )
    .await
    .unwrap();
    let root_pid = child.id().unwrap();
    child.wait().await.unwrap();

    assert!(
        !super::scope::confirm_scope_absent_with_root_probe(
            &scope,
            root_pid,
            std::time::Instant::now() + std::time::Duration::from_millis(50),
            |_| true,
        )
        .await
    );

    crate::services::owned_process::release(root_pid);
}
