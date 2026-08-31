use std::collections::HashSet;

use super::profile_defaults::{beaver_profile, BEAVER_PROFILE_ID};
use super::profile_limits::{
    MAX_CUSTOM_PROMPT_CHARS, MAX_FILES, MAX_IMAGES, MAX_MESSAGES, MAX_PROFILES, MAX_SUMMARY_TOKENS,
    MAX_TOOL_RESULTS, MIN_SUMMARY_TOKENS,
};
use super::profile_types::{CompressionBandSettings, CompressionProfile};

pub fn normalize_profile_document(
    profiles: &mut Vec<CompressionProfile>,
    global_profile_id: &mut String,
) {
    put_beaver_first(profiles);
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    profiles.retain(|profile| {
        (profile.id == BEAVER_PROFILE_ID || uuid::Uuid::parse_str(&profile.id).is_ok())
            && valid_name(&profile.name)
            && ids.insert(profile.id.clone())
            && names.insert(profile.name.trim().to_lowercase())
    });
    profiles.truncate(MAX_PROFILES);
    profiles.iter_mut().for_each(normalize_profile);
    if !profiles
        .iter()
        .any(|profile| profile.id == *global_profile_id)
    {
        *global_profile_id = BEAVER_PROFILE_ID.to_string();
    }
}

fn valid_name(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty()
        && name.chars().count() <= super::profile_limits::MAX_PROFILE_NAME_CHARS
        && !name.chars().any(char::is_control)
}

fn put_beaver_first(profiles: &mut Vec<CompressionProfile>) {
    let mut beaver = profiles
        .iter()
        .position(|profile| profile.id == BEAVER_PROFILE_ID)
        .map(|index| profiles.remove(index))
        .unwrap_or_else(beaver_profile);
    beaver.name = "Beaver".to_string();
    profiles.insert(0, beaver);
}

fn normalize_profile(profile: &mut CompressionProfile) {
    profile.threshold_percent = profile.threshold_percent.clamp(1, 90);
    profile.system_prompt = truncate(&profile.system_prompt);
    profile.handoff_prompt = truncate(&profile.handoff_prompt);
    normalize_band(&mut profile.under_64k);
    normalize_band(&mut profile.compact);
    normalize_band(&mut profile.large);
}

fn normalize_band(band: &mut CompressionBandSettings) {
    band.recent_message_count = band.recent_message_count.min(MAX_MESSAGES);
    band.summary_max_tokens = band
        .summary_max_tokens
        .clamp(MIN_SUMMARY_TOKENS, MAX_SUMMARY_TOKENS);
    band.tool_result_count = band.tool_result_count.min(MAX_TOOL_RESULTS);
    band.recent_file_count = band.recent_file_count.min(MAX_FILES);
    band.image_count = band.image_count.min(MAX_IMAGES);
}

fn truncate(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
        .take(MAX_CUSTOM_PROMPT_CHARS)
        .collect()
}
