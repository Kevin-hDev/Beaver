use super::policy::ShutdownPolicy;
use super::state::ShutdownPhase;
use super::ultimate::{RawExitActions, UltimateExit};
use super::{AppExitCoordinator, BeginResult};
use std::time::{Duration, Instant};

fn coordinator() -> AppExitCoordinator {
    let policy = ShutdownPolicy::new(
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(300),
        Duration::from_secs(1),
    )
    .expect("coordinator policy");
    let ultimate =
        UltimateExit::initialize_for_test(Instant::now(), RawExitActions::testing(|_| {}, |_| {}))
            .expect("ultimate thread");
    AppExitCoordinator::from_parts_for_test(policy, ultimate)
}

#[test]
fn first_close_owns_cleanup_and_arms_the_ultimate_guard() {
    let coordinator = coordinator();
    let admission = coordinator.admit_for_test().expect("admission");
    let cancellation = admission.cancellation_token();

    assert!(matches!(coordinator.begin(4), BeginResult::Started(_)));
    assert!(cancellation.is_cancelled());
    assert!(coordinator.ultimate_is_armed_for_test());
    assert_eq!(coordinator.phase_for_test(), ShutdownPhase::Closing);
    assert_eq!(coordinator.begin(4), BeginResult::Waiting);
    assert!(coordinator.mark_ready());
    assert_eq!(coordinator.begin(4), BeginResult::Ready);
}

#[test]
fn admission_is_permanently_closed_before_closing_is_visible() {
    let coordinator = coordinator();
    assert!(matches!(coordinator.begin(0), BeginResult::Started(_)));
    assert!(coordinator.admit_for_test().is_err());
    assert_eq!(coordinator.phase_for_test(), ShutdownPhase::Closing);
}

#[test]
fn ready_cannot_be_marked_before_closing() {
    let coordinator = coordinator();
    assert!(!coordinator.mark_ready());
    assert_eq!(coordinator.phase_for_test(), ShutdownPhase::Running);
}
