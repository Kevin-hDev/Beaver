pub fn supports_tools(model: &str) -> bool {
    model.starts_with("grok-4")
        || model.starts_with("grok-3")
        || model.starts_with("grok-2")
        || model.starts_with("grok-code")
        || model.starts_with("grok-build")
}

pub fn supports_thinking(model: &str) -> bool {
    !model.contains("non-reasoning")
        && (model.contains("reasoning")
            || model.contains("multi-agent")
            || model.starts_with("grok-4.5")
            || model.starts_with("grok-4.3")
            || model.starts_with("grok-3-mini")
            || model.starts_with("grok-build"))
}

pub fn supports_vision(model: &str) -> bool {
    model.contains("vision") || model.starts_with("grok-4") || model.starts_with("grok-build")
}

pub fn reasoning_modes(model: &str) -> &'static [&'static str] {
    let model = model.to_lowercase();
    if model.contains("non-reasoning") {
        &[]
    } else if is_grok_45(&model) {
        &["low", "medium", "high"]
    } else if is_grok_43(&model) {
        &["off", "low", "medium", "high"]
    } else if model.starts_with("grok-4.20")
        || model.starts_with("grok-build")
        || model.starts_with("grok-code")
    {
        &["auto"]
    } else {
        &[]
    }
}

pub fn reasoning_effort(model: &str, mode: Option<&str>) -> Option<&'static str> {
    let model = model.to_lowercase();
    match (is_grok_45(&model), is_grok_43(&model), mode) {
        (true, _, Some("low")) => Some("low"),
        (true, _, Some("medium")) => Some("medium"),
        (true, _, Some("high")) => Some("high"),
        (_, true, Some("off")) => Some("none"),
        (_, true, Some("low")) => Some("low"),
        (_, true, Some("medium")) => Some("medium"),
        (_, true, Some("high")) => Some("high"),
        _ => None,
    }
}

fn is_grok_45(model: &str) -> bool {
    matches!(model, "grok-4.5" | "grok-4.5-latest" | "grok-build-latest")
}

fn is_grok_43(model: &str) -> bool {
    matches!(model, "grok-4.3" | "grok-4.3-latest" | "grok-latest")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools() {
        assert!(supports_tools("grok-4"));
        assert!(supports_tools("grok-4-1-fast-reasoning"));
        assert!(supports_tools("grok-3-beta"));
        assert!(supports_tools("grok-2-1212"));
        assert!(supports_tools("grok-code-fast"));
        assert!(supports_tools("grok-build-0.1"));
        assert!(!supports_tools("grok-beta"));
    }

    #[test]
    fn thinking() {
        assert!(supports_thinking("grok-4-1-fast-reasoning"));
        assert!(supports_thinking("grok-4-fast-reasoning"));
        assert!(supports_thinking("grok-4.20-multi-agent-beta-0309"));
        assert!(supports_thinking("grok-3-mini"));
        assert!(supports_thinking("grok-3-mini-fast-beta"));
        assert!(supports_thinking("grok-4.5"));
        assert!(supports_thinking("grok-build-0.1"));
        assert!(!supports_thinking("grok-4.20-0309-non-reasoning"));
        assert!(!supports_thinking("grok-4"));
        assert!(!supports_thinking("grok-3-beta"));
    }

    #[test]
    fn vision() {
        assert!(supports_vision("grok-4"));
        assert!(supports_vision("grok-4-latest"));
        assert!(supports_vision("grok-2-vision-1212"));
        assert!(supports_vision("grok-build-0.1"));
        assert!(supports_vision("grok-vision-beta"));
        assert!(!supports_vision("grok-3-beta"));
        assert!(!supports_vision("grok-3-mini"));
    }

    #[test]
    fn aliases_keep_their_official_reasoning_modes() {
        assert_eq!(
            reasoning_modes("grok-4.5-latest"),
            &["low", "medium", "high"]
        );
        assert_eq!(
            reasoning_modes("grok-latest"),
            &["off", "low", "medium", "high"]
        );
        assert_eq!(reasoning_modes("grok-4.20"), &["auto"]);
        assert!(reasoning_modes("grok-4.20-non-reasoning").is_empty());
        assert_eq!(
            reasoning_effort("grok-4.3-latest", Some("off")),
            Some("none")
        );
    }
}
