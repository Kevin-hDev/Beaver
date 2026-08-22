pub const PROVIDER_ID: &str = "openai";

pub fn is_gpt_56(model: &str) -> bool {
    let model = model.rsplit_once('/').map(|(_, id)| id).unwrap_or(model);
    matches!(
        model.to_lowercase().as_str(),
        "gpt-5.6" | "gpt-5.6-sol" | "gpt-5.6-terra" | "gpt-5.6-luna"
    )
}

pub fn uses_max_completion_tokens(model: &str) -> bool {
    let model = model.rsplit_once('/').map(|(_, id)| id).unwrap_or(model);
    let model = model.to_lowercase();
    model.starts_with("gpt-5")
        || ["o1", "o3", "o4"].iter().any(|prefix| {
            model
                .strip_prefix(*prefix)
                .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('-'))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_direct_and_gateway_model_ids() {
        assert!(is_gpt_56("gpt-5.6"));
        assert!(is_gpt_56("gpt-5.6-sol"));
        assert!(is_gpt_56("openai/gpt-5.6-terra"));
        assert!(!is_gpt_56("openai/gpt-5.6-terra-pro"));
        assert!(!is_gpt_56("gpt-5.5"));
    }

    #[test]
    fn completion_token_field_covers_openai_reasoning_families() {
        for model in ["gpt-5", "gpt-5.6-sol", "o1", "o3-mini", "openai/o4-mini"] {
            assert!(uses_max_completion_tokens(model));
        }
        assert!(!uses_max_completion_tokens("gpt-4o"));
    }
}
