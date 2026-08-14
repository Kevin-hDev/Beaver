use super::policy::{
    post_loop_sweep_timeout, ShutdownPolicy, ShutdownTimeline, OLLAMA_SETUP_GRACE_TIMEOUT,
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct TestClock {
    now: Instant,
}

impl TestClock {
    fn new(now: Instant) -> Self {
        Self { now }
    }

    fn advance(&mut self, duration: Duration) {
        self.now += duration;
    }

    fn now(self) -> Instant {
        self.now
    }
}

#[test]
fn all_deadlines_share_one_origin() {
    let origin = Instant::now();
    let policy = ShutdownPolicy::new(
        Duration::from_secs(8),
        Duration::from_secs(10),
        Duration::from_secs(13),
        Duration::from_secs(15),
    )
    .expect("ordered policy");
    let timeline = ShutdownTimeline::from_origin(origin, policy);

    assert_eq!(
        timeline.graceful_deadline(),
        origin + Duration::from_secs(8)
    );
    assert_eq!(
        timeline.tauri_exit_deadline(),
        origin + Duration::from_secs(10)
    );
    assert_eq!(
        timeline.emergency_deadline(),
        origin + Duration::from_secs(13)
    );
    assert_eq!(
        timeline.ultimate_deadline(),
        origin + Duration::from_secs(15)
    );
    assert_eq!(
        timeline.cef_helper_exit_deadline(),
        origin + Duration::from_secs(14)
    );
}

#[test]
fn remaining_budget_never_restarts_after_deadline() {
    let origin = Instant::now();
    let policy = ShutdownPolicy::new(
        Duration::from_millis(8),
        Duration::from_millis(10),
        Duration::from_millis(13),
        Duration::from_millis(15),
    )
    .expect("ordered policy");
    let timeline = ShutdownTimeline::from_origin(origin, policy);

    assert_eq!(
        timeline.remaining_at(
            timeline.graceful_deadline(),
            origin + Duration::from_millis(3)
        ),
        Duration::from_millis(5)
    );
    assert_eq!(
        timeline.remaining_at(
            timeline.graceful_deadline(),
            origin + Duration::from_millis(9)
        ),
        Duration::ZERO
    );
    assert_eq!(
        timeline.cef_helper_exit_deadline(),
        origin + Duration::from_millis(14)
    );
}

#[test]
fn unordered_or_zero_policy_is_rejected() {
    assert!(ShutdownPolicy::new(
        Duration::ZERO,
        Duration::from_secs(10),
        Duration::from_secs(13),
        Duration::from_secs(15),
    )
    .is_none());
    assert!(ShutdownPolicy::new(
        Duration::from_secs(8),
        Duration::from_secs(7),
        Duration::from_secs(13),
        Duration::from_secs(15),
    )
    .is_none());
}

#[test]
fn production_policy_matches_the_contract() {
    let policy = ShutdownPolicy::production();
    assert_eq!(policy.graceful(), Duration::from_secs(8));
    assert_eq!(policy.tauri_exit(), Duration::from_secs(10));
    assert_eq!(post_loop_sweep_timeout(), Duration::from_secs(3));
    assert_eq!(policy.emergency(), Duration::from_secs(13));
    assert_eq!(policy.ultimate(), Duration::from_secs(15));
}

#[test]
fn ollama_setup_deadline_is_derived_from_the_shutdown_origin() {
    let origin = Instant::now();
    let policy = ShutdownPolicy::production();
    let timeline = ShutdownTimeline::from_origin(origin, policy);

    assert_eq!(
        timeline.ollama_setup_deadline(),
        origin + OLLAMA_SETUP_GRACE_TIMEOUT
    );

    let capped_policy = ShutdownPolicy::new(
        Duration::from_secs(2),
        Duration::from_secs(10),
        Duration::from_secs(13),
        Duration::from_secs(15),
    )
    .expect("ordered capped policy");
    let capped_timeline = ShutdownTimeline::from_origin(origin, capped_policy);

    assert_eq!(
        capped_timeline.ollama_setup_deadline(),
        origin + Duration::from_secs(2)
    );
}

#[test]
fn late_ollama_waiter_only_receives_the_remaining_grace() {
    let origin = Instant::now();
    let timeline = ShutdownTimeline::from_origin(origin, ShutdownPolicy::production());
    let mut clock = TestClock::new(origin);
    clock.advance(Duration::from_millis(2_400));
    let deadline = timeline.ollama_setup_deadline();

    assert_eq!(
        timeline.remaining_at(deadline, clock.now()),
        Duration::from_millis(600)
    );
}

#[test]
fn shutdown_deadlines_keep_the_total_order() {
    let origin = Instant::now();
    let timeline = ShutdownTimeline::from_origin(origin, ShutdownPolicy::production());
    let deadlines = [
        timeline.ollama_setup_deadline().duration_since(origin),
        timeline.graceful_deadline().duration_since(origin),
        timeline.tauri_exit_deadline().duration_since(origin),
        timeline.emergency_deadline().duration_since(origin),
        timeline.cef_helper_exit_deadline().duration_since(origin),
        timeline.ultimate_deadline().duration_since(origin),
    ];

    assert_eq!(
        deadlines,
        [
            Duration::from_secs(3),
            Duration::from_secs(8),
            Duration::from_secs(10),
            Duration::from_secs(13),
            Duration::from_secs(14),
            Duration::from_secs(15),
        ]
    );
    assert!(deadlines.windows(2).all(|pair| pair[0] < pair[1]));
}
