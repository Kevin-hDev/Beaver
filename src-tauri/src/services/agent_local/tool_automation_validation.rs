use super::types_tools::ToolResult;
use crate::models::WakeupSchedule;
use serde_json::Value;

pub fn parse_schedule(value: Option<&Value>) -> Result<WakeupSchedule, ToolResult> {
    value.cloned()
        .ok_or_else(|| ToolResult::validation("automation_schedule_required", "Déclencheur requis."))
        .and_then(|value| serde_json::from_value(value).map_err(|_| {
            ToolResult::validation("automation_schedule_invalid", "Déclencheur invalide.")
        }))
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
    fn parses_supported_schedule_and_rejects_unknown_shape() {
        assert!(matches!(
            parse_schedule(Some(&json!({"kind":"daily", "time":"08:00"}))).unwrap(),
            WakeupSchedule::Daily { .. }
        ));
        assert!(parse_schedule(Some(&json!({"kind":"cron", "value":"* * * * *"}))).is_err());
    }
}
