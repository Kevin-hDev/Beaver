use super::*;

#[test]
fn highest_priority_session_wins_then_previous_work_resumes() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.start("work", 1, now);
    arbiter.update(
        "work",
        Some(1),
        MascotAnimation::WorkLaptop,
        None,
        false,
        now,
    );
    arbiter.start("failed", 2, now);
    let failed = arbiter.update(
        "failed",
        Some(2),
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
    arbiter.start("waiting", 1, now);
    arbiter.update(
        "waiting",
        Some(1),
        MascotAnimation::Waiting,
        None,
        false,
        now,
    );
    arbiter.start("done", 2, now);
    arbiter.update(
        "done",
        Some(2),
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
        arbiter.start(
            &format!("session-{index}"),
            index,
            now + Duration::from_millis(index),
        );
    }

    assert_eq!(arbiter.session_count(), MAX_ACTIVE_SESSIONS);
}

#[test]
fn invalid_session_identifiers_are_ignored() {
    let mut arbiter = ActivityArbiter::default();
    arbiter.start("", 1, Instant::now());

    assert_eq!(arbiter.session_count(), 0);
    assert_eq!(arbiter.state().animation, MascotAnimation::Idle);
}

#[test]
fn transient_alert_resumes_the_same_session_activity() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.start("work", 1, now);
    arbiter.update(
        "work",
        Some(1),
        MascotAnimation::WorkLaptop,
        None,
        false,
        now,
    );
    arbiter.update(
        "work",
        Some(1),
        MascotAnimation::Alert,
        Some(Duration::from_secs(1)),
        true,
        now,
    );

    arbiter.refresh(now + Duration::from_secs(2));
    assert_eq!(arbiter.state().animation, MascotAnimation::WorkLaptop);
}

#[test]
fn stale_generation_cannot_override_or_remove_the_current_run() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.start("session", 1, now);
    arbiter.update(
        "session",
        Some(1),
        MascotAnimation::WorkLaptop,
        None,
        false,
        now,
    );
    arbiter.start("session", 2, now + Duration::from_millis(1));

    assert!(arbiter
        .update(
            "session",
            Some(1),
            MascotAnimation::WorkLaptop,
            None,
            false,
            now + Duration::from_millis(2),
        )
        .is_none());
    assert!(arbiter
        .remove("session", 1, now + Duration::from_millis(3))
        .is_none());
    assert_eq!(arbiter.state().animation, MascotAnimation::Thinking);
}

#[test]
fn events_without_an_owned_session_cannot_create_an_orphan() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();

    assert!(arbiter
        .update(
            "missing",
            None,
            MascotAnimation::WorkLaptop,
            None,
            false,
            now,
        )
        .is_none());
    assert_eq!(arbiter.session_count(), 0);
    assert_eq!(arbiter.state().animation, MascotAnimation::Idle);
}

#[test]
fn terminal_state_expires_to_idle() {
    let now = Instant::now();
    let mut arbiter = ActivityArbiter::default();
    arbiter.start("session", 1, now);
    arbiter.update(
        "session",
        Some(1),
        MascotAnimation::Success,
        Some(Duration::from_secs(2)),
        false,
        now,
    );

    let idle = arbiter.refresh(now + Duration::from_secs(3));
    assert_eq!(idle.expect("idle state").animation, MascotAnimation::Idle);
    assert_eq!(arbiter.session_count(), 0);
}
