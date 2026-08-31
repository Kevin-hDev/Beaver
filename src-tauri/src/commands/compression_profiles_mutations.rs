use crate::models::compression_profile_contract::CompressionProfileInput;
use crate::services::compress::profile_defaults::{beaver_profile, BEAVER_PROFILE_ID};
use crate::services::compress::profile_store::CompressionProfileStoreError;
use crate::services::compress::profile_store_document::CompressionProfileDocument;
use crate::services::compress::profile_types::CompressionProfile;
use crate::services::compress::profile_validation::validate_profile_input;

pub(super) fn create(
    document: &mut CompressionProfileDocument,
    source_profile_id: &str,
    name: String,
) -> Result<(), CompressionProfileStoreError> {
    let source = find_profile(document, source_profile_id)?.clone();
    let mut profile = source;
    profile.id = uuid::Uuid::new_v4().to_string();
    profile.name = name.trim().to_string();
    profile.revision = 1;
    validate_profile_input(&profile, &document.profiles)
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    document.global_profile_id = profile.id.clone();
    document.global_selection_revision = next_revision(document.global_selection_revision)?;
    document.profiles.push(profile);
    Ok(())
}

pub(super) fn rename(
    document: &mut CompressionProfileDocument,
    profile_id: &str,
    name: String,
) -> Result<(), CompressionProfileStoreError> {
    if profile_id == BEAVER_PROFILE_ID {
        return Err(CompressionProfileStoreError::Invalid);
    }
    let index = profile_index(document, profile_id)?;
    let mut candidate = document.profiles[index].clone();
    candidate.name = name.trim().to_string();
    candidate.revision = next_revision(candidate.revision)?;
    validate_profile_input(&candidate, &document.profiles)
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    document.profiles[index] = candidate;
    Ok(())
}

pub(super) fn save(
    document: &mut CompressionProfileDocument,
    input: CompressionProfileInput,
) -> Result<(), CompressionProfileStoreError> {
    let mut candidate: CompressionProfile = input.into();
    candidate.name = candidate.name.trim().to_string();
    let index = profile_index(document, &candidate.id)?;
    let current = &document.profiles[index];
    if candidate.revision != current.revision
        || (candidate.id == BEAVER_PROFILE_ID && candidate.name != "Beaver")
    {
        return Err(CompressionProfileStoreError::Invalid);
    }
    candidate.revision = next_revision(current.revision)?;
    validate_profile_input(&candidate, &document.profiles)
        .map_err(|_| CompressionProfileStoreError::Invalid)?;
    document.profiles[index] = candidate;
    Ok(())
}

pub(super) fn select_global(
    document: &mut CompressionProfileDocument,
    profile_id: &str,
) -> Result<(), CompressionProfileStoreError> {
    find_profile(document, profile_id)?;
    if document.global_profile_id != profile_id {
        document.global_profile_id = profile_id.to_string();
        document.global_selection_revision = next_revision(document.global_selection_revision)?;
    }
    Ok(())
}

pub(super) fn set_automatic_enabled(
    document: &mut CompressionProfileDocument,
    enabled: bool,
) -> Result<(), CompressionProfileStoreError> {
    document.automatic_enabled = enabled;
    Ok(())
}

pub(super) fn reset_beaver(
    document: &mut CompressionProfileDocument,
) -> Result<(), CompressionProfileStoreError> {
    let index = profile_index(document, BEAVER_PROFILE_ID)?;
    let mut reset = beaver_profile();
    reset.revision = next_revision(document.profiles[index].revision)?;
    document.profiles[index] = reset;
    Ok(())
}

pub(super) fn reset_prompts(
    document: &mut CompressionProfileDocument,
    profile_id: &str,
) -> Result<(), CompressionProfileStoreError> {
    let index = profile_index(document, profile_id)?;
    let defaults = beaver_profile();
    let profile = &mut document.profiles[index];
    profile.system_prompt = defaults.system_prompt;
    profile.handoff_prompt = defaults.handoff_prompt;
    profile.revision = next_revision(profile.revision)?;
    Ok(())
}

pub(super) fn delete(
    document: &mut CompressionProfileDocument,
    profile_id: &str,
) -> Result<(), CompressionProfileStoreError> {
    if profile_id == BEAVER_PROFILE_ID {
        return Err(CompressionProfileStoreError::Invalid);
    }
    let index = profile_index(document, profile_id)?;
    document.profiles.remove(index);
    if document.global_profile_id == profile_id {
        document.global_profile_id = BEAVER_PROFILE_ID.to_string();
        document.global_selection_revision = next_revision(document.global_selection_revision)?;
    }
    Ok(())
}

fn find_profile<'a>(
    document: &'a CompressionProfileDocument,
    profile_id: &str,
) -> Result<&'a CompressionProfile, CompressionProfileStoreError> {
    document
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or(CompressionProfileStoreError::Invalid)
}

fn profile_index(
    document: &CompressionProfileDocument,
    profile_id: &str,
) -> Result<usize, CompressionProfileStoreError> {
    document
        .profiles
        .iter()
        .position(|profile| profile.id == profile_id)
        .ok_or(CompressionProfileStoreError::Invalid)
}

fn next_revision(revision: u64) -> Result<u64, CompressionProfileStoreError> {
    revision
        .checked_add(1)
        .ok_or(CompressionProfileStoreError::Invalid)
}
