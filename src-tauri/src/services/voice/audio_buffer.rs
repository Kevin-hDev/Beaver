use std::ops::Range;
use zeroize::Zeroize;

use crate::services::voice::{errors::VoiceError, limits::MAX_PCM_SAMPLES};

#[derive(Default)]
pub struct AudioBuffer {
    samples: Vec<i16>,
    lost_samples: u64,
}

impl AudioBuffer {
    pub fn append(&mut self, samples: &[i16], lost_samples: u64) -> Result<(), VoiceError> {
        let next_len = self
            .samples
            .len()
            .checked_add(samples.len())
            .ok_or_else(VoiceError::invalid_settings)?;
        if next_len > MAX_PCM_SAMPLES {
            return Err(VoiceError::invalid_settings());
        }
        self.samples.extend_from_slice(samples);
        self.lost_samples = self.lost_samples.saturating_add(lost_samples);
        Ok(())
    }

    pub fn samples(&self) -> &[i16] {
        &self.samples
    }

    pub fn lost_samples(&self) -> u64 {
        self.lost_samples
    }

    pub fn trim_to(&mut self, range: Range<usize>) -> Result<(), VoiceError> {
        if range.start >= range.end || range.end > self.samples.len() {
            return Err(VoiceError::invalid_settings());
        }
        self.samples[..range.start].zeroize();
        self.samples[range.end..].zeroize();
        let kept = range.len();
        self.samples.copy_within(range, 0);
        self.samples[kept..].zeroize();
        self.samples.truncate(kept);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.samples.zeroize();
        self.samples.clear();
    }
}

impl Drop for AudioBuffer {
    fn drop(&mut self) {
        self.samples.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_is_bounded_to_thirty_minutes() {
        let mut buffer = AudioBuffer::default();
        buffer.samples.resize(MAX_PCM_SAMPLES, 1);
        assert!(buffer.append(&[1], 0).is_err());
        assert_eq!(std::mem::size_of_val(buffer.samples()), 57_600_000);
    }

    #[test]
    fn loss_count_is_saturating() {
        let mut buffer = AudioBuffer::default();
        buffer.append(&[1], u64::MAX).unwrap();
        buffer.append(&[2], 1).unwrap();
        assert_eq!(buffer.lost_samples(), u64::MAX);
    }

    #[test]
    fn trim_zeroizes_outer_silence_and_keeps_the_requested_range() {
        let mut buffer = AudioBuffer::default();
        buffer.append(&[0, 1, 2, 3, 0], 0).unwrap();
        buffer.trim_to(1..4).unwrap();
        assert_eq!(buffer.samples(), [1, 2, 3]);
    }
}
