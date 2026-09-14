use super::errors::VoiceError;
use super::types::{VoiceSettings, VoiceSettingsPatch};

pub fn read_voice_settings() -> Result<VoiceSettings, VoiceError> {
    crate::services::config::read_config()
        .map(|config| config.voice)
        .map_err(|_| VoiceError::configuration_unavailable())
}

pub fn update_voice_settings(patch: VoiceSettingsPatch) -> Result<VoiceSettings, VoiceError> {
    patch.validate()?;
    crate::services::config::update_config(move |config| {
        patch.apply(&mut config.voice);
        Ok(config.voice.clone())
    })
    .map_err(|_| VoiceError::configuration_unavailable())
}

#[cfg(test)]
pub(super) fn read_voice_settings_at_path(
    path: &std::path::Path,
    data_dir: &std::path::Path,
) -> Result<VoiceSettings, VoiceError> {
    crate::services::config::read_config_from_path(path, data_dir)
        .map(|config| config.voice)
        .map_err(|_| VoiceError::configuration_unavailable())
}

#[cfg(test)]
pub(super) fn update_voice_settings_at_path(
    path: &std::path::Path,
    data_dir: &std::path::Path,
    patch: VoiceSettingsPatch,
) -> Result<VoiceSettings, VoiceError> {
    patch.validate()?;
    crate::services::config::update_config_at_path(path, data_dir, move |config| {
        patch.apply(&mut config.voice);
        Ok(config.voice.clone())
    })
    .map_err(|_| VoiceError::configuration_unavailable())
}
