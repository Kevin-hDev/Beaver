use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    FromSample, Sample, SampleFormat, SizedSample,
};

use crate::services::voice::{errors::VoiceError, limits, types::VoiceInputDevice};

use super::{
    device,
    ring::{CaptureChunk, InputRing, InputSampleFormat},
};

pub struct CaptureStream {
    stream: cpal::Stream,
    ring: InputRing,
    disconnected: Arc<AtomicBool>,
}

impl CaptureStream {
    pub fn start(&self) -> Result<(), VoiceError> {
        self.stream
            .play()
            .map_err(|_| VoiceError::configuration_unavailable())
    }

    pub fn ring(&self) -> &InputRing {
        &self.ring
    }

    pub fn disconnected(&self) -> bool {
        self.disconnected.load(Ordering::Acquire)
    }

    pub fn sample_rate(&self) -> u32 {
        self.ring.sample_rate()
    }

    pub fn channels(&self) -> u16 {
        self.ring.channels()
    }

    pub fn finish(self) -> CaptureChunk {
        drop(self.stream);
        self.ring.drain()
    }
}

#[cfg(test)]
pub fn open_input_stream(selection: &VoiceInputDevice) -> Result<CaptureStream, VoiceError> {
    open_input_stream_if(selection, false, true)
}

pub fn open_input_stream_if(
    selection: &VoiceInputDevice,
    cancelled: bool,
    foreground: bool,
) -> Result<CaptureStream, VoiceError> {
    if !capture_start_allowed(cancelled, foreground) {
        return Err(VoiceError::invalid_transition());
    }
    let device = device::resolve(selection)?;
    let supported = device
        .default_input_config()
        .map_err(|_| VoiceError::configuration_unavailable())?;
    let config = supported.config();
    let channels = config.channels;
    let sample_rate = config.sample_rate;
    let one_second = usize::try_from(sample_rate)
        .ok()
        .and_then(|rate| rate.checked_mul(usize::from(channels)))
        .ok_or_else(VoiceError::invalid_settings)?;
    let max_samples = limits::MAX_INPUT_RING_BYTES / size_of::<f32>();
    if one_second > max_samples {
        return Err(VoiceError::configuration_unavailable());
    }
    let capacity = one_second;
    let disconnected = Arc::new(AtomicBool::new(false));
    let format = match supported.sample_format() {
        SampleFormat::F32 => InputSampleFormat::F32,
        SampleFormat::I16 => InputSampleFormat::I16,
        SampleFormat::I32 => InputSampleFormat::I32,
        SampleFormat::U16 => InputSampleFormat::U16,
        _ => return Err(VoiceError::configuration_unavailable()),
    };
    let ring = InputRing::new(capacity, sample_rate, channels)?;
    let stream = match format {
        InputSampleFormat::F32 => build::<f32>(&device, &config, &ring, &disconnected),
        InputSampleFormat::I16 => build::<i16>(&device, &config, &ring, &disconnected),
        InputSampleFormat::I32 => build::<i32>(&device, &config, &ring, &disconnected),
        InputSampleFormat::U16 => build::<u16>(&device, &config, &ring, &disconnected),
    }?;
    Ok(CaptureStream {
        stream,
        ring,
        disconnected,
    })
}

const fn capture_start_allowed(cancelled: bool, foreground: bool) -> bool {
    !cancelled && foreground
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    ring: &InputRing,
    disconnected: &Arc<AtomicBool>,
) -> Result<cpal::Stream, VoiceError>
where
    T: SizedSample + Sample,
    f32: FromSample<T>,
{
    let callback_ring = ring.clone();
    let failed = Arc::clone(disconnected);
    device
        .build_input_stream(
            *config,
            move |data: &[T], _| {
                callback_ring.push(data.iter().copied().map(f32::from_sample), data.len());
            },
            move |_| {
                failed.store(true, Ordering::Release);
            },
            None,
        )
        .map_err(|_| VoiceError::configuration_unavailable())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_or_background_preparation_never_opens_the_device() {
        assert!(!capture_start_allowed(true, true));
        assert!(!capture_start_allowed(false, false));
        assert!(capture_start_allowed(false, true));
    }
}
