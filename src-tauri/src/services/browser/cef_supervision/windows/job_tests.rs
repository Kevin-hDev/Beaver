use super::windows::{WindowsJobGuard, WindowsProcessIdentity, WindowsProcessProbe};
use super::windows_identity_tests::ChildGuard;

#[test]
fn an_empty_kill_on_close_job_confines_and_stops_the_identified_child() {
    let child = ChildGuard::spawn();
    let probe = WindowsProcessProbe::read(child.id()).expect("probe");
    let identity = WindowsProcessIdentity::acquire(
        child.id(),
        std::process::id(),
        probe.started_at(),
        probe.executable(),
    )
    .expect("identity");
    let job = WindowsJobGuard::new().expect("job");

    assert!(job.is_empty().expect("empty job"));
    assert!(job.has_only_kill_on_close().expect("job limits"));
    job.assign(&identity).expect("assignment");
    assert!(job.contains(&identity).expect("membership"));
    drop(job);

    assert!(identity.wait_for_exit(5_000).expect("job termination"));
}
