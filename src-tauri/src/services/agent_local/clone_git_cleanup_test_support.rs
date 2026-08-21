use crate::services::agent_local::types_session::{AgentSession, CloneMode};
use chrono::Utc;
use git2::{Repository, Signature};

pub(super) fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("README.md"), "init").expect("write file");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("README.md"))
        .expect("add");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let sig = Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    tmp
}

pub(super) fn session(
    id: &str,
    parent: Option<&str>,
    git_branch: Option<&str>,
) -> AgentSession {
    AgentSession {
        id: id.into(),
        name: id.into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        pinned_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
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
        clone_parent_session_id: parent.map(str::to_string),
        clone_parent_message_id: parent.map(|_| "m1".into()),
        clone_mode: parent.map(|_| CloneMode::Cut),
        clone_summary: None,
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: parent.map(str::to_string),
        git_branch: git_branch.map(str::to_string),
    }
}

pub(super) async fn save_clone_tabs(
    root_id: &str,
    clone_a_id: &str,
    clone_b_id: Option<&str>,
    branch: &str,
) {
    super::session_store::save(&session(root_id, None, None))
        .await
        .expect("save root");
    super::session_store::save(&session(clone_a_id, Some(root_id), Some(branch)))
        .await
        .expect("save clone a");
    super::session_tabs::add_clone_tab(root_id, clone_a_id, "m1", CloneMode::Cut)
        .await
        .expect("add clone a tab");
    if let Some(clone_b_id) = clone_b_id {
        super::session_store::save(&session(clone_b_id, Some(root_id), Some(branch)))
            .await
            .expect("save clone b");
        super::session_tabs::add_clone_tab(root_id, clone_b_id, "m1", CloneMode::Cut)
            .await
            .expect("add clone b tab");
    }
}

pub(super) fn tab_id_for_session(
    tabs: &super::session_tabs::SessionTabs,
    session_id: &str,
) -> String {
    tabs.tabs
        .iter()
        .find(|tab| tab.session_id == session_id)
        .expect("clone tab")
        .tab_id
        .clone()
}

pub(super) async fn cleanup_sessions(root_id: &str, clone_ids: &[String]) {
    for clone_id in clone_ids {
        let _ = super::session_tabs::remove_session_from_tabs(clone_id).await;
        let _ = super::session_store::delete_one(clone_id).await;
    }
    let _ = super::session_tabs::remove_session_from_tabs(root_id).await;
    let _ = super::session_store::delete_one(root_id).await;
}
