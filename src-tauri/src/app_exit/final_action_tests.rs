use super::final_action::{run, FinalActionSource};
use super::state::{BeginClosing, ShutdownState};
use super::ExitIntent;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn watchdog_restart_dispatches_restart_once() {
    let state = ShutdownState::new();
    assert_eq!(state.begin_closing(), BeginClosing::Started);
    let calls = AtomicUsize::new(0);

    assert!(run(
        &state,
        ExitIntent::Restart,
        0,
        FinalActionSource::Watchdog,
        |intent, _| {
            assert_eq!(intent, ExitIntent::Restart);
            calls.fetch_add(1, Ordering::AcqRel);
        },
    ));
    assert_eq!(calls.load(Ordering::Acquire), 1);
}

#[test]
fn cleanup_after_watchdog_cannot_dispatch_a_second_action() {
    let state = ShutdownState::new();
    assert_eq!(state.begin_closing(), BeginClosing::Started);
    let calls = AtomicUsize::new(0);
    let dispatch = |_, _| {
        calls.fetch_add(1, Ordering::AcqRel);
    };

    assert!(run(
        &state,
        ExitIntent::Restart,
        0,
        FinalActionSource::Watchdog,
        dispatch,
    ));
    assert!(!run(
        &state,
        ExitIntent::Restart,
        0,
        FinalActionSource::Cleanup,
        dispatch,
    ));
    assert_eq!(calls.load(Ordering::Acquire), 1);
}
