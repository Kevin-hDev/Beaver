use super::liveness_policy::{MacLivenessDecision, MacLivenessError, MacLivenessState};
use super::process_state::MacProcessObservation;

const MS: u64 = 1_000_000;

#[test]
fn unknown_expires_at_exactly_two_hundred_fifty_milliseconds() {
    let mut state = MacLivenessState::new();
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 10 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 259 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 260 * MS, None),
        Ok(MacLivenessDecision::Expired)
    );
}

#[test]
fn decisive_observation_clears_the_previous_unknown_budget() {
    let mut state = MacLivenessState::new();
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 10 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Alive, 20 * MS, None),
        Ok(MacLivenessDecision::Alive)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 259 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 509 * MS, None),
        Ok(MacLivenessDecision::Expired)
    );
}

#[test]
fn stopped_clears_the_previous_unknown_budget() {
    let mut state = MacLivenessState::new();
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 10 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Stopped, 20 * MS, None),
        Ok(MacLivenessDecision::Stopped)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 259 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
}

#[test]
fn two_helpers_never_share_a_budget() {
    let mut first = MacLivenessState::new();
    let mut second = MacLivenessState::new();
    assert_eq!(
        first.apply(MacProcessObservation::Unknown, 10 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        second.apply(MacProcessObservation::Unknown, 259 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        first.apply(MacProcessObservation::Unknown, 260 * MS, None),
        Ok(MacLivenessDecision::Expired)
    );
    assert_eq!(
        second.apply(MacProcessObservation::Unknown, 508 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
}

#[test]
fn nearer_closing_cap_wins_without_reissuing_time() {
    let mut state = MacLivenessState::new();
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 10 * MS, None),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 99 * MS, Some(100 * MS)),
        Ok(MacLivenessDecision::Pending)
    );
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 100 * MS, Some(100 * MS)),
        Ok(MacLivenessDecision::Expired)
    );
}

#[test]
fn invalid_or_overflowing_clock_fails_closed() {
    let mut invalid = MacLivenessState::new();
    assert_eq!(
        invalid.apply(MacProcessObservation::Unknown, 0, None),
        Err(MacLivenessError::Clock)
    );
    let mut overflow = MacLivenessState::new();
    assert_eq!(
        overflow.apply(MacProcessObservation::Unknown, u64::MAX, None),
        Err(MacLivenessError::Clock)
    );
}

#[test]
fn an_invalid_closing_cap_fails_closed() {
    let mut state = MacLivenessState::new();
    assert_eq!(
        state.apply(MacProcessObservation::Unknown, 10 * MS, Some(0)),
        Err(MacLivenessError::Clock)
    );
}
