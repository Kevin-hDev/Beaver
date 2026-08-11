use super::policy::ShutdownPolicy;
use super::state::ShutdownPhase;
use super::ultimate::{RawExitActions, UltimateExit};
use super::{AppExitCoordinator, BeginResult, ExitIntent};
use crate::services::browser::CefShutdownBarrier;
use std::sync::{Arc, Barrier};
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

    assert!(matches!(
        coordinator.begin(4),
        BeginResult::Started(_, ExitIntent::Exit)
    ));
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
    assert!(matches!(
        coordinator.begin(0),
        BeginResult::Started(_, ExitIntent::Exit)
    ));
    assert!(coordinator.admit_for_test().is_err());
    assert_eq!(coordinator.phase_for_test(), ShutdownPhase::Closing);
}

#[test]
fn cef_barrier_timeout_keeps_coordinated_shutdown_running() {
    let coordinator = coordinator();

    let result = coordinator.begin_with_cef_close(0, |_, _| CefShutdownBarrier::TimedOut);

    assert!(matches!(result, BeginResult::Started(_, ExitIntent::Exit)));
    assert!(coordinator.ultimate_is_armed_for_test());
    assert_eq!(coordinator.phase_for_test(), ShutdownPhase::Closing);
}

#[test]
fn ready_cannot_be_marked_before_closing() {
    let coordinator = coordinator();
    assert!(!coordinator.mark_ready());
    assert_eq!(coordinator.phase_for_test(), ShutdownPhase::Running);
}

#[test]
fn a_closed_registry_with_running_state_is_an_invariant_failure() {
    let coordinator = coordinator();
    coordinator.close_registry_for_test();

    assert_eq!(coordinator.begin(0), BeginResult::InvariantViolation);
}

#[test]
fn concurrent_close_requests_never_observe_a_partial_transition() {
    let coordinator = Arc::new(coordinator());
    let barrier = Arc::new(Barrier::new(17));
    let requests = (0..16)
        .map(|_| {
            let coordinator = Arc::clone(&coordinator);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                coordinator.begin(0)
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = requests
        .into_iter()
        .map(|request| request.join().expect("close request"))
        .collect::<Vec<_>>();

    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, BeginResult::Started(_, ExitIntent::Exit)))
            .count(),
        1
    );
    assert!(results.iter().all(|result| {
        matches!(
            result,
            BeginResult::Started(_, ExitIntent::Exit) | BeginResult::Waiting
        )
    }));
}

#[test]
fn first_shutdown_intent_is_immutable_and_restart_keeps_the_raw_exit_safe() {
    let coordinator = coordinator();

    assert_eq!(
        super::request_flow::requested_intent(Some(super::BEAVER_RESTART_REQUEST_CODE)),
        (ExitIntent::Restart, 0)
    );

    let started =
        coordinator.begin_with_intent(ExitIntent::Restart, 0, |_, _| CefShutdownBarrier::Drained);

    assert!(matches!(
        started,
        BeginResult::Started(_, ExitIntent::Restart)
    ));
    assert_eq!(coordinator.intent_for_test(), Some(ExitIntent::Restart));
    assert_eq!(coordinator.begin(9), BeginResult::Waiting);
    assert_eq!(coordinator.intent_for_test(), Some(ExitIntent::Restart));
}

#[test]
fn restart_button_uses_an_interceptable_beaver_sentinel() {
    let mut requested = None;

    super::request_restart_with(|code| requested = Some(code));

    let code = requested.expect("restart exit request");
    assert_ne!(code, tauri::RESTART_EXIT_CODE);
    assert_eq!(
        super::request_flow::requested_intent(Some(code)),
        (ExitIntent::Restart, 0)
    );
}

#[test]
fn final_tauri_restart_exit_does_not_start_cleanup_again() {
    let coordinator = coordinator();
    assert!(matches!(
        coordinator.begin_with_intent(ExitIntent::Restart, 0, |_, _| {
            CefShutdownBarrier::Drained
        }),
        BeginResult::Started(_, ExitIntent::Restart)
    ));
    assert!(coordinator.mark_ready());

    let (intent, exit_code) = super::request_flow::requested_intent(Some(tauri::RESTART_EXIT_CODE));

    assert_eq!(intent, ExitIntent::Exit);
    assert_eq!(exit_code, tauri::RESTART_EXIT_CODE);
    assert_eq!(
        coordinator.begin_with_intent(intent, exit_code, |_, _| {
            panic!("CEF cleanup must not start twice")
        }),
        BeginResult::Ready
    );
    assert_eq!(coordinator.intent_for_test(), Some(ExitIntent::Restart));
}
