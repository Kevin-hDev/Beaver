pub fn reasoning_effort(model: &str, mode: Option<&str>) -> Option<&'static str> {
    let model = model.to_lowercase();
    match (
        is_grok_46(&model),
        is_grok_45(&model),
        is_grok_43(&model),
        mode,
    ) {
        (true, _, _, Some("low")) => Some("low"),
        (true, _, _, Some("medium")) => Some("medium"),
        (true, _, _, Some("high")) => Some("high"),
        (true, _, _, Some("xhigh")) => Some("xhigh"),
        (_, true, _, Some("low")) => Some("low"),
        (_, true, _, Some("medium")) => Some("medium"),
        (_, true, _, Some("high")) => Some("high"),
        (_, _, true, Some("off")) => Some("none"),
        (_, _, true, Some("low")) => Some("low"),
        (_, _, true, Some("medium")) => Some("medium"),
        (_, _, true, Some("high")) => Some("high"),
        _ => None,
    }
}

fn is_grok_46(model: &str) -> bool {
    matches!(model, "grok-4.6" | "grok-4.6-latest")
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
    fn protocol_effort_mapping_stays_exact() {
        assert_eq!(
            reasoning_effort("grok-4.3-latest", Some("off")),
            Some("none")
        );
    }
}
