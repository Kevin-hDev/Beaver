use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use crate::services::voice::{errors::VoiceError, limits};
use zeroize::Zeroize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputSampleFormat {
    F32,
    I16,
    I32,
    U16,
}

#[derive(Debug)]
pub struct CaptureChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub lost_samples: u64,
}

impl Drop for CaptureChunk {
    fn drop(&mut self) {
        self.samples.zeroize();
    }
}

impl CaptureChunk {
    pub fn validate(&self) -> Result<(), VoiceError> {
        if self.channels == 0
            || self.channels > limits::MAX_INPUT_CHANNELS
            || !(limits::MIN_INPUT_SAMPLE_RATE..=limits::MAX_INPUT_SAMPLE_RATE)
                .contains(&self.sample_rate)
            || !self
                .samples
                .len()
                .is_multiple_of(usize::from(self.channels))
        {
            Err(VoiceError::invalid_settings())
        } else {
            Ok(())
        }
    }
}

#[derive(Clone)]
pub struct InputRing {
    inner: Arc<Mutex<RingState>>,
    lost: Arc<AtomicU64>,
    sample_rate: u32,
    channels: u16,
}

struct RingState {
    samples: Box<[f32]>,
    read: usize,
    len: usize,
}

impl InputRing {
    pub fn new(
        capacity_samples: usize,
        sample_rate: u32,
        channels: u16,
    ) -> Result<Self, VoiceError> {
        let bytes = capacity_samples
            .checked_mul(size_of::<f32>())
            .ok_or_else(VoiceError::invalid_settings)?;
        if capacity_samples == 0
            || bytes > limits::MAX_INPUT_RING_BYTES
            || channels == 0
            || channels > limits::MAX_INPUT_CHANNELS
            || !(limits::MIN_INPUT_SAMPLE_RATE..=limits::MAX_INPUT_SAMPLE_RATE)
                .contains(&sample_rate)
        {
            return Err(VoiceError::invalid_settings());
        }
        Ok(Self {
            inner: Arc::new(Mutex::new(RingState {
                samples: vec![0.0; capacity_samples].into_boxed_slice(),
                read: 0,
                len: 0,
            })),
            lost: Arc::new(AtomicU64::new(0)),
            sample_rate,
            channels,
        })
    }

    pub fn push<I: Iterator<Item = f32>>(&self, samples: I, count: usize) {
        let Ok(mut state) = self.inner.try_lock() else {
            self.count_loss(count);
            return;
        };
        for sample in samples.take(count) {
            let capacity = state.samples.len();
            if state.len == capacity {
                state.read = (state.read + 1) % capacity;
                self.count_loss(1);
                state.len -= 1;
            }
            let write = (state.read + state.len) % capacity;
            state.samples[write] = sample;
            state.len += 1;
        }
    }

    fn count_loss(&self, count: usize) {
        let count = u64::try_from(count).unwrap_or(u64::MAX);
        let _ = self
            .lost
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.saturating_add(count))
            });
    }

    pub fn drain(&self) -> CaptureChunk {
        let mut state = self.inner.lock().unwrap_or_else(|error| error.into_inner());
        let mut samples = Vec::with_capacity(state.len);
        while state.len != 0 {
            let read = state.read;
            samples.push(state.samples[read]);
            state.samples[read] = 0.0;
            state.read = (state.read + 1) % state.samples.len();
            state.len -= 1;
        }
        let lost_samples = self.lost.swap(0, Ordering::Relaxed);
        CaptureChunk {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
            lost_samples,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl Drop for RingState {
    fn drop(&mut self) {
        self.samples.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overwrites_oldest_and_counts_each_loss() {
        let ring = InputRing::new(3, 48_000, 1).unwrap();
        ring.push([1.0, 2.0, 3.0, 4.0].into_iter(), 4);
        let chunk = ring.drain();
        assert_eq!(chunk.samples, [2.0, 3.0, 4.0]);
        assert_eq!(chunk.lost_samples, 1);
    }

    #[test]
    fn refuses_unbounded_or_invalid_capture_parameters() {
        assert!(InputRing::new(0, 48_000, 1).is_err());
        assert!(InputRing::new(1, 1, 1).is_err());
        assert!(InputRing::new(1, 48_000, 9).is_err());
        assert!(InputRing::new(
            limits::MAX_INPUT_RING_BYTES / size_of::<f32>() + 1,
            48_000,
            1,
        )
        .is_err());
    }

    #[test]
    fn a_busy_consumer_counts_every_rejected_sample() {
        let ring = InputRing::new(3, 48_000, 1).unwrap();
        let guard = ring.inner.lock().unwrap();
        ring.push([1.0, 2.0].into_iter(), 2);
        drop(guard);
        assert_eq!(ring.drain().lost_samples, 2);
    }
}
