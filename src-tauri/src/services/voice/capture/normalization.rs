use sherpa_onnx::LinearResampler;
use zeroize::Zeroize;

use crate::services::voice::{errors::VoiceError, limits::VOICE_SAMPLE_RATE};

use super::ring::CaptureChunk;

pub struct Normalizer {
    sample_rate: u32,
    channels: u16,
    resampler: Option<LinearResampler>,
}

impl Normalizer {
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self, VoiceError> {
        let resampler = if sample_rate == VOICE_SAMPLE_RATE {
            None
        } else {
            let input_rate =
                i32::try_from(sample_rate).map_err(|_| VoiceError::invalid_settings())?;
            Some(
                LinearResampler::create(input_rate, VOICE_SAMPLE_RATE as i32)
                    .ok_or_else(VoiceError::configuration_unavailable)?,
            )
        };
        Ok(Self {
            sample_rate,
            channels,
            resampler,
        })
    }

    pub fn process(&self, mut chunk: CaptureChunk, flush: bool) -> Result<Vec<i16>, VoiceError> {
        chunk.validate()?;
        if chunk.sample_rate != self.sample_rate || chunk.channels != self.channels {
            return Err(VoiceError::invalid_settings());
        }
        let channels = usize::from(chunk.channels);
        let mut mono = Vec::with_capacity(chunk.samples.len() / channels);
        for frame in chunk.samples.chunks_exact(channels) {
            mono.push(frame.iter().copied().sum::<f32>() / channels as f32);
        }
        chunk.samples.zeroize();
        let mut normalized = match &self.resampler {
            Some(resampler) => {
                let output = resampler.resample(&mono, flush);
                mono.zeroize();
                output
            }
            None => mono,
        };
        let output = to_pcm(&normalized);
        normalized.zeroize();
        Ok(output)
    }

    pub fn finish(&self) -> Vec<i16> {
        self.resampler
            .as_ref()
            .map(|resampler| {
                let mut tail = resampler.resample(&[], true);
                let output = to_pcm(&tail);
                tail.zeroize();
                output
            })
            .unwrap_or_default()
    }
}

pub fn normalize_chunk(chunk: CaptureChunk) -> Result<Vec<i16>, VoiceError> {
    Normalizer::new(chunk.sample_rate, chunk.channels)?.process(chunk, true)
}

fn to_pcm(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::voice::capture::ring::InputSampleFormat;

    fn chunk(samples: Vec<f32>, rate: u32, channels: u16) -> CaptureChunk {
        CaptureChunk {
            samples,
            sample_rate: rate,
            channels,
            format: InputSampleFormat::F32,
            lost_samples: 0,
        }
    }

    #[test]
    fn mixes_stereo_and_clips() {
        let output = normalize_chunk(chunk(vec![2.0, 0.0, -2.0, 0.0], 16_000, 2)).unwrap();
        assert_eq!(output, [i16::MAX, i16::MIN + 1]);
    }

    #[test]
    fn resamples_common_device_rates_to_sixteen_khz() {
        for rate in [44_100, 48_000] {
            let input = vec![0.25; rate as usize];
            let output = normalize_chunk(chunk(input, rate, 1)).unwrap();
            assert!(
                (15_990..=16_010).contains(&output.len()),
                "{rate}: {}",
                output.len()
            );
        }
    }

    #[test]
    fn low_pass_prevents_high_frequency_aliasing() {
        let input: Vec<f32> = (0..48_000)
            .map(|index| (2.0 * std::f32::consts::PI * 12_000.0 * index as f32 / 48_000.0).sin())
            .collect();
        let output = normalize_chunk(chunk(input, 48_000, 1)).unwrap();
        let peak = output
            .iter()
            .skip(500)
            .take(output.len().saturating_sub(1_000))
            .map(|sample| sample.unsigned_abs())
            .max()
            .unwrap();
        assert!(peak < 1_000, "aliased steady-state peak: {peak}");
    }

    #[test]
    fn streaming_resampler_preserves_duration_across_small_chunks() {
        let normalizer = Normalizer::new(48_000, 1).unwrap();
        let mut output = Vec::new();
        for _ in 0..100 {
            output.extend(
                normalizer
                    .process(chunk(vec![0.25; 480], 48_000, 1), false)
                    .unwrap(),
            );
        }
        output.extend(normalizer.finish());
        assert!((15_990..=16_010).contains(&output.len()));
    }
}
