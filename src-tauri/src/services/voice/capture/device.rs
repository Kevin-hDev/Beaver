use cpal::traits::{DeviceTrait, HostTrait};

use crate::services::voice::{errors::VoiceError, types::VoiceInputDevice};

pub(super) fn resolve(selection: &VoiceInputDevice) -> Result<cpal::Device, VoiceError> {
    let host = cpal::default_host();
    match selection {
        VoiceInputDevice::SystemDefault => host
            .default_input_device()
            .ok_or_else(VoiceError::configuration_unavailable),
        VoiceInputDevice::Device(wanted) => {
            let devices = host
                .input_devices()
                .map_err(|_| VoiceError::configuration_unavailable())?;
            for device in devices.take(128) {
                if device.id().ok().is_some_and(|id| id.to_string() == *wanted) {
                    return Ok(device);
                }
            }
            Err(VoiceError::configuration_unavailable())
        }
    }
}
