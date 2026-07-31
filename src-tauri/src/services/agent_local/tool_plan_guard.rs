use serde_json::Value;

pub const PLAN_MODE_ALLOWED_TOOL_NAMES: &[&str] = &[
    "read_file",
    "grep",
    "glob",
    "list_dir",
    "web_search",
    "web_fetch",
    "search_extension_tools",
    "read_spreadsheet",
    "read_document",
    "read_image",
    "bash_write",
    "load_skill",
    "todo_history",
    "todo_pause",
    "todo_resume",
    "todo_delete",
    "agent_diagnostics",
    "ask_user_choice",
    "planmode",
    "forecast_read",
    "forecast_models",
];

pub const PLAN_MODE_ALLOWED_ACTIONS_TEXT: &str = "read_file, list_dir, grep, glob, web_search, web_fetch, search_extension_tools, read_spreadsheet, read_document, read_image, bash_write, load_skill, todo_history, todo_pause, todo_resume, todo_delete, agent_diagnostics, ask_user_choice, planmode, forecast_read, forecast_models, safe bash exploration and validation commands (including tests and builds), and search_mcp_tools without MCP calls";

pub fn is_allowed_in_plan_mode(tool_name: &str, args: &Value) -> bool {
    match tool_name {
        "bash" => !super::permission_gate::requires_permission("bash", args),
        "search_mcp_tools" => args.get("mode").and_then(Value::as_str) != Some("call"),
        _ => PLAN_MODE_ALLOWED_TOOL_NAMES.contains(&tool_name),
    }
}

pub fn ensure_allowed(tool_name: &str, args: &Value, plan_mode_active: bool) -> Result<(), String> {
    if !plan_mode_active || is_allowed_in_plan_mode(tool_name, args) {
        return Ok(());
    }
    Err("Action unavailable while Plan Mode is active.".to_string())
}

pub async fn ensure_allowed_for_session(
    tool_name: &str,
    args: &Value,
    session_id: &str,
    fallback_plan_mode_active: bool,
) -> Result<(), String> {
    let session_plan_mode_active = super::session_store::get(session_id)
        .await
        .map(|session| session.plan_mode_enabled)
        .unwrap_or(false);
    let plan_mode_active =
        effective_plan_mode(fallback_plan_mode_active, session_plan_mode_active);
    ensure_allowed(tool_name, args, plan_mode_active)
}

fn effective_plan_mode(batch_started_in_plan_mode: bool, current_plan_mode: bool) -> bool {
    batch_started_in_plan_mode || current_plan_mode
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn blocks_write_tools_in_plan_mode() {
        assert!(super::ensure_allowed("write_file", &json!({}), true).is_err());
        assert!(super::ensure_allowed("edit_file", &json!({}), true).is_err());
        assert!(super::ensure_allowed("todo_write", &json!({}), true).is_err());
        assert!(super::ensure_allowed("create_branch", &json!({}), true).is_err());
        assert!(super::ensure_allowed("delegate_task", &json!({}), true).is_err());
    }

    #[test]
    fn allows_write_tools_outside_plan_mode() {
        assert!(super::ensure_allowed("write_file", &json!({}), false).is_ok());
        assert!(super::ensure_allowed("todo_write", &json!({}), false).is_ok());
    }

    #[test]
    fn allows_read_tools_in_plan_mode() {
        assert!(super::ensure_allowed("read_file", &json!({}), true).is_ok());
        assert!(super::ensure_allowed("grep", &json!({}), true).is_ok());
        assert!(super::ensure_allowed("search_extension_tools", &json!({}), true).is_ok());
        assert!(super::ensure_allowed("planmode", &json!({}), true).is_ok());
        assert!(super::ensure_allowed(
            "bash_write",
            &json!({"session_id": "session"}),
            true
        )
        .is_ok());
        assert!(super::ensure_allowed(
            "bash_write",
            &json!({"session_id": "session", "stop": true}),
            true
        )
        .is_ok());
    }

    #[test]
    fn current_batch_stays_guarded_after_plan_approval() {
        assert!(super::effective_plan_mode(true, false));
        assert!(super::effective_plan_mode(false, true));
        assert!(!super::effective_plan_mode(false, false));
    }

    #[test]
    fn allows_safe_validation_commands_in_plan_mode() {
        assert!(super::ensure_allowed("bash", &json!({"command": "rm file"}), true).is_err());
        assert!(super::ensure_allowed("bash", &json!({"command": "git status"}), true).is_ok());
        assert!(super::ensure_allowed("bash", &json!({"command": "cargo test"}), true).is_ok());
        assert!(super::ensure_allowed("bash", &json!({"command": "cargo check"}), true).is_ok());
        assert!(super::ensure_allowed("bash", &json!({"command": "npm run build"}), true).is_ok());
        assert!(super::ensure_allowed("bash", &json!({"command": "npm test"}), true).is_ok());
        assert!(super::ensure_allowed("bash", &json!({"command": "npx tsc --noEmit"}), true).is_ok());
    }

    #[test]
    fn plan_mode_keeps_existing_shell_sessions_usable() {
        assert!(super::ensure_allowed(
            "bash_write",
            &json!({"session_id": "session", "chars": "test input\n"}),
            true
        )
        .is_ok());
        assert!(super::ensure_allowed(
            "bash_write",
            &json!({"session_id": "session", "eof": true}),
            true
        )
        .is_ok());
    }
}
