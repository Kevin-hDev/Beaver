use super::voice_probe::{TestedProbeState as ProbeState, VoiceProbeStatus, MAX_LEVEL_HISTORY};

#[test]
fn a_second_probe_cannot_start() {
    let mut state = ProbeState::default();
    assert!(state.begin().is_ok());
    assert_eq!(state.begin().unwrap_err().public_code(), "voice-probe-busy");
}

#[test]
fn a_failed_start_can_be_retried() {
    let mut state = ProbeState::default();
    state.begin().expect("first probe");
    state.fail();

    assert_eq!(state.snapshot().status, VoiceProbeStatus::Error);
    assert!(state.begin().is_ok());
}

#[test]
fn level_history_stays_bounded() {
    let mut state = ProbeState::default();
    state.begin().expect("probe start");
    state.mark_listening();

    for index in 0..(MAX_LEVEL_HISTORY + 20) {
        state.record(index as u64, 0.5);
    }

    let snapshot = state.snapshot();
    assert_eq!(snapshot.levels.len(), MAX_LEVEL_HISTORY);
    assert_eq!(snapshot.sample_count, (MAX_LEVEL_HISTORY + 19) as u64);
}
