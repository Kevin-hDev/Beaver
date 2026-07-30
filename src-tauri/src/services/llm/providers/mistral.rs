pub fn is_adjustable_reasoning(model: &str) -> bool {
    matches!(
        model,
        "mistral-small-2603"
            | "mistral-small-latest"
            | "mistral-medium-3-5"
            | "mistral-medium-3"
            | "mistral-medium-latest"
    )
}

pub fn supports_thinking(model: &str) -> bool {
    is_adjustable_reasoning(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mistral_reasoning_models() {
        assert!(is_adjustable_reasoning("mistral-small-latest"));
        assert!(is_adjustable_reasoning("mistral-small-2603"));
        assert!(is_adjustable_reasoning("mistral-medium-3-5"));
        assert!(is_adjustable_reasoning("mistral-medium-3"));
        assert!(!is_adjustable_reasoning("mistral-small-2506"));
        assert!(!supports_thinking("codestral-latest"));
    }
}
