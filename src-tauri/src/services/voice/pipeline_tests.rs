use super::{
    capture::activity::CaptureStopReason,
    download::VoiceEngine,
    pipeline::{catch_pipeline_panic, effective_language},
    pipeline_stop::{stop_action, StopAction},
    types::VoiceLanguage,
};

#[test]
fn involuntary_micro_stops_transcribe_captured_audio() {
    assert_eq!(
        stop_action(CaptureStopReason::Disconnected),
        StopAction::Validate
    );
    assert_eq!(
        stop_action(CaptureStopReason::HiddenOrLocked),
        StopAction::Validate
    );
    assert_eq!(
        stop_action(CaptureStopReason::NoSpeech),
        StopAction::Discard
    );
}

#[test]
fn cohere_refuses_a_request_without_a_resolved_language() {
    assert!(effective_language(VoiceEngine::CohereTranscribe, &VoiceLanguage::Automatic).is_err());
    assert!(effective_language(
        VoiceEngine::CohereTranscribe,
        &VoiceLanguage::FollowInterface
    )
    .is_err());
    assert!(effective_language(
        VoiceEngine::CohereTranscribe,
        &VoiceLanguage::Language("fr".into())
    )
    .is_ok());
}

#[test]
fn pipeline_panics_become_a_controlled_error() {
    assert!(catch_pipeline_panic(|| panic!("native failure")).is_err());
    assert!(catch_pipeline_panic(|| Ok(())).is_ok());
}
