pub const CLAUDIATOR: &str = "Claudiator";
pub const GEMINITOR: &str = "Geminitor";
pub const CLAUDIATOR_COLOR: &str = "claudiator";
pub const GEMINITOR_COLOR: &str = "geminitor";

const MAX_DESCRIPTION_CHARS: usize = 160;

pub fn default_name(subagent_type: &str) -> &'static str {
    match subagent_type {
        "coder" => CLAUDIATOR,
        _ => GEMINITOR,
    }
}

pub fn default_color_key(subagent_type: &str) -> &'static str {
    match subagent_type {
        "coder" => CLAUDIATOR_COLOR,
        _ => GEMINITOR_COLOR,
    }
}

pub fn clean_name(_input: Option<&str>, subagent_type: &str) -> String {
    default_name(subagent_type).to_string()
}

pub fn clean_description(input: Option<&str>, prompt: &str) -> String {
    let fallback = prompt
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Mission sous-agent");
    bounded_non_empty(input, fallback, MAX_DESCRIPTION_CHARS)
}

pub fn legacy_mission_label(input: Option<&str>, subagent_type: &str) -> Option<String> {
    let value = input?.trim();
    if value.is_empty() || is_default_or_legacy_name(value, subagent_type) {
        None
    } else {
        Some(bounded_non_empty(Some(value), value, MAX_DESCRIPTION_CHARS))
    }
}

fn is_default_or_legacy_name(value: &str, subagent_type: &str) -> bool {
    let normalized = value.to_lowercase();
    matches!(
        normalized.as_str(),
        "agent" | "explore" | "explorer" | "coder"
    ) || normalized == default_name(subagent_type).to_lowercase()
}

fn bounded_non_empty(input: Option<&str>, fallback: &str, max_chars: usize) -> String {
    let raw = input
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_subagent_type() {
        assert_eq!(default_name("coder"), "Claudiator");
        assert_eq!(default_color_key("coder"), "claudiator");
        assert_eq!(default_name("explorer"), "Geminitor");
        assert_eq!(default_color_key("explorer"), "geminitor");
    }

    #[test]
    fn clean_name_keeps_product_identity() {
        assert_eq!(
            clean_name(Some("Audit subagents long"), "explorer"),
            "Geminitor"
        );
        assert_eq!(clean_name(Some("Implementation"), "coder"), "Claudiator");
    }

    #[test]
    fn legacy_custom_name_can_become_mission_description() {
        assert_eq!(
            legacy_mission_label(Some("Audit subagents long"), "explorer"),
            Some("Audit subagents long".to_string())
        );
        assert_eq!(legacy_mission_label(Some("Geminitor"), "explorer"), None);
        assert_eq!(legacy_mission_label(Some("agent"), "coder"), None);
    }

    #[test]
    fn legacy_custom_name_is_bounded() {
        let input = "x".repeat(MAX_DESCRIPTION_CHARS + 20);
        let label = legacy_mission_label(Some(&input), "explorer").unwrap();

        assert_eq!(label.chars().count(), MAX_DESCRIPTION_CHARS);
    }
}
