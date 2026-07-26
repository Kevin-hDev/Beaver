use super::view_state::{ViewPhase, ViewState};

#[test]
fn creation_is_accepted_only_once_until_browser_is_ready() {
    let mut state = ViewState::default();

    assert!(state.begin_creation());
    assert!(!state.begin_creation());
    assert_eq!(state.phase(), ViewPhase::Creating);
    assert!(state.mark_ready());
    assert!(!state.begin_creation());
    assert_eq!(state.phase(), ViewPhase::Ready);
}

#[test]
fn failed_creation_can_be_retried_but_closed_browser_cannot() {
    let mut state = ViewState::default();

    assert!(state.begin_creation());
    assert!(state.mark_creation_failed());
    assert!(state.begin_creation());
    assert!(state.mark_ready());
    assert!(state.begin_closing());
    assert!(state.mark_closed());
    assert!(!state.begin_creation());
    assert_eq!(state.phase(), ViewPhase::Closed);
}

#[test]
fn creation_can_be_cancelled_before_browser_becomes_ready() {
    let mut state = ViewState::default();

    assert!(state.begin_creation());
    assert!(state.begin_closing());
    assert!(!state.mark_ready());
    assert!(state.mark_closed());
    assert_eq!(state.phase(), ViewPhase::Closed);
}
