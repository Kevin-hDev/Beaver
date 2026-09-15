use super::{contracts::VoiceDestination, errors::VoiceError, limits, types::VoiceLanguage};

pub fn validate_start(
    destination: &VoiceDestination,
    context_generation: u64,
    language: &Option<VoiceLanguage>,
    foreground: bool,
) -> Result<(), VoiceError> {
    if context_generation == 0 || context_generation > limits::MAX_CONTEXT_GENERATION || !foreground
    {
        return Err(VoiceError::invalid_settings());
    }
    match destination {
        VoiceDestination::Draft { draft_key } => {
            validate_id(draft_key, limits::MAX_DESTINATION_ID_CHARS)
        }
        VoiceDestination::Trial { trial_id } => {
            validate_id(trial_id, limits::MAX_DESTINATION_ID_CHARS)
        }
    }?;
    if let Some(VoiceLanguage::Language(code)) = language {
        let patch = super::types::VoiceSettingsPatch {
            language: Some(VoiceLanguage::Language(code.clone())),
            ..Default::default()
        };
        patch.validate()?;
    }
    Ok(())
}

pub fn validate_id(value: &str, max: usize) -> Result<(), VoiceError> {
    if value.is_empty() || value.chars().count() > max || value.chars().any(char::is_control) {
        Err(VoiceError::invalid_settings())
    } else {
        Ok(())
    }
}
