use super::*;
use crate::services::agent_local::types_session::{AgentSession, CloneMode};
use chrono::Utc;
use git2::{Repository, Signature};
use uuid::Uuid;

fn session(id: &str, parent: Option<&str>) -> AgentSession {
    AgentSession {
        id: id.into(),
        name: id.into(),
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
        clone_parent_session_id: parent.map(str::to_string),
        clone_parent_message_id: Some("m1".into()),
        clone_mode: Some(CloneMode::Cut),
        clone_summary: None,
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: parent.map(str::to_string),
        git_branch: None,
    }
}

fn init_repo_with_branch(branch_name: &str) -> tempfile::TempDir {
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
    let commit = repo
        .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .expect("commit");
    let commit = repo.find_commit(commit).expect("find commit");
    repo.branch(branch_name, &commit, false).expect("branch");
    tmp
}

#[tokio::test]
async fn link_existing_branch_saves_session_and_tab() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let repo = init_repo_with_branch("feature/manual");
    let root = session(&root_id, None);
    let clone = session(&clone_id, Some(&root_id));
    super::session_store::save(&root).await.expect("save root");
    super::session_store::save(&clone)
        .await
        .expect("save clone");
    super::session_tabs::add_clone_tab(&root_id, &clone_id, "m1", CloneMode::Cut)
        .await
        .expect("add tab");

    let tabs = link_existing_branch(&root_id, &clone_id, repo.path(), "feature/manual")
        .await
        .expect("link branch");

    let saved = super::session_store::get(&clone_id)
        .await
        .expect("saved clone");
    assert_eq!(saved.git_branch.as_deref(), Some("feature/manual"));
    let tab = tabs
        .tabs
        .iter()
        .find(|tab| tab.session_id == clone_id)
        .expect("tab");
    assert_eq!(tab.git_branch.as_deref(), Some("feature/manual"));

    let _ = super::session_tabs::remove_session_from_tabs(&clone_id).await;
    let _ = super::session_tabs::remove_session_from_tabs(&root_id).await;
    let _ = super::session_store::delete_one(&clone_id).await;
    let _ = super::session_store::delete_one(&root_id).await;
}
