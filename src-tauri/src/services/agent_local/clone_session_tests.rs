use super::*;
use chrono::Utc;

fn message(id: &str, role: &str, content: &str) -> AgentMessage {
    AgentMessage {
        id: id.into(),
        role: role.into(),
        content: content.into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_activities: None,
        segments: None,
        files: vec![],
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

fn session() -> AgentSession {
    AgentSession {
        id: "550e8400-e29b-41d4-a716-446655440000".into(),
        name: "Main".into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        accumulated_tokens: 0,
        messages: vec![
            message("m1", "user", "start"),
            message("m2", "assistant", "answer"),
            message("m3", "user", "future"),
        ],
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

#[test]
fn build_clone_cuts_at_selected_message() {
    let source = session();
    let clone = build_clone(&source, "m2", CloneMode::Cut, 1, &source.id);

    assert_eq!(clone.messages.len(), 2);
    assert_eq!(clone.messages[1].id, "m2");
    assert_eq!(clone.clone_parent_session_id, Some(source.id));
    assert_eq!(clone.clone_parent_message_id, Some("m2".into()));
    assert_eq!(clone.clone_mode, Some(CloneMode::Cut));
    assert!(clone.clone_summary.is_none());
    assert!(clone.stream_failures.is_empty());
    assert!(clone.diagnostic_runs.is_empty());
}

#[test]
fn hidden_context_message_uses_clone_prefix() {
    let hidden = hidden_context_message("Useful summary");

    assert_eq!(hidden.role, "user");
    assert!(hidden
        .content
        .starts_with(clone_summary::CLONE_SUMMARY_PREFIX));
}

/// Construit une session qui est elle-même un clone (parent immédiat + racine
/// du groupe), pour simuler un clone-de-clone.
fn clone_session_as_source(root_id: &str, parent_id: &str) -> AgentSession {
    let mut source = session();
    source.id = parent_id.into();
    source.clone_parent_session_id = Some(root_id.into());
    source.clone_root_session_id = Some(root_id.into());
    source.clone_mode = Some(CloneMode::Summary);
    source
}

#[test]
fn build_clone_from_main_sets_root_to_main() {
    // Clone depuis la session principale : la racine du nouveau clone est
    // l'id de la session principale.
    let source = session();
    let clone = build_clone(&source, "m2", CloneMode::Cut, 1, &source.id);

    assert_eq!(
        clone.clone_root_session_id.as_deref(),
        Some(source.id.as_str())
    );
    // Le parent immédiat est aussi la racine dans ce cas.
    assert_eq!(
        clone.clone_parent_session_id.as_deref(),
        Some(source.id.as_str())
    );
}

#[test]
fn build_clone_from_clone_propagates_root_id() {
    // Clone depuis un clone : la racine est héritée du parent, mais le parent
    // immédiat est le clone intermédiaire (pas la racine).
    let root_id = "root-11111111-1111-1111-1111-111111111111";
    let clone_intermediate_id = "clone-22222222-2222-2222-2222-222222222222";
    let source = clone_session_as_source(root_id, clone_intermediate_id);
    let clone = build_clone(&source, "m2", CloneMode::Cut, 1, root_id);

    assert_eq!(clone.clone_root_session_id.as_deref(), Some(root_id));
    assert_eq!(
        clone.clone_parent_session_id.as_deref(),
        Some(clone_intermediate_id)
    );
    // Le parent immédiat et la racine diffèrent bien : on est sur un clone-de-clone.
    assert_ne!(clone.clone_parent_session_id, clone.clone_root_session_id);
}

#[test]
fn build_clone_does_not_inherit_git_branch() {
    let mut source = clone_session_as_source(
        "root-11111111-1111-1111-1111-111111111111",
        "clone-22222222-2222-2222-2222-222222222222",
    );
    source.git_branch = Some("clone-12345678".into());

    let clone = build_clone(
        &source,
        "m2",
        CloneMode::Cut,
        1,
        "root-11111111-1111-1111-1111-111111111111",
    );

    assert_eq!(clone.git_branch, None);
}
