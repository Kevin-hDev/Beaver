use super::activity::{
    automatic_stop_reason, max_duration_ms, no_speech_guard_reached, silence_timeout_ms,
    CaptureStopReason, SpeechClock,
};
use crate::services::voice::types::{VoiceMaxDuration, VoiceSilenceTimeout};

#[test]
fn thinking_after_first_speech_never_triggers_the_initial_guard() {
    assert!(!no_speech_guard_reached(119_999, false));
    assert!(no_speech_guard_reached(120_000, false));
    assert!(!no_speech_guard_reached(120_001, true));
}

#[test]
fn silence_values_are_exact_and_never_is_unbounded() {
    assert_eq!(
        silence_timeout_ms(VoiceSilenceTimeout::ThreeSeconds),
        Some(3_000)
    );
    assert_eq!(
        silence_timeout_ms(VoiceSilenceTimeout::ThirtySeconds),
        Some(30_000)
    );
    assert_eq!(silence_timeout_ms(VoiceSilenceTimeout::Never), None);
}

#[test]
fn all_duration_values_stay_under_the_rust_hard_limit() {
    assert_eq!(max_duration_ms(VoiceMaxDuration::Two), 120_000);
    assert_eq!(max_duration_ms(VoiceMaxDuration::Five), 300_000);
    assert_eq!(max_duration_ms(VoiceMaxDuration::Ten), 600_000);
    assert_eq!(max_duration_ms(VoiceMaxDuration::Twenty), 1_200_000);
    assert_eq!(max_duration_ms(VoiceMaxDuration::Thirty), 1_800_000);
}

#[test]
fn speech_clock_saturates_and_keeps_only_outer_silence_out_of_inference() {
    let mut clock = SpeechClock::default();
    clock.add_segment(160_000, 8_000);
    assert_eq!(clock.spoken_ms(), 500);
    assert_eq!(clock.inference_range(200_000), Some(152_800..175_200));
    clock.add_spoken_ms(u64::MAX);
    clock.add_spoken_ms(1);
    assert_eq!(clock.spoken_ms(), u64::MAX);
}

#[test]
fn short_manual_capture_gets_only_the_missing_transcription_tail() {
    let mut clock = SpeechClock::default();
    clock.add_segment(0, 1_600);
    assert_eq!(clock.missing_transcription_tail(1_600), 4_000);
    assert_eq!(clock.missing_transcription_tail(3_200), 2_400);
    assert_eq!(clock.missing_transcription_tail(5_600), 0);
}

#[test]
fn stop_reasons_distinguish_no_speech_silence_limit_and_disconnect() {
    let none = SpeechClock::default();
    assert_eq!(
        automatic_stop_reason(
            120_000,
            none,
            120_000,
            VoiceSilenceTimeout::Never,
            VoiceMaxDuration::Thirty,
            false,
            false
        ),
        Some(CaptureStopReason::NoSpeech)
    );
    let mut spoken = SpeechClock::default();
    spoken.add_spoken_ms(500);
    assert_eq!(
        automatic_stop_reason(
            6_000,
            spoken,
            5_000,
            VoiceSilenceTimeout::FiveSeconds,
            VoiceMaxDuration::Thirty,
            false,
            false
        ),
        Some(CaptureStopReason::Silence)
    );
    assert_eq!(
        automatic_stop_reason(
            1,
            spoken,
            0,
            VoiceSilenceTimeout::Never,
            VoiceMaxDuration::Thirty,
            true,
            false
        ),
        Some(CaptureStopReason::Disconnected)
    );
}
