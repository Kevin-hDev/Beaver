use serde_json::Value;

pub fn is_control_only(tool_calls: &[(String, Value)]) -> bool {
    !tool_calls.is_empty()
        && tool_calls.iter().all(|(name, _)| {
            matches!(
                name.as_str(),
                "list_subagents"
                    | "get_subagent"
                    | "message_subagent"
                    | "cancel_subagent"
                    | "archive_subagent"
            )
        })
}
