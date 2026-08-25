use super::*;
use chrono::Utc;

fn agent_message(role: &str, content: &str) -> AgentMessage {
    AgentMessage {
        id: "message-1".into(),
        role: role.into(),
        content: content.into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn session(messages: Vec<AgentMessage>) -> AgentSession {
    AgentSession {
        id: "session-1".into(),
        name: "Gateway".into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        pinned_at: None,
        model: "model".into(),
        provider: "provider".into(),
        thinking_enabled: false,
        fast_mode_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        context_tokens: None,
        messages,
        todos: Vec::new(),
        todo_neglect_count: 0,
        todo_runs: Vec::new(),
        active_todo_run_id: None,
        stream_failures: Vec::new(),
        diagnostic_runs: Vec::new(),
        plan_mode_enabled: false,
        plan_runs: Vec::new(),
        active_plan_id: None,
        plan_workflow_status: Default::default(),
        is_heartbeat: false,
        is_gateway: true,
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

#[test]
fn invalid_session_role_fails_without_exposing_the_role() {
    let private_role = "private-provider-role";
    let error = build_chat_messages(&session(vec![agent_message(private_role, "secret")]))
        .expect_err("an invalid role must block gateway history");

    assert_eq!(error, "Historique de session invalide.");
    assert!(!error.contains(private_role));
    assert!(!error.contains("secret"));
}

#[test]
fn valid_session_roles_keep_their_existing_gateway_shape() {
    let mut assistant = agent_message("assistant", "answer");
    assistant.thinking = Some("reasoning".into());
    let mut tool = agent_message("tool", "result");
    tool.tool_name = Some("lookup".into());
    let history = build_chat_messages(&session(vec![
        agent_message("system", "hidden"),
        agent_message("user", "question"),
        assistant,
        tool,
    ]))
    .expect("valid gateway history");

    assert_eq!(
        history
            .iter()
            .map(|message| message.role.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant", "tool"]
    );
    assert_eq!(history[1].display_thinking.as_deref(), Some("reasoning"));
    assert!(history[1].legacy_tool_loop_reasoning.is_none());
    assert_eq!(history[2].tool_name.as_deref(), Some("lookup"));
}

#[test]
fn scheduled_history_keeps_tool_calls_and_results() {
    let call = ToolCallOllama {
        id: Some("call-1".into()),
        extra_content: None,
        function: ToolCallFunction {
            name: "read_file".into(),
            arguments: serde_json::json!({"path":"README.md"}),
        },
    };
    let assistant = ChatMessage::assistant(String::new(), None, None, None, Some(vec![call]));
    let tool = ChatMessage::tool(
        "Beaver".into(),
        Some("call-1".into()),
        Some("read_file".into()),
    );

    let saved_call = chat_to_agent_message(&assistant).unwrap();
    let saved_result = chat_to_agent_message(&tool).unwrap();

    assert_eq!(saved_call.tool_calls.unwrap()[0].function.name, "read_file");
    assert_eq!(saved_result.role, "tool");
    assert_eq!(saved_result.tool_name.as_deref(), Some("read_file"));
    assert_eq!(saved_result.content, "Beaver");
}
