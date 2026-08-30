use super::profile_types::{BudgetMode, CompressionWindowBand, SummaryOutputBudget, TokenBudget};

pub fn band_for_window(context_window: u64) -> Option<CompressionWindowBand> {
    match context_window {
        0 => None,
        1..64_000 => Some(CompressionWindowBand::Under64K),
        64_000..128_000 => Some(CompressionWindowBand::Compact),
        _ => Some(CompressionWindowBand::Large),
    }
}

// Task 5 applies this profile calculation to the context selector.
#[allow(dead_code)]
pub fn resolve_budget(spec: &TokenBudget, context_window: u64) -> u32 {
    let percentage = ((context_window as u128).saturating_mul(spec.percent_basis_points as u128)
        / 10_000)
        .min(u32::MAX as u128) as u32;

    match spec.mode {
        BudgetMode::Fixed => spec.fixed_tokens,
        BudgetMode::Percentage => {
            let capped = if spec.fixed_tokens == 0 {
                percentage
            } else {
                percentage.min(spec.fixed_tokens)
            };
            capped.max(spec.minimum_tokens)
        }
        BudgetMode::Minimum => spec.fixed_tokens.min(percentage),
    }
}

// Task 7 moves summary output sizing onto profiles.
#[allow(dead_code)]
pub fn summary_output_limit(
    budget: &SummaryOutputBudget,
    context_window: u64,
    input_tokens: u64,
) -> u32 {
    let window_limit = resolve_budget(&budget.window_limit, context_window);
    let input_limit = input_tokens
        .checked_div(u64::from(budget.input_ratio_divisor.max(1)))
        .unwrap_or_default()
        .clamp(
            u64::from(budget.input_floor_tokens),
            u64::from(budget.input_ceiling_tokens),
        )
        .min(u64::from(u32::MAX)) as u32;
    window_limit.min(input_limit).max(1)
}
