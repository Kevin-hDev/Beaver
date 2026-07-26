use super::*;

#[test]
fn highest_priority_session_wins_then_previous_work_resumes() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.update("work", MascotAnimation::WorkLaptop, None, false, now);
    let failed = arbiter.update(
        "failed",
        MascotAnimation::Failed,
        Some(Duration::from_secs(2)),
        false,
        now,
    );

    assert_eq!(failed.expect("state").animation, MascotAnimation::Failed);
    let resumed = arbiter.refresh(now + Duration::from_secs(3));
    assert_eq!(
        resumed.expect("state").animation,
        MascotAnimation::WorkLaptop
    );
}

#[test]
fn waiting_beats_success_from_another_session() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.update("waiting", MascotAnimation::Waiting, None, false, now);
    arbiter.update(
        "done",
        MascotAnimation::Success,
        Some(Duration::from_secs(2)),
        false,
        now + Duration::from_millis(1),
    );

    assert_eq!(arbiter.state().animation, MascotAnimation::Waiting);
}

#[test]
fn externally_fed_session_collection_stays_bounded() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    for index in 0..80 {
        arbiter.update(
            &format!("session-{index}"),
            MascotAnimation::Thinking,
            None,
            false,
            now + Duration::from_millis(index),
        );
    }

    assert_eq!(arbiter.session_count(), MAX_ACTIVE_SESSIONS);
}

#[test]
fn invalid_session_identifiers_are_ignored() {
    let mut arbiter = ActivityArbiter::default();
    arbiter.update("", MascotAnimation::Failed, None, false, Instant::now());

    assert_eq!(arbiter.session_count(), 0);
    assert_eq!(arbiter.state().animation, MascotAnimation::Idle);
}

#[test]
fn transient_alert_resumes_the_same_session_activity() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.update("work", MascotAnimation::WorkLaptop, None, false, now);
    arbiter.update(
        "work",
        MascotAnimation::Alert,
        Some(Duration::from_secs(1)),
        true,
        now,
    );

    arbiter.refresh(now + Duration::from_secs(2));
    assert_eq!(arbiter.state().animation, MascotAnimation::WorkLaptop);
}
