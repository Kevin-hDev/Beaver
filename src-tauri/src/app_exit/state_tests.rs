use super::state::{BeginClosing, ShutdownPhase, ShutdownState};
use std::sync::Arc;

#[test]
fn state_only_moves_forward() {
    let state = ShutdownState::new();
    assert_eq!(state.phase(), ShutdownPhase::Running);
    assert_eq!(state.begin_closing(), BeginClosing::Started);
    assert_eq!(state.begin_closing(), BeginClosing::AlreadyClosing);
    assert!(state.mark_ready());
    assert_eq!(state.phase(), ShutdownPhase::ReadyToExit);
    assert_eq!(state.begin_closing(), BeginClosing::Ready);
    assert!(!state.mark_ready());
}

#[test]
fn ready_cannot_skip_closing() {
    let state = ShutdownState::new();
    assert!(!state.mark_ready());
    assert_eq!(state.phase(), ShutdownPhase::Running);
}

#[test]
fn concurrent_requests_have_one_owner() {
    let state = Arc::new(ShutdownState::new());
    let threads = (0..16)
        .map(|_| {
            let state = Arc::clone(&state);
            std::thread::spawn(move || state.begin_closing())
        })
        .collect::<Vec<_>>();
    let results = threads
        .into_iter()
        .map(|thread| thread.join().expect("request thread"))
        .collect::<Vec<_>>();

    assert_eq!(
        results
            .iter()
            .filter(|result| **result == BeginClosing::Started)
            .count(),
        1
    );
    assert_eq!(state.phase(), ShutdownPhase::Closing);
}
