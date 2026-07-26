pub fn supports_tools(model: &str) -> bool {
    is_k3(model)
        || model.starts_with("kimi-k2")
        || model.starts_with("kimi-latest")
        || model.starts_with("kimi-for-coding")
}

pub fn supports_thinking(model: &str) -> bool {
    is_forced_thinking(model) || is_switchable_thinking(model)
}

pub fn is_forced_thinking(model: &str) -> bool {
    is_k3(model)
        || model.starts_with("kimi-k2.7-code")
        || model.contains("k2-thinking")
        || model.contains("thinking-preview")
        || model.starts_with("kimi-for-coding")
}

pub fn is_switchable_thinking(model: &str) -> bool {
    model.starts_with("kimi-k2.5") || model.starts_with("kimi-k2.6")
}

pub fn supports_vision(model: &str) -> bool {
    is_k3(model)
        || model.contains("k2.5")
        || model.contains("k2.6")
        || model.contains("k2.7")
        || model.starts_with("kimi-latest")
        || model.contains("thinking-preview")
        || model.contains("vision")
        || model.starts_with("kimi-for-coding")
}

pub fn is_k3(model: &str) -> bool {
    model == "k3" || model.starts_with("kimi-k3")
}

pub fn default_reasoning_mode(model: &str) -> Option<&'static str> {
    if is_k3(model) {
        Some("max")
    } else if is_forced_thinking(model) {
        Some("auto")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools() {
        assert!(supports_tools("k3"));
        assert!(supports_tools("kimi-k2.6"));
        assert!(supports_tools("kimi-k2.7-code"));
        assert!(supports_tools("kimi-k2.7-code-highspeed"));
        assert!(supports_tools("kimi-k2.5"));
        assert!(supports_tools("kimi-k2-thinking"));
        assert!(supports_tools("kimi-latest"));
        assert!(supports_tools("kimi-latest-128k"));
        assert!(supports_tools("kimi-for-coding"));
        assert!(supports_tools("kimi-for-coding-highspeed"));
        assert!(!supports_tools("moonshot-v1-128k"));
    }

    #[test]
    fn thinking() {
        assert!(supports_thinking("k3"));
        assert!(supports_thinking("kimi-k2-thinking"));
        assert!(supports_thinking("kimi-thinking-preview"));
        assert!(supports_thinking("kimi-k2.5"));
        assert!(supports_thinking("kimi-k2.6"));
        assert!(supports_thinking("kimi-k2.7-code"));
        assert!(supports_thinking("kimi-k2.7-code-highspeed"));
        assert!(supports_thinking("kimi-for-coding"));
        assert!(!supports_thinking("kimi-latest"));
    }

    #[test]
    fn vision() {
        assert!(supports_vision("k3"));
        assert!(supports_vision("kimi-k2.5"));
        assert!(supports_vision("kimi-k2.6"));
        assert!(supports_vision("kimi-k2.7-code"));
        assert!(supports_vision("kimi-k2.7-code-highspeed"));
        assert!(supports_vision("kimi-latest"));
        assert!(supports_vision("kimi-thinking-preview"));
        assert!(supports_vision("moonshot-v1-128k-vision-preview"));
        assert!(supports_vision("kimi-for-coding"));
        assert!(!supports_vision("kimi-k2-thinking"));
    }

    #[test]
    fn official_reasoning_defaults() {
        assert_eq!(default_reasoning_mode("k3"), Some("max"));
        assert_eq!(default_reasoning_mode("kimi-for-coding"), Some("auto"));
        assert_eq!(default_reasoning_mode("kimi-k2.5"), None);
    }
}
