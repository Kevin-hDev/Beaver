use super::voice_probe_state::VoiceProbeError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream};
use std::sync::mpsc;

const LEVEL_CHANNEL_CAPACITY: usize = 8;

pub(super) enum InputUpdate {
    Samples { count: u64, level: f32 },
    Failed,
}

pub(super) fn build_input_stream() -> Result<(Stream, mpsc::Receiver<InputUpdate>), VoiceProbeError>
{
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or(VoiceProbeError::Unavailable)?;
    let config = device
        .default_input_config()
        .map_err(|_| VoiceProbeError::Unavailable)?;
    let (sender, receiver) = mpsc::sync_channel(LEVEL_CHANNEL_CAPACITY);
    let stream_config = config.config();
    let stream = match config.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, sender),
        SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, sender),
        SampleFormat::I32 => build_stream::<i32>(&device, &stream_config, sender),
        SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, sender),
        _ => Err(VoiceProbeError::Unavailable),
    }?;
    Ok((stream, receiver))
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sender: mpsc::SyncSender<InputUpdate>,
) -> Result<Stream, VoiceProbeError>
where
    T: SizedSample + Sample,
    f32: FromSample<T>,
{
    let error_sender = sender.clone();
    device
        .build_input_stream(
            *config,
            move |data: &[T], _| {
                let level = data
                    .iter()
                    .copied()
                    .map(f32::from_sample)
                    .map(f32::abs)
                    .fold(0.0, f32::max);
                let count = u64::try_from(data.len()).unwrap_or(u64::MAX);
                let _ = sender.try_send(InputUpdate::Samples { count, level });
            },
            move |_| {
                let _ = error_sender.try_send(InputUpdate::Failed);
                ::log::warn!("voice_probe_stream_failed");
            },
            None,
        )
        .map_err(|_| VoiceProbeError::Unavailable)
}
