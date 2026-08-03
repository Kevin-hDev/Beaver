use super::types_tools::ToolResult;
use crate::models::WakeupSchedule;
use serde_json::Value;
use std::collections::BTreeSet;

const MAX_SKILLS: usize = 8;
const MAX_TOOLS: usize = 12;
const DENIED_TOOLS: &[&str] = &[
    "ask_user_choice", "plan_mode", "manage_automation", "delegate_task",
    "list_subagents", "get_subagent", "cancel_subagent", "message_subagent",
    "archive_subagent", "inspect_subagent_changes", "apply_subagent_changes",
    "discard_subagent_changes",
];

pub fn parse_schedule(value: Option<&Value>) -> Result<WakeupSchedule, ToolResult> {
    value.cloned()
        .ok_or_else(|| ToolResult::validation("automation_schedule_required", "Déclencheur requis."))
        .and_then(|value| serde_json::from_value(value).map_err(|_| {
            ToolResult::validation("automation_schedule_invalid", "Déclencheur invalide.")
        }))
}

pub async fn validate_skills(value: Option<&Value>) -> Result<Vec<String>, ToolResult> {
    let ids = string_list(value, MAX_SKILLS, "automation_skills_invalid")?;
    let available = super::tool_skill_loader::list_skills().await
        .map_err(|_| internal_error())?.into_iter().map(|skill| skill.id)
        .collect::<BTreeSet<_>>();
    if ids.iter().any(|id| !available.contains(id)) {
        return Err(ToolResult::validation(
            "automation_skill_unknown", "Skill d'automatisation introuvable.",
        ));
    }
    Ok(ids)
}

pub fn validate_tools(value: Option<&Value>) -> Result<Vec<String>, ToolResult> {
    let names = string_list(value, MAX_TOOLS, "automation_tools_invalid")?;
    let available = super::tool_catalog::catalog().into_iter()
        .map(|entry| entry.id).collect::<BTreeSet<_>>();
    if names.iter().any(|name| {
        !available.contains(name.as_str()) || DENIED_TOOLS.contains(&name.as_str())
    }) {
        return Err(ToolResult::validation(
            "automation_tool_unknown", "Outil d'automatisation non autorisé.",
        ));
    }
    Ok(names)
}

fn string_list(
    value: Option<&Value>,
    max: usize,
    code: &'static str,
) -> Result<Vec<String>, ToolResult> {
    let Some(values) = value.and_then(Value::as_array) else { return Ok(Vec::new()); };
    let mut result = BTreeSet::new();
    for value in values {
        let Some(value) = value.as_str().filter(|value| !value.is_empty() && value.len() <= 768) else {
            return Err(ToolResult::validation(code, "Liste d'automatisation invalide."));
        };
        result.insert(value.to_string());
        if result.len() > max {
            return Err(ToolResult::validation(code, "Liste d'automatisation trop longue."));
        }
    }
    Ok(result.into_iter().collect())
}

pub fn text(args: &Value, key: &str) -> String {
    args[key].as_str().unwrap_or_default().trim().to_string()
}

pub fn internal_error() -> ToolResult {
    ToolResult::internal("automation_unavailable", "Automatisation indisponible.", true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_bounded_tools_and_rejects_nested_agents() {
        assert_eq!(
            validate_tools(Some(&json!(["read_file", "grep"]))).unwrap(),
            ["grep", "read_file"]
        );
        assert!(validate_tools(Some(&json!(["delegate_task"]))).is_err());
    }

    #[test]
    fn parses_supported_schedule_and_rejects_unknown_shape() {
        assert!(matches!(
            parse_schedule(Some(&json!({"kind":"daily", "time":"08:00"}))).unwrap(),
            WakeupSchedule::Daily { .. }
        ));
        assert!(parse_schedule(Some(&json!({"kind":"cron", "value":"* * * * *"}))).is_err());
    }
}
