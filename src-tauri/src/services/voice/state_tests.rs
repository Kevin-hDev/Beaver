use super::state::{cancel_disposition, CancelDisposition, VoiceState};
use super::types::VoicePhase;
use super::{errors::VoiceErrorCode, limits};

#[test]
fn recovery_threshold_counts_speech_only() {
    assert_eq!(cancel_disposition(29_999), CancelDisposition::Discard);
    assert_eq!(cancel_disposition(30_000), CancelDisposition::Recover);
}

#[test]
fn public_error_parameters_are_bounded() {
    let error = super::errors::VoiceError::with_params(
        VoiceErrorCode::InvalidSettings,
        vec![super::errors::VoiceErrorParam {
            key: super::errors::VoiceErrorParamKey::RequiredBytes,
            value: "42".into(),
        }],
    )
    .expect("bounded public parameter");
    assert_eq!(error.params().len(), 1);
    assert!(super::errors::VoiceError::with_params(
        VoiceErrorCode::InvalidSettings,
        vec![super::errors::VoiceErrorParam {
            key: super::errors::VoiceErrorParamKey::AvailableBytes,
            value: "x".repeat(limits::MAX_ERROR_PARAM_CHARS + 1),
        }],
    )
    .is_err());
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
