use serde_json::Value;

pub fn uses_auto_bypass(mode: &str) -> bool {
    matches!(mode, "auto" | "subagent")
}

pub fn requires_sensitive_bash_prompt(
    mode: &str,
    tool_name: &str,
    args: &Value,
) -> bool {
    if uses_auto_bypass(mode) {
        return false;
    }
    let input = match tool_name {
        "bash" => args["command"].as_str(),
        "bash_write" => args["chars"].as_str(),
        _ => None,
    };
    input
        .map(super::sensitive_data::bash_touches_sensitive_data)
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "permission_policy_tests.rs"]
mod tests;
