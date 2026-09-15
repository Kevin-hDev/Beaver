use std::time::Instant;

use zeroize::Zeroize;

use crate::services::voice::{
    audio_buffer::AudioBuffer,
    errors::VoiceError,
    limits::MAX_PCM_SAMPLES,
    types::{VoiceInputDevice, VoiceInputGain, VoiceMaxDuration, VoiceSilenceTimeout},
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
        gain: VoiceInputGain,
        cancelled: bool,
        foreground: bool,
        generation: u64,
    ) -> Result<Self, VoiceError> {
        let stream = open_input_stream_if(device, cancelled, foreground)?;
        let normalizer = Normalizer::new(stream.sample_rate(), stream.channels(), gain)?;
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
        let speech_observed = self.speech.observe_vad(vad, &waveform, false);
        waveform.zeroize();

        let elapsed_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if speech_observed {
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
        // Fermer le flux avant le dernier drain garantit que le clic de
        // validation ne laisse pas la fin d'un mot dans le tampon du micro.
        let chunk = self.stream.finish();
        let lost_samples = chunk.lost_samples;
        let mut tail = self.normalizer.process(chunk, true)?;
        let remaining = MAX_PCM_SAMPLES.saturating_sub(self.audio.samples().len());
        tail.truncate(remaining);
        self.audio.append(&tail, lost_samples)?;
        let mut waveform: Vec<f32> = tail
            .iter()
            .map(|sample| f32::from(*sample) / i16::MAX as f32)
            .collect();
        tail.zeroize();
        self.speech.observe_vad(vad, &waveform, true);
        waveform.zeroize();
        if let Some(range) = self.speech.inference_range(self.audio.samples().len()) {
            self.audio.trim_to(range)?;
        }
        Ok((self.audio, self.speech.spoken_ms()))
    }
}
