pub fn supported_modes(model: &str) -> &'static [&'static str] {
    if is_forced_reasoning(model) {
        &["low", "medium", "high"]
    } else {
        &["off", "low", "medium", "high"]
    }
}

pub fn is_forced_reasoning(model: &str) -> bool {
    let model = model.to_lowercase();
    model.starts_with("gemini-3") || model.starts_with("gemini-2.5-pro")
}
