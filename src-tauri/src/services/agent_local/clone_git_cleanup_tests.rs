use super::*;
use crate::services::agent_local::types_session::{AgentSession, CloneMode};
use crate::services::git::{branch, branch_delete};
use chrono::Utc;
use git2::{Repository, Signature};
use uuid::Uuid;

fn init_repo_with_commit() -> tempfile::TempDir {
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

fn session(id: &str, parent: Option<&str>, git_branch: Option<&str>) -> AgentSession {
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
        clone_parent_message_id: parent.map(|_| "m1".into()),
        clone_mode: parent.map(|_| CloneMode::Cut),
        clone_summary: None,
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: parent.map(str::to_string),
        git_branch: git_branch.map(str::to_string),
    }
}

#[tokio::test]
async fn cleanup_unlinks_branch_from_other_sessions() {
    let root_id = Uuid::new_v4().to_string();
    let clone_a_id = Uuid::new_v4().to_string();
    let clone_b_id = Uuid::new_v4().to_string();
    let branch_name = "clone-11111111";
    let repo = init_repo_with_commit();
    save_clone_tabs(&root_id, &clone_a_id, Some(&clone_b_id), branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_a_id);
    branch::create_branch(repo.path(), branch_name).expect("seed shared branch");

    let tabs = close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master"))
        .await
        .expect("cleanup shared branch");

    assert!(!branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));
    let archived = super::session_store::get(&clone_a_id)
        .await
        .expect("archived clone");
    assert!(archived.archived_at.is_some());
    let linked = super::session_store::get(&clone_b_id)
        .await
        .expect("other clone");
    assert_eq!(linked.git_branch, None);
    assert!(tabs.tabs.iter().all(|tab| tab.session_id != clone_a_id));
    assert!(tabs
        .tabs
        .iter()
        .find(|tab| tab.session_id == clone_b_id)
        .is_some_and(|tab| tab.git_branch.is_none()));

    cleanup_sessions(&root_id, &[clone_a_id, clone_b_id]).await;
}

#[tokio::test]
async fn cleanup_prefers_main_over_temporary_checkpoint() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let branch_name = "clone-33333333";
    let repo = init_repo_with_commit();
    branch::create_branch(repo.path(), "Delete-branch").expect("create fallback");
    branch::create_branch(repo.path(), "main").expect("create main");
    branch::create_branch(repo.path(), branch_name).expect("create clone branch");
    save_clone_tabs(&root_id, &clone_id, None, branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_id);

    close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("Delete-branch"))
        .await
        .expect("cleanup with stale checkpoint");

    assert!(!branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));
    assert_eq!(branch::get_context(repo.path()).branch, "main");

    cleanup_sessions(&root_id, &[clone_id]).await;
}

#[tokio::test]
async fn cleanup_deletes_manually_linked_branch() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let branch_name = "feature/shared";
    let repo = init_repo_with_commit();
    branch::create_branch(repo.path(), branch_name).expect("create manual branch");
    save_clone_tabs(&root_id, &clone_id, None, branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_id);

    close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master"))
        .await
        .expect("cleanup manual branch");

    assert!(!branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));
    let archived = super::session_store::get(&clone_id)
        .await
        .expect("archived clone");
    assert!(archived.archived_at.is_some());
    assert_eq!(archived.git_branch, None);

    cleanup_sessions(&root_id, &[clone_id]).await;
}

#[tokio::test]
async fn cleanup_refuses_to_delete_main_branch() {
    let root_id = Uuid::new_v4().to_string();
    let clone_id = Uuid::new_v4().to_string();
    let branch_name = "main";
    let repo = init_repo_with_commit();
    branch::create_branch(repo.path(), branch_name).expect("create main");
    save_clone_tabs(&root_id, &clone_id, None, branch_name).await;
    let tabs = super::session_tabs::list(&root_id).await.expect("tabs");
    let tab_id = tab_id_for_session(&tabs, &clone_id);

    let result =
        close_tab_with_branch_cleanup(&root_id, &tab_id, repo.path(), Some("master")).await;

    assert_eq!(result, Err(GitActionError::ProtectedBranch));
    assert!(branch_delete::branch_exists(repo.path(), branch_name).expect("branch check"));

    cleanup_sessions(&root_id, &[clone_id]).await;
}

async fn save_clone_tabs(root_id: &str, clone_a_id: &str, clone_b_id: Option<&str>, branch: &str) {
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

fn tab_id_for_session(tabs: &super::session_tabs::SessionTabs, session_id: &str) -> String {
    tabs.tabs
        .iter()
        .find(|tab| tab.session_id == session_id)
        .expect("clone tab")
        .tab_id
        .clone()
}

async fn cleanup_sessions(root_id: &str, clone_ids: &[String]) {
    for clone_id in clone_ids {
        let _ = super::session_tabs::remove_session_from_tabs(clone_id).await;
        let _ = super::session_store::delete_one(clone_id).await;
    }
    let _ = super::session_tabs::remove_session_from_tabs(root_id).await;
    let _ = super::session_store::delete_one(root_id).await;
}
