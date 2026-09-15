use super::state::{cancel_disposition, CancelDisposition, VoiceState};
use super::types::VoicePhase;

#[test]
fn recovery_threshold_counts_speech_only() {
    assert_eq!(cancel_disposition(29_999), CancelDisposition::Discard);
    assert_eq!(cancel_disposition(30_000), CancelDisposition::Recover);
}

#[test]
fn occupied_phases_refuse_a_second_operation_and_lock_the_right_resources() {
    let mut state = VoiceState::default();
    state.begin().expect("first preparation");
    assert!(state.begin().is_err());

    for phase in [
        VoicePhase::Preparing,
        VoicePhase::Listening,
        VoicePhase::Transcribing,
        VoicePhase::Recovering,
        VoicePhase::Stopping,
    ] {
        state.set_phase_for_test(phase);
        assert!(state.blocks_new_operation());
        assert!(state.locks_models());
    }

    state.set_phase_for_test(VoicePhase::Delivering);
    assert!(state.blocks_new_operation());
    assert!(!state.locks_models());

    state.finish();
    assert!(!state.blocks_new_operation());
    assert!(!state.locks_models());
    state
        .begin()
        .expect("ready recovery no longer occupies work");
}
