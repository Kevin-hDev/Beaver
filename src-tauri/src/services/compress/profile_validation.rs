use std::collections::HashSet;

use super::profile_limits::{
    MAX_BUDGET_TOKENS, MAX_CATEGORY_ITEMS, MAX_CUSTOM_PROMPT_CHARS, MAX_MODEL_FIELD_CHARS,
    MAX_PROFILES, MAX_PROFILE_NAME_CHARS, MAX_RETRIES,
};
use super::profile_types::{
    CompressionBandSettings, CompressionCategory, CompressionProfile, ItemBudget,
    SummaryFailurePolicy, SummaryModelSelection, TokenBudget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileValidationError {
    InvalidId,
    InvalidName,
    DuplicateName,
    TooManyProfiles,
    InvalidPrompt,
    InvalidModel,
    InvalidBudget,
    InvalidReductionOrder,
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
    validate_summary(profile)?;
    for band in [&profile.under_64k, &profile.compact, &profile.large] {
        validate_band(band)?;
    }
    let expected = HashSet::from([
        CompressionCategory::Images,
        CompressionCategory::Files,
        CompressionCategory::Tools,
        CompressionCategory::AssistantMessages,
        CompressionCategory::UserMessages,
    ]);
    if profile.reduction_order.len() != expected.len() {
        return Err(ProfileValidationError::InvalidReductionOrder);
    }
    let categories: HashSet<_> = profile.reduction_order.iter().copied().collect();
    if categories != expected {
        return Err(ProfileValidationError::InvalidReductionOrder);
    }
    Ok(())
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
    let folded_name = name.to_lowercase();
    if existing
        .iter()
        .any(|item| item.id != profile.id && item.name.trim().to_lowercase() == folded_name)
    {
        return Err(ProfileValidationError::DuplicateName);
    }
    if !(1..=90).contains(&profile.threshold_percent) {
        return Err(ProfileValidationError::InvalidBudget);
    }
    Ok(())
}

fn validate_summary(profile: &CompressionProfile) -> Result<(), ProfileValidationError> {
    for prompt in [
        &profile.summary.system_prompt,
        &profile.summary.handoff_prompt,
    ] {
        let invalid_control = prompt
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'));
        if prompt.chars().count() > MAX_CUSTOM_PROMPT_CHARS || invalid_control {
            return Err(ProfileValidationError::InvalidPrompt);
        }
    }
    validate_model(&profile.summary.model)?;
    if let Some(model) = &profile.summary.fallback_model {
        validate_model(model)?;
    }
    if profile.summary.ordinary_retries > MAX_RETRIES {
        return Err(ProfileValidationError::InvalidBudget);
    }
    if profile.summary.failure_policy == SummaryFailurePolicy::TryFallback
        && profile.summary.fallback_model.is_none()
    {
        return Err(ProfileValidationError::InvalidModel);
    }
    validate_token_budget(&profile.summary.input_budget)
}

fn validate_model(model: &SummaryModelSelection) -> Result<(), ProfileValidationError> {
    let SummaryModelSelection::Explicit { provider, model } = model else {
        return Ok(());
    };
    if [provider, model].iter().any(|value| {
        value.is_empty()
            || value.chars().count() > MAX_MODEL_FIELD_CHARS
            || value.chars().any(char::is_control)
    }) {
        return Err(ProfileValidationError::InvalidModel);
    }
    Ok(())
}

fn validate_band(band: &CompressionBandSettings) -> Result<(), ProfileValidationError> {
    if !(1..=100).contains(&band.target_percent) || band.summary_output.input_ratio_divisor == 0 {
        return Err(ProfileValidationError::InvalidBudget);
    }
    for budget in token_budgets(band) {
        validate_token_budget(budget)?;
    }
    let values = [
        band.summary_output.input_floor_tokens,
        band.summary_output.input_ceiling_tokens,
    ];
    if values.iter().any(|value| *value > MAX_BUDGET_TOKENS)
        || band.summary_output.input_floor_tokens > band.summary_output.input_ceiling_tokens
    {
        return Err(ProfileValidationError::InvalidBudget);
    }
    for budget in item_budgets(band) {
        validate_item_budget(budget)?;
    }
    if band.images.max_items > MAX_CATEGORY_ITEMS {
        return Err(ProfileValidationError::InvalidBudget);
    }
    Ok(())
}

fn token_budgets(band: &CompressionBandSettings) -> [&TokenBudget; 10] {
    [
        &band.response_reserve,
        &band.minimum_reduction,
        &band.summary_output.window_limit,
        &band.user_messages.tokens,
        &band.assistant_messages.tokens,
        &band.evidence_envelope,
        &band.git_tokens.tokens,
        &band.plan_and_tasks_tokens.tokens,
        &band.subagent_detail_tokens.tokens,
        &band.unresolved_state_tokens.tokens,
    ]
}

fn item_budgets(band: &CompressionBandSettings) -> [&ItemBudget; 5] {
    [
        &band.tools,
        &band.files,
        &band.modified_files,
        &band.text_attachments,
        &band.critical_references,
    ]
}

fn validate_token_budget(budget: &TokenBudget) -> Result<(), ProfileValidationError> {
    if budget.fixed_tokens > MAX_BUDGET_TOKENS
        || budget.minimum_tokens > MAX_BUDGET_TOKENS
        || budget.percent_basis_points > 10_000
        || (budget.fixed_tokens > 0 && budget.minimum_tokens > budget.fixed_tokens)
    {
        return Err(ProfileValidationError::InvalidBudget);
    }
    Ok(())
}

fn validate_item_budget(budget: &ItemBudget) -> Result<(), ProfileValidationError> {
    if budget.max_items > MAX_CATEGORY_ITEMS
        || budget.tokens_per_item > MAX_BUDGET_TOKENS
        || budget.total_tokens > MAX_BUDGET_TOKENS
    {
        return Err(ProfileValidationError::InvalidBudget);
    }
    Ok(())
}
