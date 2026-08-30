use super::stream_diagnostics_failure::apply_failure;
use super::stream_diagnostics_support::{event, push_event};
use super::types_diagnostics::AgentDiagnosticRun;
use super::types_session::AgentSession;
use chrono::Utc;

#[test]
fn diagnostic_events_are_bounded() {
    let mut run = test_run();
    for i in 0..20 {
        push_event(
            &mut run,
            "tool_execution",
            &format!("event {i}"),
            None,
            None,
        );
    }
    assert_eq!(run.events.len(), 12);
    assert_eq!(run.events[0].message, "event 8");
}

#[test]
fn failure_summary_uses_last_tool_without_raw_error() {
    let mut session = test_session();
    session.diagnostic_runs.push(test_run());
    session.diagnostic_runs[0].last_tool = Some(super::types_diagnostics::AgentDiagnosticTool {
        name: "write_file".to_string(),
        status: "started".to_string(),
        args: None,
        is_error: false,
        result_status: None,
        error_code: None,
        error_category: None,
        retryable: None,
        truncated: false,
        warning_count: 0,
    });
    apply_failure(&mut session, 0, "/Users/kevinh/secret stack trace", false);
    let run = &session.diagnostic_runs[0];
    assert_eq!(run.status, "failed");
    assert_eq!(run.error_type.as_deref(), Some("unknown"));
    assert_eq!(
        run.safe_summary.as_deref(),
        Some("Interruption après le dernier tool write_file (unknown).")
    );
}

#[test]
fn failure_keeps_compression_phase_and_classifies_wrapped_ollama_loss() {
    let mut session = test_session();
    let mut run = test_run();
    run.phase = "compression".to_string();
    session.diagnostic_runs.push(run);

    apply_failure(
        &mut session,
        0,
        "Compression Ollama : ollama_connection_lost",
        false,
    );

    let run = &session.diagnostic_runs[0];
    assert_eq!(run.phase, "compression");
    assert_eq!(run.error_type.as_deref(), Some("connection_lost"));
    assert_eq!(
        run.safe_summary.as_deref(),
        Some("Interruption pendant compression (connection_lost).")
    );
}

#[test]
fn max_turns_summary_does_not_blame_last_tool() {
    let mut session = test_session();
    session.diagnostic_runs.push(test_run());
    session.diagnostic_runs[0].last_tool = Some(super::types_diagnostics::AgentDiagnosticTool {
        name: "list_dir".to_string(),
        status: "detected".to_string(),
        args: None,
        is_error: false,
        result_status: None,
        error_code: None,
        error_category: None,
        retryable: None,
        truncated: false,
        warning_count: 0,
    });

    apply_failure(
        &mut session,
        0,
        "Limite de tours agent atteinte (200).",
        false,
    );

    let run = &session.diagnostic_runs[0];
    assert_eq!(run.error_type.as_deref(), Some("max_turns"));
    assert_eq!(
        run.safe_summary.as_deref(),
        Some("Limite de tours agent atteinte après le dernier tool list_dir.")
    );
}

#[test]
fn codex_api_500_is_provider_error() {
    let mut session = test_session();
    session.diagnostic_runs.push(test_run());

    apply_failure(
        &mut session,
        0,
        "Codex API 500 Internal Server Error",
        false,
    );

    let run = &session.diagnostic_runs[0];
    assert_eq!(run.error_type.as_deref(), Some("provider_error"));
}

#[test]
fn plan_mode_workflow_failure_has_clear_summary() {
    let mut session = test_session();
    session.diagnostic_runs.push(test_run());

    apply_failure(
        &mut session,
        0,
        "Plan Mode workflow could not be enforced.",
        false,
    );

    let run = &session.diagnostic_runs[0];
    assert_eq!(run.error_type.as_deref(), Some("tool_error"));
    assert_eq!(
        run.safe_summary.as_deref(),
        Some("Workflow Plan Mode non respecté par le modèle.")
    );
}

fn test_run() -> AgentDiagnosticRun {
    let now = Utc::now();
    AgentDiagnosticRun {
        request_id: "req-1".to_string(),
        generation: 1,
        status: "running".to_string(),
        severity: "info".to_string(),
        started_at: now,
        updated_at: now,
        ended_at: None,
        phase: "request_start".to_string(),
        error_type: None,
        last_tool: None,
        recent_tools: vec![],
        active_todo: None,
        safe_summary: None,
        events: vec![event("request_start", "start", None, None)],
    }
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
