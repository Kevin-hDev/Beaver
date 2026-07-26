pub fn is_adjustable_reasoning(model: &str) -> bool {
    model == "mistral-small-latest"
        || model == "mistral-medium-3-5"
        || model == "mistral-medium-3.5"
}

pub fn is_native_reasoning(model: &str) -> bool {
    model.starts_with("magistral-small") || model.starts_with("magistral-medium")
}

pub fn supports_thinking(model: &str) -> bool {
    is_adjustable_reasoning(model) || is_native_reasoning(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mistral_reasoning_models() {
        assert!(is_adjustable_reasoning("mistral-small-latest"));
        assert!(is_adjustable_reasoning("mistral-medium-3-5"));
        assert!(!is_adjustable_reasoning("mistral-small-2506"));
        assert!(is_native_reasoning("magistral-small-latest"));
        assert!(is_native_reasoning("magistral-medium-latest"));
        assert!(!supports_thinking("codestral-latest"));
    }
}
