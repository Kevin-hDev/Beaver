use super::profile_limits::{
    MAX_CUSTOM_PROMPT_CHARS, MAX_FILES, MAX_IMAGES, MAX_MESSAGES, MAX_PROFILES,
    MAX_PROFILE_NAME_CHARS, MAX_SUMMARY_TOKENS, MAX_TOOL_RESULTS, MIN_SUMMARY_TOKENS,
};
use super::profile_types::{CompressionBandSettings, CompressionProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileValidationError {
    InvalidId,
    InvalidName,
    DuplicateName,
    TooManyProfiles,
    InvalidPrompt,
    InvalidBudget,
}

pub fn normalize_profile_document(
    profiles: &mut Vec<CompressionProfile>,
    global_profile_id: &mut String,
) {
    super::profile_normalization::normalize_profile_document(profiles, global_profile_id);
}

pub fn validate_profile_input(
    profile: &CompressionProfile,
    existing: &[CompressionProfile],
) -> Result<(), ProfileValidationError> {
    validate_identity(profile, existing)?;
    validate_prompt(&profile.system_prompt)?;
    validate_prompt(&profile.handoff_prompt)?;
    validate_band(&profile.under_64k)?;
    validate_band(&profile.compact)?;
    validate_band(&profile.large)
}

fn validate_identity(
    profile: &CompressionProfile,
    existing: &[CompressionProfile],
) -> Result<(), ProfileValidationError> {
    let updates_existing = existing.iter().any(|item| item.id == profile.id);
    if !updates_existing && existing.len() >= MAX_PROFILES {
        return Err(ProfileValidationError::TooManyProfiles);
    }
    if profile.id != "beaver" && uuid::Uuid::parse_str(&profile.id).is_err() {
        return Err(ProfileValidationError::InvalidId);
    }
    let name = profile.name.trim();
    if name.is_empty()
        || name.chars().count() > MAX_PROFILE_NAME_CHARS
        || name.chars().any(char::is_control)
    {
        return Err(ProfileValidationError::InvalidName);
    }
    if existing
        .iter()
        .any(|item| item.id != profile.id && item.name.trim().eq_ignore_ascii_case(name))
    {
        return Err(ProfileValidationError::DuplicateName);
    }
    if !(1..=90).contains(&profile.threshold_percent) {
        return Err(ProfileValidationError::InvalidBudget);
    }
    Ok(())
}

fn validate_prompt(prompt: &str) -> Result<(), ProfileValidationError> {
    let invalid_control = prompt
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'));
    if prompt.chars().count() > MAX_CUSTOM_PROMPT_CHARS || invalid_control {
        return Err(ProfileValidationError::InvalidPrompt);
    }
    Ok(())
}

fn validate_band(band: &CompressionBandSettings) -> Result<(), ProfileValidationError> {
    if band.recent_message_count > MAX_MESSAGES
        || !(MIN_SUMMARY_TOKENS..=MAX_SUMMARY_TOKENS).contains(&band.summary_max_tokens)
        || band.tool_result_count > MAX_TOOL_RESULTS
        || band.recent_file_count > MAX_FILES
        || band.image_count > MAX_IMAGES
    {
        return Err(ProfileValidationError::InvalidBudget);
    }
    Ok(())
}
