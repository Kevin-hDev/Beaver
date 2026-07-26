pub fn is_gpt_oss_effort(model: &str) -> bool {
    model.contains("gpt-oss-20b") || model.contains("gpt-oss-120b")
}

pub fn is_qwen_switchable(model: &str) -> bool {
    model == "qwen3-32b" || model.contains("qwen3-32b")
}

pub fn is_safeguard(model: &str) -> bool {
    model.contains("gpt-oss-safeguard-20b")
}

pub fn supports_thinking(model: &str) -> bool {
    is_gpt_oss_effort(model) || is_qwen_switchable(model) || is_safeguard(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_groq_reasoning_models() {
        assert!(is_gpt_oss_effort("openai/gpt-oss-20b"));
        assert!(is_gpt_oss_effort("openai/gpt-oss-120b"));
        assert!(is_qwen_switchable("qwen3-32b"));
        assert!(is_qwen_switchable("qwen/qwen3-32b"));
        assert!(is_safeguard("openai/gpt-oss-safeguard-20b"));
        assert!(!supports_thinking("llama-3.3-70b-versatile"));
    }
}
