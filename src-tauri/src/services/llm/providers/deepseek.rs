pub fn supports_thinking(model: &str) -> bool {
    model == "deepseek-reasoner"
        || model.contains("deepseek-r1")
        || model.starts_with("deepseek-v4-pro")
        || model.starts_with("deepseek-v4-flash")
        || model.starts_with("deepseek-v3.2")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_deepseek_reasoning_models() {
        assert!(supports_thinking("deepseek-reasoner"));
        assert!(supports_thinking("deepseek-r1"));
        assert!(supports_thinking("deepseek-v4-pro"));
        assert!(supports_thinking("deepseek-v4-flash"));
        assert!(supports_thinking("deepseek-v3.2"));
        assert!(!supports_thinking("deepseek-chat"));
    }
}
