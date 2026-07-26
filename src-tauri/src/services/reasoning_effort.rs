pub fn codex(model: &str, mode: Option<&str>) -> String {
    let supported = super::reasoning::supported_modes("codex-oauth", model, true);
    let default = if model == "gpt-5.3-codex-spark" {
        "high"
    } else {
        "medium"
    };
    mode.filter(|value| supported.contains(value))
        .unwrap_or(default)
        .to_string()
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
        Some("low") => Some("low"),
        Some("medium") => Some("medium"),
        Some("high") => Some("high"),
        Some("xhigh") => Some("xhigh"),
        Some("max") => Some("max"),
        _ => None,
    }
}
