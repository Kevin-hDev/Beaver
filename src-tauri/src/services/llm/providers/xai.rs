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

pub fn context_length(model: &str) -> Option<u32> {
    match model {
        "grok-4.5" => Some(500_000),
        "grok-4.3" | "grok-4.20-0309-reasoning" | "grok-4.20-0309-non-reasoning" => Some(1_000_000),
        "grok-build-0.1" => Some(256_000),
        _ => None,
    }
}

pub fn reasoning_modes(model: &str) -> &'static [&'static str] {
    match model {
        "grok-4.5" => &["low", "medium", "high"],
        "grok-4.3" => &["off", "low", "medium", "high"],
        "grok-4.20-0309-reasoning" | "grok-build-0.1" => &["auto"],
        _ => &[],
    }
}

pub fn reasoning_effort(model: &str, mode: Option<&str>) -> Option<&'static str> {
    match (model, mode) {
        ("grok-4.5", Some("low")) => Some("low"),
        ("grok-4.5", Some("medium")) => Some("medium"),
        ("grok-4.5", Some("high")) => Some("high"),
        ("grok-4.3", Some("off")) => Some("none"),
        ("grok-4.3", Some("low")) => Some("low"),
        ("grok-4.3", Some("medium")) => Some("medium"),
        ("grok-4.3", Some("high")) => Some("high"),
        _ => None,
    }
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
}
