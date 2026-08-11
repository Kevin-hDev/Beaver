use super::windows::{
    classify_termination, WindowsCefTracker, WindowsConfinement, WindowsHelperObjects,
    WindowsNativeAuthority, WindowsProcessIdentity, WindowsProcessProbe, WindowsTerminationState,
};
use super::windows_identity_tests::ChildGuard;
use super::{CefAuthorityTable, CefProcessRole, CefPublication, CEF_SLOT_CAPACITY};
use std::time::{Duration, Instant};

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

#[test]
fn emergency_close_rejects_new_helpers_and_force_stops_an_admitted_helper() {
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
    let pending_ticket = tracker.reserve().expect("pending launch ticket");
    let pending_marker = pending_ticket.decode_marker().expect("pending marker");
    let pending_names = super::CefIpcNames::from_marker(&pending_marker).expect("pending names");
    let pending_helper = WindowsHelperObjects::open(&pending_names).expect("pending objects");

    assert!(tracker.close_gate_for_test());
    assert!(tracker.reserve().is_err());
    assert!(WindowsProcessProbe::read(child.id()).is_ok());
    assert!(helper.wait_for_closing(100).expect("admitted closing"));
    assert!(pending_helper
        .wait_for_closing(100)
        .expect("pending closing"));
    let admitted_control = helper.control_snapshot().expect("admitted control");
    let pending_control = pending_helper.control_snapshot().expect("pending control");
    assert!(admitted_control.closing);
    assert_eq!(
        pending_control.deadline_ticks,
        admitted_control.deadline_ticks
    );
    assert_ne!(admitted_control.deadline_ticks, 0);

    tracker.force_for_test();
    assert!(wait_until_native_process_disappears(child.id()));
}

#[test]
fn unpublished_reservations_expire_without_failing_the_tracker() {
    let child = ChildGuard::spawn();
    let probe = WindowsProcessProbe::read(child.id()).expect("probe");
    let tracker = WindowsCefTracker::start(probe.executable()).expect("tracker");
    let expires_at = Instant::now() + Duration::from_secs(2);
    for _ in 0..CEF_SLOT_CAPACITY {
        tracker
            .handle()
            .reserve_until_for_test(expires_at)
            .expect("expiring reservation");
    }
    assert!(tracker.reserve().is_err());

    let deadline = expires_at + Duration::from_secs(3);
    let mut replacements = Vec::with_capacity(CEF_SLOT_CAPACITY);
    while replacements.len() < CEF_SLOT_CAPACITY && Instant::now() < deadline {
        match tracker.reserve() {
            Ok(ticket) => replacements.push(ticket),
            Err(_) => std::thread::sleep(Duration::from_millis(10)),
        }
    }

    assert_eq!(replacements.len(), CEF_SLOT_CAPACITY);
    assert!(tracker.reserve().is_err());
    assert_eq!(tracker.failure(), None);
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
