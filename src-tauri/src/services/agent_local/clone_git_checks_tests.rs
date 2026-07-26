use super::*;
use crate::services::agent_local::types_session::AgentSession;
use chrono::Utc;

const ROOT_ID: &str = "root-aaaaaaaa-1111-1111-1111-111111111111";
const INTERMEDIATE_ID: &str = "clone-bbbbbbbb-2222-2222-2222-222222222222";

fn clone_session(id: &str, parent_id: &str, root_id: Option<&str>) -> AgentSession {
    AgentSession {
        id: id.into(),
        name: "Clone".into(),
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
        clone_parent_session_id: Some(parent_id.into()),
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: root_id.map(str::to_string),
        git_branch: None,
    }
}

#[tokio::test]
async fn ensure_clone_belongs_to_root_accepts_clone_of_clone() {
    let intermediate = clone_session(INTERMEDIATE_ID, ROOT_ID, Some(ROOT_ID));

    assert!(ensure_clone_belongs_to_root(&intermediate, ROOT_ID)
        .await
        .is_ok());
    assert!(ensure_clone_belongs_to_root_string(&intermediate, ROOT_ID)
        .await
        .is_ok());

    let clone_of_clone_id = "clone-of-clone";
    let clone_of_clone = clone_session(clone_of_clone_id, INTERMEDIATE_ID, Some(ROOT_ID));

    assert!(ensure_clone_belongs_to_root(&clone_of_clone, ROOT_ID)
        .await
        .is_ok());
    assert!(
        ensure_clone_belongs_to_root_string(&clone_of_clone, ROOT_ID)
            .await
            .is_ok()
    );

    let other_root = "other-cccccccc-3333-3333-3333-333333333333";
    assert!(ensure_clone_belongs_to_root(&clone_of_clone, other_root)
        .await
        .is_err());
}

#[tokio::test]
async fn clone_linked_branch_uses_session_as_source_of_truth() {
    let mut clone = clone_session(INTERMEDIATE_ID, ROOT_ID, None);
    clone.git_branch = Some("clone-12345678".into());

    let branch = clone_linked_branch(&clone, ROOT_ID)
        .await
        .expect("linked branch");

    assert_eq!(branch.as_deref(), Some("clone-12345678"));
}
