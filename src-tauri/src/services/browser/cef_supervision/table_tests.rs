use super::{
    CefAuthorityTable, CefLaunchGate, CefLaunchMarker, CefProcessRole, CefPublication,
    CefTableError, CEF_SLOT_CAPACITY,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn the_fixed_table_refuses_the_sixty_fifth_reservation() {
    let table = CefAuthorityTable::new();
    let mut reservations = Vec::with_capacity(CEF_SLOT_CAPACITY);
    for _ in 0..CEF_SLOT_CAPACITY {
        reservations.push(table.try_reserve(CefProcessRole::Helper).expect("slot"));
    }

    assert_eq!(
        table.try_reserve(CefProcessRole::Helper).unwrap_err(),
        CefTableError::Capacity
    );

    let old_marker = reservations.remove(0).encode_marker();
    let released = reservations;
    drop(released);
    let replacement = table
        .try_reserve(CefProcessRole::Helper)
        .expect("replacement");
    assert_ne!(replacement.encode_marker().as_str(), old_marker.as_str());
}

#[test]
fn closing_invalidates_every_unadmitted_reservation() {
    let table = CefAuthorityTable::new();
    let reservation = table
        .try_reserve(CefProcessRole::Helper)
        .expect("reservation");
    let publication = CefPublication::from_marker(reservation.marker(), 41).expect("publication");

    assert!(table.close_and_invalidate(Instant::now() + Duration::from_millis(20)));
    assert_eq!(
        table.try_reserve(CefProcessRole::Helper).unwrap_err(),
        CefTableError::Closed
    );
    assert_eq!(
        table.claim(&publication).unwrap_err(),
        CefTableError::Closed
    );
}

#[test]
fn publication_is_claimed_once_and_admission_releases_the_exact_slot() {
    let table = CefAuthorityTable::new();
    let reservation = table
        .try_reserve(CefProcessRole::Helper)
        .expect("reservation");
    let publication = CefPublication::from_marker(reservation.marker(), 42).expect("publication");
    let replay = CefPublication::from_marker(reservation.marker(), 42).expect("replay");

    let claim = table.claim(&publication).expect("claim");
    assert_eq!(table.claim(&replay).unwrap_err(), CefTableError::Stale);
    let admission = claim.admit().expect("admission");
    drop(admission);
    drop(reservation);

    assert!(table.try_reserve(CefProcessRole::Helper).is_ok());
}

#[test]
fn a_marker_from_one_slot_cannot_publish_into_another_slot() {
    let table = CefAuthorityTable::new();
    let first = table.try_reserve(CefProcessRole::Helper).expect("first");
    let second = table.try_reserve(CefProcessRole::Helper).expect("second");
    let first_value = first.encode_marker();
    let forged_value = first_value.replacen("v1:0:", "v1:1:", 1);
    let forged_marker = CefLaunchMarker::decode_unique(&[&forged_value]).expect("syntactic marker");
    let forged_publication = CefPublication::from_marker(&forged_marker, 43).expect("publication");

    assert_eq!(
        table.claim(&forged_publication).unwrap_err(),
        CefTableError::Stale
    );

    let valid = CefPublication::from_marker(second.marker(), 44).expect("valid publication");
    assert!(table.claim(&valid).is_ok());
}

#[test]
fn a_stale_generation_cannot_claim_a_reused_slot() {
    let table = CefAuthorityTable::new();
    let first = table.try_reserve(CefProcessRole::Helper).expect("first");
    let old_value = first.encode_marker();
    drop(first);
    let replacement = table
        .try_reserve(CefProcessRole::Helper)
        .expect("replacement");
    let old_marker = CefLaunchMarker::decode_unique(&[old_value.as_str()]).expect("old marker");
    let stale = CefPublication::from_marker(&old_marker, 45).expect("stale publication");

    assert_eq!(table.claim(&stale).unwrap_err(), CefTableError::Stale);

    let current = CefPublication::from_marker(replacement.marker(), 46).expect("current");
    assert!(table.claim(&current).is_ok());
}

#[test]
fn the_gate_never_waits_past_its_absolute_deadline() {
    let gate = Arc::new(CefLaunchGate::new());
    let permit = gate.try_enter().expect("permit");
    let closer = Arc::clone(&gate);
    let started = Instant::now();
    let join = std::thread::spawn(move || {
        closer.close_and_wait(Instant::now() + Duration::from_millis(5))
    });

    assert!(!join.join().expect("closer"));
    assert!(started.elapsed() < Duration::from_millis(100));
    drop(permit);
    assert!(gate.close_and_wait(Instant::now() + Duration::from_millis(20)));
    assert!(gate.try_enter().is_err());
}

#[test]
fn a_zero_pid_is_never_publishable() {
    let table = CefAuthorityTable::new();
    let reservation = table
        .try_reserve(CefProcessRole::Helper)
        .expect("reservation");

    assert_eq!(
        CefPublication::from_marker(reservation.marker(), 0).unwrap_err(),
        CefTableError::Invalid
    );
}
