pub fn codex(model: &str, mode: Option<&str>) -> String {
    // Share the selector's validated default whenever model metadata exists.
    // Legacy callers still require a String for an entirely unknown model:
    // preserve medium here, not an admission guarantee or invented catalog entry.
    // Changing that compatibility contract requires auditing every caller first.
    super::reasoning::normalize_for_model("codex-oauth", model, mode, true)
        .unwrap_or_else(|| "medium".to_string())
}

pub fn openai(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("low") => Some("low"),
        Some("medium") | Some("auto") => Some("medium"),
        Some("high") => Some("high"),
        Some("xhigh") => Some("xhigh"),
        Some("max") => Some("max"),
        None => None,
        _ => None,
    }
}

pub fn simple(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("low") => Some("low"),
        Some("medium") | Some("auto") => Some("medium"),
        Some("high") | Some("xhigh") => Some("high"),
        None => None,
        _ => None,
    }
}

pub fn zai(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("low") => Some("low"),
        Some("medium") => Some("medium"),
        Some("high") => Some("high"),
        Some("xhigh") => Some("xhigh"),
        _ => None,
    }
}

pub fn openrouter(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("off") => Some("none"),
        Some("minimal") => Some("minimal"),
        Some("low") => Some("low"),
        Some("medium") => Some("medium"),
        Some("high") => Some("high"),
        Some("xhigh") => Some("xhigh"),
        Some("max") => Some("max"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "reasoning_effort_tests.rs"]
mod tests;
