use super::stream_diagnostics::push_failure;
use super::stream_diagnostics_failure::classify_error;
use super::types_session::AgentSession;
use chrono::Utc;

#[test]
fn stable_provider_failures_keep_only_actionable_codes() {
    assert_eq!(
        classify_error("provider_request_rejected", false),
        "provider_error"
    );
    assert_eq!(
        classify_error("provider_configuration_invalid", false),
        "provider_configuration_invalid"
    );
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
        schema_version: super::session_limits::CURRENT_SESSION_SCHEMA_VERSION,
        id: "abc-123".into(),
        name: "Test".into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        pinned_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        fast_mode_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        context_tokens: None,
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
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        working_dir_managed: false,
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
