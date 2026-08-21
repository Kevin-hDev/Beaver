use super::unix::UnixProcessIdentity;
use sysinfo::Pid;

#[test]
fn stale_descendant_identity_is_rejected_before_signal() {
    let identity = UnixProcessIdentity::new(Pid::from_u32(42), 100);

    assert!(!identity.matches(Pid::from_u32(42), 101));
    assert!(identity.matches(Pid::from_u32(42), 100));
}

#[test]
fn after_parent_cleanup_refuses_a_reused_root_pid() {
    assert!(!super::kill_pipe_holders_after_parent_exit(
        std::process::id(),
        super::ProcessKind::Searxng,
    ));
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
