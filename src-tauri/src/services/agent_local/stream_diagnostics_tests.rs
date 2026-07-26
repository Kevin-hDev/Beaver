use super::stream_diagnostics::push_failure;
use super::stream_diagnostics_failure::classify_error;
use super::types_session::AgentSession;
use chrono::Utc;

#[test]
fn stable_provider_failures_are_provider_errors() {
    for code in [
        "provider_connection_failed",
        "provider_request_rejected",
        "provider_configuration_invalid",
    ] {
        assert_eq!(classify_error(code, false), "provider_error");
    }
}

#[test]
fn provider_overload_is_classified_explicitly() {
    assert_eq!(
        classify_error(
            "Codex: Our servers are currently overloaded. Please try again later.",
            false
        ),
        "provider_overloaded"
    );
}

#[test]
fn stream_failures_are_bounded_and_sanitized() {
    let mut session = test_session();
    for i in 0..25 {
        push_failure(
            &mut session,
            &format!("secret /Users/kevinh/project token-{i}"),
            false,
        );
    }

    assert_eq!(session.stream_failures.len(), 20);
    assert!(session
        .stream_failures
        .iter()
        .all(|failure| failure.code == "stream_error"));
}

fn test_session() -> AgentSession {
    AgentSession {
        id: "abc-123".into(),
        name: "Test".into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: vec![],
        todos: vec![],
        todo_neglect_count: 0,
        todo_runs: vec![],
        active_todo_run_id: None,
        stream_failures: vec![],
        diagnostic_runs: vec![],
        plan_mode_enabled: false,
        plan_runs: vec![],
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        plan_approval_decision: None,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        parent_session_id: None,
        subagent_type: None,
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        subagent_queued_prompts: Vec::new(),
        subagent_hidden_reports: Vec::new(),
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        clone_root_session_id: None,
        git_branch: None,
    }
}
