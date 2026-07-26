pub const GPT_56_CONTEXT_LENGTH: u32 = 1_050_000;

pub fn is_gpt_56(model: &str) -> bool {
    let model = model.rsplit_once('/').map(|(_, id)| id).unwrap_or(model);
    matches!(
        model.to_lowercase().as_str(),
        "gpt-5.6"
            | "gpt-5.6-sol"
            | "gpt-5.6-terra"
            | "gpt-5.6-luna"
            | "gpt-5.6-sol-pro"
            | "gpt-5.6-terra-pro"
            | "gpt-5.6-luna-pro"
    )
}

pub fn context_length(model: &str) -> Option<u32> {
    is_gpt_56(model).then_some(GPT_56_CONTEXT_LENGTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_direct_and_gateway_model_ids() {
        assert!(is_gpt_56("gpt-5.6"));
        assert!(is_gpt_56("gpt-5.6-sol"));
        assert!(is_gpt_56("openai/gpt-5.6-terra"));
        assert!(is_gpt_56("openai/gpt-5.6-terra-pro"));
        assert!(!is_gpt_56("gpt-5.5"));
    }
}
