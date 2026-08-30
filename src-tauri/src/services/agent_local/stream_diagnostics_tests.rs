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
        preserve_reasoning: Default::default(),
        accumulated_tokens: 0,
        context_tokens: None,
        compression_profile_selection: None,
        compression_count: 0,
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

#[tokio::test]
async fn a_started_request_accepts_exactly_one_terminal_transition() {
    let session =
        super::session_store::create_full("Terminal once", "qwen3.5:4b", "ollama", false, None)
            .await
            .unwrap();
    let request_id = super::stream_diagnostics::start_request(&session.id, 1).await;
    super::stream_diagnostics::record_cancelled(&session.id, &request_id).await;
    super::stream_diagnostics::record_failure(
        &session.id,
        Some(&request_id),
        "conversation_admission_failed",
        false,
    )
    .await;

    let stored = super::session_store::get(&session.id).await.unwrap();
    let run = stored
        .diagnostic_runs
        .iter()
        .find(|run| run.request_id == request_id)
        .unwrap();
    assert_eq!(run.status, "cancelled");
    assert_eq!(
        run.events
            .iter()
            .filter(|event| event.phase == "failed")
            .count(),
        1
    );
    assert!(stored.stream_failures.is_empty());
    super::session_store::delete_one(&session.id).await.unwrap();
}
