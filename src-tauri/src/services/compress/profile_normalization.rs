use std::collections::HashSet;

use super::profile_defaults::{beaver_profile, default_reduction_order, BEAVER_PROFILE_ID};
use super::profile_limits::{
    MAX_BUDGET_TOKENS, MAX_CATEGORY_ITEMS, MAX_CUSTOM_PROMPT_CHARS, MAX_PROFILES, MAX_RETRIES,
};
use super::profile_types::{CompressionBandSettings, CompressionProfile, ItemBudget, TokenBudget};

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
    profile.summary.ordinary_retries = profile.summary.ordinary_retries.min(MAX_RETRIES);
    profile.summary.system_prompt = truncate(&profile.summary.system_prompt);
    profile.summary.handoff_prompt = truncate(&profile.summary.handoff_prompt);
    normalize_token_budget(&mut profile.summary.input_budget);
    for band in [
        &mut profile.under_64k,
        &mut profile.compact,
        &mut profile.large,
    ] {
        normalize_band(band);
    }
    let mut categories = HashSet::new();
    profile
        .reduction_order
        .retain(|category| categories.insert(*category));
    if profile.reduction_order.is_empty() {
        profile.reduction_order = default_reduction_order();
    }
}

fn normalize_band(band: &mut CompressionBandSettings) {
    band.target_percent = band.target_percent.clamp(1, 100);
    for budget in [
        &mut band.response_reserve,
        &mut band.minimum_reduction,
        &mut band.summary_output.window_limit,
        &mut band.user_messages.tokens,
        &mut band.assistant_messages.tokens,
        &mut band.evidence_envelope,
    ] {
        normalize_token_budget(budget);
    }
    band.summary_output.input_ratio_divisor = band.summary_output.input_ratio_divisor.max(1);
    band.summary_output.input_floor_tokens = band
        .summary_output
        .input_floor_tokens
        .min(MAX_BUDGET_TOKENS);
    band.summary_output.input_ceiling_tokens = band
        .summary_output
        .input_ceiling_tokens
        .min(MAX_BUDGET_TOKENS);
    if band.summary_output.input_floor_tokens > band.summary_output.input_ceiling_tokens {
        band.summary_output.input_floor_tokens = band.summary_output.input_ceiling_tokens;
    }
    for budget in [
        &mut band.tools,
        &mut band.files,
        &mut band.text_attachments,
        &mut band.critical_references,
    ] {
        normalize_item_budget(budget);
    }
    band.images.max_items = band.images.max_items.min(MAX_CATEGORY_ITEMS);
    for value in [
        &mut band.git_tokens,
        &mut band.plan_and_tasks_tokens,
        &mut band.subagent_detail_tokens,
        &mut band.unresolved_state_tokens,
    ] {
        *value = (*value).min(MAX_BUDGET_TOKENS);
    }
}

fn normalize_token_budget(budget: &mut TokenBudget) {
    budget.fixed_tokens = budget.fixed_tokens.min(MAX_BUDGET_TOKENS);
    budget.minimum_tokens = budget.minimum_tokens.min(budget.fixed_tokens);
    budget.percent_basis_points = budget.percent_basis_points.min(10_000);
}

fn normalize_item_budget(budget: &mut ItemBudget) {
    budget.max_items = budget.max_items.min(MAX_CATEGORY_ITEMS);
    budget.tokens_per_item = budget.tokens_per_item.min(MAX_BUDGET_TOKENS);
    budget.total_tokens = budget.total_tokens.min(MAX_BUDGET_TOKENS);
}

fn truncate(value: &str) -> String {
    value.chars().take(MAX_CUSTOM_PROMPT_CHARS).collect()
}
