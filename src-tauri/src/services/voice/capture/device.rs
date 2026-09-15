use cpal::traits::{DeviceTrait, HostTrait};

use crate::services::voice::{contracts::VoiceDevice, errors::VoiceError, types::VoiceInputDevice};

pub fn list() -> Result<Vec<VoiceDevice>, VoiceError> {
    let devices = cpal::default_host()
        .input_devices()
        .map_err(|_| VoiceError::configuration_unavailable())?;
    devices
        .take(128)
        .map(|device| {
            let id = device
                .id()
                .map_err(|_| VoiceError::configuration_unavailable())?
                .to_string();
            let name = device.to_string();
            if id.is_empty()
                || id.chars().count() > crate::services::voice::limits::MAX_DEVICE_ID_CHARS
                || name.is_empty()
                || name.chars().count() > crate::services::voice::limits::MAX_DEVICE_NAME_CHARS
                || id.chars().chain(name.chars()).any(char::is_control)
            {
                return Err(VoiceError::configuration_unavailable());
            }
            Ok(VoiceDevice { id, name })
        })
        .collect()
}

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
