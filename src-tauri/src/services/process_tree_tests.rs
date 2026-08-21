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
