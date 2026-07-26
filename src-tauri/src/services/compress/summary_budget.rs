pub const SMALL_CONTEXT_LIMIT: u64 = 64_000;
pub const LARGE_CONTEXT_LIMIT: u64 = 200_000;

const SMALL_SUMMARY_MAX: u64 = 8_000;
const MEDIUM_SUMMARY_MAX: u64 = 32_000;
const LARGE_SUMMARY_MAX: u64 = 50_000;
const SUMMARY_PERCENT: u64 = 15;
const EXTRA_LARGE_SUMMARY_PERCENT: u64 = 10;
const SUMMARY_OUTPUT_MIN: u64 = 1_000;
const SUMMARY_OUTPUT_MAX: u64 = 16_000;
const SUMMARY_INPUT_RATIO: u64 = 3;

pub fn summary_token_limit(context_window: u64) -> Option<u64> {
    if context_window == 0 {
        return None;
    }
    let pct = if context_window > 1_000_000 {
        EXTRA_LARGE_SUMMARY_PERCENT
    } else {
        SUMMARY_PERCENT
    };
    let target = context_window.saturating_mul(pct) / 100;
    let max = if context_window < SMALL_CONTEXT_LIMIT {
        SMALL_SUMMARY_MAX
    } else if context_window < LARGE_CONTEXT_LIMIT {
        MEDIUM_SUMMARY_MAX
    } else {
        LARGE_SUMMARY_MAX
    };
    Some(target.min(max).max(1_000))
}

pub fn summary_instruction_for_input(
    context_window: u64,
    input_tokens: usize,
) -> (Option<String>, u32) {
    let context_cap = summary_token_limit(context_window).unwrap_or(SUMMARY_OUTPUT_MAX);
    let input_cap =
        ((input_tokens as u64) / SUMMARY_INPUT_RATIO).clamp(SUMMARY_OUTPUT_MIN, SUMMARY_OUTPUT_MAX);
    let limit = context_cap.min(input_cap).max(1) as u32;
    (
        Some(format!(
            "Keep the summary compact and bounded. Target at most {limit} tokens. \
             Preserve concrete files, decisions, errors, user constraints, current state, \
             and next steps. Do not include full code blocks unless they are essential."
        )),
        limit,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_50k_cap_for_large_contexts() {
        assert_eq!(summary_token_limit(400_000), Some(50_000));
        assert_eq!(summary_token_limit(1_000_000), Some(50_000));
    }

    #[test]
    fn keeps_unknown_context_unbounded() {
        assert_eq!(summary_token_limit(0), None);
    }

    #[test]
    fn input_sized_summary_stays_small_for_small_sessions() {
        let (_, limit) = summary_instruction_for_input(258_000, 1_000);
        assert_eq!(limit, 1_000);
    }

    #[test]
    fn input_sized_summary_is_capped_for_large_sessions() {
        let (_, limit) = summary_instruction_for_input(258_000, 65_000);
        assert_eq!(limit, 16_000);
    }
}
