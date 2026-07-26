pub fn supports_tools(model: &str) -> bool {
    model.starts_with("glm-5") || model.starts_with("glm-4")
}

pub fn supports_thinking(model: &str) -> bool {
    model.starts_with("glm-5")
        || model.starts_with("glm-4.7")
        || model.starts_with("glm-4.6")
        || model.starts_with("glm-4.5")
}

pub fn supports_vision(model: &str) -> bool {
    model.starts_with("glm") && model.contains("v")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools() {
        assert!(supports_tools("glm-5"));
        assert!(supports_tools("glm-5.2"));
        assert!(supports_tools("glm-5-code"));
        assert!(supports_tools("glm-4.7"));
        assert!(supports_tools("glm-4.5-flash"));
        assert!(supports_tools("glm-4-32b-0414-128k"));
        assert!(!supports_tools("chatglm-6b"));
    }

    #[test]
    fn thinking() {
        assert!(supports_thinking("glm-5"));
        assert!(supports_thinking("glm-5.2"));
        assert!(supports_thinking("glm-5-code"));
        assert!(supports_thinking("glm-4.7"));
        assert!(supports_thinking("glm-4.6"));
        assert!(supports_thinking("glm-4.5"));
        assert!(supports_thinking("glm-4.5-flash"));
        assert!(supports_thinking("glm-4.5v"));
        assert!(!supports_thinking("glm-4-32b-0414-128k"));
    }

    #[test]
    fn vision() {
        assert!(supports_vision("glm-4.5v"));
        assert!(!supports_vision("glm-5"));
        assert!(!supports_vision("glm-4.5-flash"));
        assert!(!supports_vision("glm-4.7"));
    }
}
