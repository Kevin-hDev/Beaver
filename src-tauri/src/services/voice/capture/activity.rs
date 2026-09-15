use crate::services::voice::{
    limits::{MAX_CAPTURE_SECONDS, NO_SPEECH_GUARD_MS},
    types::{VoiceMaxDuration, VoiceSilenceTimeout},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureStopReason {
    Validated,
    Silence,
    DurationLimit,
    NoSpeech,
    Disconnected,
    HiddenOrLocked,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SpeechClock {
    spoken_ms: u64,
    detected: bool,
}

impl SpeechClock {
    pub fn add_segment(&mut self, duration_ms: u64) {
        self.spoken_ms = self.spoken_ms.saturating_add(duration_ms);
    }

    pub const fn spoken_ms(self) -> u64 {
        self.spoken_ms
    }

    pub const fn has_spoken(self) -> bool {
        self.detected || self.spoken_ms != 0
    }

    pub fn observe_vad(
        &mut self,
        vad: &sherpa_onnx::VoiceActivityDetector,
        samples: &[f32],
        flush: bool,
    ) -> bool {
        vad.accept_waveform(samples);
        let currently_speaking = vad.detected();
        self.detected |= currently_speaking;
        if flush {
            vad.flush();
        }
        while let Some(segment) = vad.front() {
            self.detected = true;
            let duration_ms = u64::try_from(segment.n())
                .unwrap_or(0)
                .saturating_mul(1_000)
                / 16_000;
            self.add_segment(duration_ms);
            drop(segment);
            vad.pop();
        }
        currently_speaking
    }
}

pub const fn no_speech_guard_reached(elapsed_ms: u64, has_spoken: bool) -> bool {
    !has_spoken && elapsed_ms >= NO_SPEECH_GUARD_MS
}

pub const fn silence_timeout_ms(timeout: VoiceSilenceTimeout) -> Option<u64> {
    match timeout {
        VoiceSilenceTimeout::ThreeSeconds => Some(3_000),
        VoiceSilenceTimeout::FiveSeconds => Some(5_000),
        VoiceSilenceTimeout::TenSeconds => Some(10_000),
        VoiceSilenceTimeout::TwentySeconds => Some(20_000),
        VoiceSilenceTimeout::ThirtySeconds => Some(30_000),
        VoiceSilenceTimeout::Never => None,
    }
}

pub const fn max_duration_ms(duration: VoiceMaxDuration) -> u64 {
    let minutes = match duration {
        VoiceMaxDuration::Two => 2,
        VoiceMaxDuration::Five => 5,
        VoiceMaxDuration::Ten => 10,
        VoiceMaxDuration::Twenty => 20,
        VoiceMaxDuration::Thirty => 30,
    };
    let configured = minutes * 60 * 1_000;
    let hard_limit = MAX_CAPTURE_SECONDS as u64 * 1_000;
    if configured < hard_limit {
        configured
    } else {
        hard_limit
    }
}

pub const fn automatic_stop_reason(
    elapsed_ms: u64,
    clock: SpeechClock,
    silence_ms: u64,
    silence_timeout: VoiceSilenceTimeout,
    max_duration: VoiceMaxDuration,
    disconnected: bool,
    hidden_or_locked: bool,
) -> Option<CaptureStopReason> {
    if disconnected {
        return Some(CaptureStopReason::Disconnected);
    }
    if hidden_or_locked {
        return Some(CaptureStopReason::HiddenOrLocked);
    }
    if no_speech_guard_reached(elapsed_ms, clock.has_spoken()) {
        return Some(CaptureStopReason::NoSpeech);
    }
    if elapsed_ms >= max_duration_ms(max_duration) {
        return Some(CaptureStopReason::DurationLimit);
    }
    match silence_timeout_ms(silence_timeout) {
        Some(limit) if clock.has_spoken() && silence_ms >= limit => {
            Some(CaptureStopReason::Silence)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn speech_clock_saturates_instead_of_wrapping() {
        let mut clock = SpeechClock::default();
        clock.add_segment(u64::MAX);
        clock.add_segment(1);
        assert_eq!(clock.spoken_ms(), u64::MAX);
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
                false,
            ),
            Some(CaptureStopReason::NoSpeech)
        );
        let mut spoken = SpeechClock::default();
        spoken.add_segment(500);
        assert_eq!(
            automatic_stop_reason(
                6_000,
                spoken,
                5_000,
                VoiceSilenceTimeout::FiveSeconds,
                VoiceMaxDuration::Thirty,
                false,
                false,
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
                false,
            ),
            Some(CaptureStopReason::Disconnected)
        );
    }
}
