pub(super) fn valid_provider_id(provider: &str) -> bool {
    !provider.is_empty()
        && provider.len() <= 32
        && provider
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || matches!(byte, b'-' | b'_'))
}

pub(super) fn valid_date(date: &str) -> bool {
    date.len() == 10 && chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

pub(super) fn valid_source_url(url: &str) -> bool {
    url.len() <= 512 && url.starts_with("https://") && !url.chars().any(char::is_control)
}

pub(super) fn valid_reasoning_contract(
    supports_thinking: bool,
    modes: &[String],
    default_mode: Option<&str>,
) -> Result<(), &'static str> {
    const ALLOWED_MODES: [&str; 8] = [
        "off", "auto", "low", "medium", "high", "xhigh", "max", "ultra",
    ];

    if !supports_thinking && (!modes.is_empty() || default_mode.is_some()) {
        return Err("reasoning_modes");
    }
    // Borne implicite : huit valeurs autorisées et uniques, avec entrées bornées en amont.
    let mut seen = std::collections::HashSet::with_capacity(modes.len());
    if modes
        .iter()
        .any(|mode| !ALLOWED_MODES.contains(&mode.as_str()) || !seen.insert(mode.as_str()))
    {
        return Err("reasoning_modes");
    }
    if default_mode.is_some_and(|mode| !seen.contains(mode)) {
        return Err("reasoning_default");
    }
    Ok(())
}

pub(super) fn valid_reasoning_transport(
    is_model_studio: bool,
    supports_thinking: bool,
    modes: &[String],
    supports_toggle: bool,
    supports_replay: bool,
    supports_tools: bool,
    requires_tool_stream: bool,
) -> Result<(), &'static str> {
    if (supports_toggle || supports_replay) && (!is_model_studio || !supports_thinking) {
        return Err("reasoning_transport");
    }
    if is_model_studio && modes.iter().any(|mode| mode == "off") && !supports_toggle {
        return Err("reasoning_transport");
    }
    if requires_tool_stream && (!is_model_studio || !supports_tools) {
        return Err("tool_stream_transport");
    }
    Ok(())
}
