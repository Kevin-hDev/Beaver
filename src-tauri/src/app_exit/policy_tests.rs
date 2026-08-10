use super::policy::{post_loop_sweep_timeout, ShutdownPolicy, ShutdownTimeline};
use std::time::{Duration, Instant};

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
