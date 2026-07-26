#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_validate::validate;
    use serde_json::json;

    #[test]
    fn valid_bash() {
        let args = json!({"command": "ls", "timeout": 30});
        assert!(validate("bash", &args).is_ok());
    }

    #[test]
    fn bash_missing_command() {
        let args = json!({"timeout": 30});
        let err = validate("bash", &args).unwrap_err();
        assert!(err.contains("command"));
    }

    #[test]
    fn bash_wrong_type() {
        let args = json!({"command": 42});
        let err = validate("bash", &args).unwrap_err();
        assert!(err.contains("string"));
    }

    #[test]
    fn strips_unknown_args() {
        let args = json!({"command": "ls", "inject": "evil"});
        let cleaned = validate("bash", &args).unwrap();
        assert!(cleaned.get("inject").is_none());
        assert!(cleaned.get("command").is_some());
    }

    #[test]
    fn optional_args_absent() {
        let args = json!({"command": "ls"});
        assert!(validate("bash", &args).is_ok());
    }

    #[test]
    fn read_file_valid() {
        let args = json!({"path": "foo.rs", "offset": 0, "limit": 100});
        assert!(validate("read_file", &args).is_ok());
    }

    #[test]
    fn edit_file_missing_old_string() {
        let args = json!({"path": "f.rs", "new_string": "x"});
        let err = validate("edit_file", &args).unwrap_err();
        assert!(err.contains("old_string"));
    }

    #[test]
    fn search_mcp_valid() {
        let args = json!({"mode": "call", "tool_id": "svc.tool", "arguments": {"key": "val"}});
        assert!(validate("search_mcp_tools", &args).is_ok());
    }

    #[test]
    fn todo_write_requires_todos_array() {
        assert!(validate("todo_write", &json!({"todos": []})).is_ok());
        assert!(validate("todo_write", &json!({"todos": "nope"})).is_err());
    }

    #[test]
    fn hidden_todo_tools_validate_args() {
        assert!(validate("todo_history", &json!({})).is_ok());
        assert!(validate("agent_diagnostics", &json!({})).is_ok());
        assert!(validate("agent_diagnostics", &json!({"limit": 10})).is_ok());
        assert!(validate("agent_diagnostics", &json!({"limit": "10"})).is_err());
        assert!(validate("todo_pause", &json!({"reason": "debug"})).is_ok());
        assert!(validate("todo_resume", &json!({"id": "abc"})).is_ok());
        assert!(validate("todo_resume", &json!({})).is_err());
        assert!(validate("todo_delete", &json!({"id": "abc"})).is_ok());
        assert!(validate("todo_delete", &json!({"active": true})).is_ok());
        assert!(validate("todo_delete", &json!({"active": false})).is_err());
        assert!(validate("todo_delete", &json!({"active": "true"})).is_err());
        assert!(validate("todo_delete", &json!({"id": "abc", "active": true})).is_err());
        assert!(validate("todo_delete", &json!({})).is_err());
    }

    #[test]
    fn plan_tools_validate_args() {
        assert!(validate("planmode", &json!({"title": "Plan", "content": "Steps"})).is_ok());
        assert!(validate("planmode", &json!({"title": "Plan"})).is_err());
        assert!(validate("exitplanmode", &json!({"status": "approved"})).is_ok());
        assert!(validate("exitplanmode", &json!({})).is_err());
    }

    #[test]
    fn subagent_tools_validate_new_args() {
        let cleaned = validate(
            "delegate_task",
            &json!({
                "prompt": "Analyse",
                "subagent_type": "explorer",
                "mode": "detach",
                "display_name": "Geminitor",
                "description": "Analyse ciblée",
                "subagent_id": "child"
            }),
        )
        .unwrap();
        assert!(cleaned.get("mode").is_none());
        assert!(validate(
            "message_subagent",
            &json!({"subagent_id": "a", "prompt": "Suite"})
        )
        .is_ok());
        assert!(validate("archive_subagent", &json!({"subagent_id": "a"})).is_ok());
        assert!(validate("archive_subagent", &json!({})).is_err());
    }

    #[test]
    fn unknown_tool_passes_through() {
        let args = json!({"anything": true});
        assert!(validate("unknown_future_tool", &args).is_ok());
    }

    #[test]
    fn non_object_args_rejected() {
        let args = json!("not an object");
        assert!(validate("bash", &args).is_err());
    }

    #[test]
    fn write_spreadsheet_valid() {
        let args = json!({"path": "out.xlsx", "operations": [{"type": "set_cell"}]});
        assert!(validate("write_spreadsheet", &args).is_ok());
    }

    #[test]
    fn forecast_keeps_reusable_profile_and_read_pagination() {
        let cleaned = validate(
            "forecast",
            &json!({
                "data_profile_id": "profile-1",
                "target_column": "sales",
                "date_column": "date",
                "horizon": 7,
                "frequency": "D",
                "confidence_level": 0.8
            }),
        )
        .unwrap();
        assert_eq!(cleaned["data_profile_id"], "profile-1");

        let read = validate(
            "forecast_read",
            &json!({"analysis_id": "analysis-1", "offset": 200, "limit": 100}),
        )
        .unwrap();
        assert_eq!(read["offset"], 200);
        assert_eq!(read["limit"], 100);
    }

    #[test]
    fn forecast_tools_enforce_declared_bounds() {
        assert!(validate("forecast_read", &json!({"offset": -1})).is_err());
        assert!(validate("forecast_read", &json!({"limit": 0})).is_err());
        assert!(validate("forecast_read", &json!({"limit": 201})).is_err());
        assert!(validate(
            "forecast_data_audit",
            &json!({
                "data": "[]",
                "target_column": "x".repeat(81),
                "date_column": "date",
                "horizon": 1,
                "frequency": "D"
            })
        )
        .is_err());
    }

    #[test]
    fn forecast_audit_rejects_existing_profile_ids() {
        let cleaned = validate(
            "forecast_data_audit",
            &json!({
                "data": "[]",
                "data_profile_id": "profile-1",
                "target_column": "sales",
                "date_column": "date",
                "horizon": 7,
                "frequency": "D",
                "confidence_level": 0.8
            }),
        )
        .unwrap();

        assert!(cleaned.get("data_profile_id").is_none());
    }

    #[test]
    fn null_required_arg_rejected() {
        let args = json!({"command": null});
        assert!(validate("bash", &args).is_err());
    }
}
