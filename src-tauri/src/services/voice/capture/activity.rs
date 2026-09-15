use crate::services::voice::{
    limits::{MAX_CAPTURE_SECONDS, NO_SPEECH_GUARD_MS, VOICE_SAMPLE_RATE},
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
    first_speech_sample: Option<usize>,
    last_speech_sample: usize,
}

impl SpeechClock {
    pub(crate) fn add_spoken_ms(&mut self, duration_ms: u64) {
        self.spoken_ms = self.spoken_ms.saturating_add(duration_ms);
    }

    pub(crate) fn add_segment(&mut self, start: usize, samples: usize) {
        self.detected = true;
        self.first_speech_sample = Some(
            self.first_speech_sample
                .map_or(start, |first| first.min(start)),
        );
        self.last_speech_sample = self.last_speech_sample.max(start.saturating_add(samples));
        self.add_spoken_ms(
            u64::try_from(samples)
                .unwrap_or(u64::MAX)
                .saturating_mul(1_000)
                / u64::from(VOICE_SAMPLE_RATE),
        );
    }

    pub const fn spoken_ms(self) -> u64 {
        self.spoken_ms
    }

    pub const fn has_spoken(self) -> bool {
        self.detected || self.spoken_ms != 0
    }

    pub fn inference_range(self, total_samples: usize) -> Option<std::ops::Range<usize>> {
        const CONTEXT_SAMPLES: usize = 450 * VOICE_SAMPLE_RATE as usize / 1_000;
        let start = self.first_speech_sample?.saturating_sub(CONTEXT_SAMPLES);
        let end = self
            .last_speech_sample
            .saturating_add(CONTEXT_SAMPLES)
            .min(total_samples);
        (start < end).then_some(start..end)
    }

    pub fn observe_vad(
        &mut self,
        vad: &sherpa_onnx::VoiceActivityDetector,
        samples: &[f32],
        flush: bool,
    ) -> bool {
        vad.accept_waveform(samples);
        let currently_speaking = vad.detected();
        let mut speech_observed = currently_speaking;
        self.detected |= currently_speaking;
        if flush {
            vad.flush();
        }
        while let Some(segment) = vad.front() {
            speech_observed = true;
            if let (Ok(start), Ok(samples)) = (
                usize::try_from(segment.start()),
                usize::try_from(segment.n()),
            ) {
                self.add_segment(start, samples);
            }
            drop(segment);
            vad.pop();
        }
        speech_observed
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
