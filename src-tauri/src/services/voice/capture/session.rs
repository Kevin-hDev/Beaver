use std::time::Instant;

use zeroize::Zeroize;

use crate::services::voice::{
    audio_buffer::AudioBuffer,
    errors::VoiceError,
    limits::MAX_PCM_SAMPLES,
    types::{VoiceInputDevice, VoiceMaxDuration, VoiceSilenceTimeout},
};
use sherpa_onnx::VoiceActivityDetector;

use super::{
    activity::{automatic_stop_reason, CaptureStopReason, SpeechClock},
    level::LevelFrame,
    normalization::Normalizer,
    stream::{open_input_stream_if, CaptureStream},
    window_events::{session_is_locked, WindowEventState},
};

pub struct CapturePoll {
    pub level: LevelFrame,
    pub speech_ms: u64,
    pub stop_reason: Option<CaptureStopReason>,
}

pub struct CaptureSession {
    stream: CaptureStream,
    normalizer: Normalizer,
    audio: AudioBuffer,
    speech: SpeechClock,
    started: Instant,
    last_speech_ms: u64,
    generation: u64,
}

impl CaptureSession {
    pub fn open(
        device: &VoiceInputDevice,
        cancelled: bool,
        foreground: bool,
        generation: u64,
    ) -> Result<Self, VoiceError> {
        let stream = open_input_stream_if(device, cancelled, foreground)?;
        let normalizer = Normalizer::new(stream.sample_rate(), stream.channels())?;
        stream.start()?;
        Ok(Self {
            stream,
            normalizer,
            audio: AudioBuffer::default(),
            speech: SpeechClock::default(),
            started: Instant::now(),
            last_speech_ms: 0,
            generation,
        })
    }

    pub fn poll(
        &mut self,
        vad: &VoiceActivityDetector,
        windows: &WindowEventState,
        silence_timeout: VoiceSilenceTimeout,
        max_duration: VoiceMaxDuration,
    ) -> Result<CapturePoll, VoiceError> {
        let chunk = self.stream.ring().drain();
        let lost_samples = chunk.lost_samples;
        let mut pcm = self.normalizer.process(chunk, false)?;
        let remaining = MAX_PCM_SAMPLES.saturating_sub(self.audio.samples().len());
        pcm.truncate(remaining);
        self.audio.append(&pcm, lost_samples)?;
        let level = LevelFrame::from_pcm(&pcm, self.generation, lost_samples);
        let mut waveform: Vec<f32> = pcm
            .iter()
            .map(|sample| f32::from(*sample) / i16::MAX as f32)
            .collect();
        pcm.zeroize();
        let speaking = self.speech.observe_vad(vad, &waveform, false);
        waveform.zeroize();

        let elapsed_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if speaking {
            self.last_speech_ms = elapsed_ms;
        }
        let silence_ms = elapsed_ms.saturating_sub(self.last_speech_ms);
        let hidden = windows.take_stop_signal().is_some() || session_is_locked();
        let stop_reason = automatic_stop_reason(
            elapsed_ms,
            self.speech,
            silence_ms,
            silence_timeout,
            max_duration,
            self.stream.disconnected(),
            hidden,
        );
        Ok(CapturePoll {
            level,
            speech_ms: self.speech.spoken_ms(),
            stop_reason,
        })
    }

    pub fn finish(mut self, vad: &VoiceActivityDetector) -> Result<(AudioBuffer, u64), VoiceError> {
        let mut tail = self.normalizer.finish();
        let remaining = MAX_PCM_SAMPLES.saturating_sub(self.audio.samples().len());
        tail.truncate(remaining);
        self.audio.append(&tail, 0)?;
        tail.zeroize();
        self.speech.observe_vad(vad, &[], true);
        Ok((self.audio, self.speech.spoken_ms()))
    }
}
