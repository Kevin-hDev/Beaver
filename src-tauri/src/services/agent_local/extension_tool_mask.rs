use serde_json::Value;

const MASK_PERCENT: usize = 10;
const UNKNOWN_CONTEXT_TOKEN_LIMIT: usize = 20_000;

pub fn should_mask(plugin_definitions: &[Value], context_window: u64) -> bool {
    let tokens = plugin_definitions
        .iter()
        .map(|definition| {
            crate::services::token_counting::estimate_text_tokens(&definition.to_string())
        })
        .sum::<usize>();
    if context_window == 0 {
        return tokens > UNKNOWN_CONTEXT_TOKEN_LIMIT;
    }
    let context = usize::try_from(context_window).unwrap_or(usize::MAX);
    tokens.saturating_mul(100) > context.saturating_mul(MASK_PERCENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn known_context_uses_ten_percent_of_the_total_window() {
        let definitions = vec![json!({"description": "x".repeat(4_500)})];

        assert!(should_mask(&definitions, 10_000));
        assert!(!should_mask(&definitions, 100_000));
    }

    #[test]
    fn ten_percent_boundary_masks_only_when_strictly_exceeded() {
        let definitions = vec![json!({"description": "fixed boundary payload"})];
        let tokens = crate::services::token_counting::estimate_text_tokens(
            &definitions[0].to_string(),
        ) as u64;

        assert!(!should_mask(&definitions, tokens.saturating_mul(10)));
        assert!(should_mask(
            &definitions,
            tokens.saturating_mul(10).saturating_sub(1)
        ));
    }

    #[test]
    fn unknown_context_uses_the_absolute_fallback() {
        let small = vec![json!({"description": "x".repeat(1_000)})];
        let large = vec![json!({"description": "x".repeat(90_000)})];

        assert!(!should_mask(&small, 0));
        assert!(should_mask(&large, 0));
    }
}
