use super::windows::{
    classify_termination, WindowsCefTracker, WindowsConfinement, WindowsHelperObjects,
    WindowsNativeAuthority, WindowsProcessIdentity, WindowsProcessProbe, WindowsTerminationState,
};
use super::windows_identity_tests::ChildGuard;
use super::{CefAuthorityTable, CefProcessRole, CefPublication};

#[test]
fn an_accepted_termination_is_not_released_before_the_handle_is_signaled() {
    assert_eq!(
        classify_termination(true, false),
        WindowsTerminationState::Terminating
    );
    assert_eq!(
        classify_termination(true, true),
        WindowsTerminationState::Exited
    );
}

#[test]
fn the_precreated_tracker_claims_publication_and_confines_before_admission() {
    let child = ChildGuard::spawn();
    let probe = WindowsProcessProbe::read(child.id()).expect("probe");
    let tracker = WindowsCefTracker::start(probe.executable()).expect("tracker");
    let ticket = tracker.reserve().expect("launch ticket");
    let marker = ticket.decode_marker().expect("ticket marker");
    let names = super::CefIpcNames::from_marker(&marker).expect("names");
    let helper = WindowsHelperObjects::open(&names).expect("helper objects");
    helper
        .publish(marker.generation(), child.id(), probe.started_at(), 0)
        .expect("publication");

    assert!(helper.wait_for_admission(2_000).expect("admission wait"));
    assert_eq!(tracker.failure(), None);
    drop(tracker);
    assert!(wait_until_native_process_disappears(child.id()));
}

#[test]
fn admission_happens_only_after_the_job_proof_and_termination_keeps_the_proof() {
    let child = ChildGuard::spawn();
    let probe = WindowsProcessProbe::read(child.id()).expect("probe");
    let identity = WindowsProcessIdentity::acquire(
        child.id(),
        std::process::id(),
        probe.started_at(),
        probe.executable(),
    )
    .expect("identity");
    let confinement = WindowsConfinement::establish(identity).expect("confinement");
    let table = CefAuthorityTable::new();
    let reservation = table
        .try_reserve(CefProcessRole::Helper)
        .expect("reservation");
    let publication =
        CefPublication::from_marker(reservation.marker(), child.id()).expect("publication");
    let claim = table.claim(&publication).expect("claim");
    let authority = WindowsNativeAuthority::new();
    let pending = authority
        .prepare(&claim, confinement)
        .expect("native proof");
    let tracked = pending.admit(claim).expect("admission");

    assert_eq!(
        tracked.observe().expect("observed state"),
        WindowsTerminationState::Admitted
    );
    assert!(matches!(
        tracked.terminate().expect("termination request"),
        WindowsTerminationState::Terminating | WindowsTerminationState::Exited
    ));
    for _ in 0..100 {
        if tracked.observe().expect("termination observation") == WindowsTerminationState::Exited {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(
        tracked.observe().expect("final termination state"),
        WindowsTerminationState::Exited
    );
    drop(tracked);
    assert!(wait_until_process_disappears(child.id(), &authority));
    assert_eq!(authority.occupied_slots(), 0);
}

#[test]
fn abandoning_a_prepared_admission_closes_its_job_without_admitting_cef() {
    let child = ChildGuard::spawn();
    let probe = WindowsProcessProbe::read(child.id()).expect("probe");
    let identity = WindowsProcessIdentity::acquire(
        child.id(),
        std::process::id(),
        probe.started_at(),
        probe.executable(),
    )
    .expect("identity");
    let confinement = WindowsConfinement::establish(identity).expect("confinement");
    let table = CefAuthorityTable::new();
    let reservation = table
        .try_reserve(CefProcessRole::Helper)
        .expect("reservation");
    let publication =
        CefPublication::from_marker(reservation.marker(), child.id()).expect("publication");
    let claim = table.claim(&publication).expect("claim");
    let authority = WindowsNativeAuthority::new();
    let pending = authority
        .prepare(&claim, confinement)
        .expect("native proof");

    drop(pending);
    drop(claim);
    assert!(wait_until_process_disappears(child.id(), &authority));
    assert_eq!(authority.occupied_slots(), 0);
}

fn wait_until_process_disappears(pid: u32, authority: &WindowsNativeAuthority) -> bool {
    for _ in 0..100 {
        authority.refresh_all().expect("bounded tracker refresh");
        if WindowsProcessProbe::read(pid).is_err() {
            authority.refresh_all().expect("final tracker refresh");
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    false
}

fn wait_until_native_process_disappears(pid: u32) -> bool {
    for _ in 0..100 {
        if WindowsProcessProbe::read(pid).is_err() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    false
}
