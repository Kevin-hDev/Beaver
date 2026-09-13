use super::{cleanup_execution, recover_and_remove_orphan, serialize_prompt_metadata};
use crate::services::agent_local::types_subagent_change::{
    SubagentChangeMeta, SubagentChangeStatus, SubagentWorkspaceKind,
};
use chrono::Utc;
use serde_json::json;

#[test]
fn emitted_metadata_uses_the_exact_tool_argument_names() {
    let change = pending_change();
    let encoded = serialize_prompt_metadata(&change).expect("serialize metadata");
    let metadata: serde_json::Value = serde_json::from_str(&encoded).expect("parse metadata");

    assert_eq!(metadata["subagent_id"], change.child_session_id);
    assert_eq!(metadata["change_id"], change.id);
    assert_eq!(metadata["child_session_id"], metadata["subagent_id"]);
    assert_eq!(metadata["id"], metadata["change_id"]);

    let args = json!({
        "subagent_id": metadata["subagent_id"],
        "change_id": metadata["change_id"],
    });
    for tool in [
        "inspect_subagent_changes",
        "apply_subagent_changes",
        "discard_subagent_changes",
    ] {
        assert!(crate::services::agent_local::tool_validate::validate(tool, &args).is_ok());
    }
}

fn pending_change() -> SubagentChangeMeta {
    let now = Utc::now();
    SubagentChangeMeta {
        id: uuid::Uuid::new_v4().to_string(),
        child_session_id: uuid::Uuid::new_v4().to_string(),
        project_id: "project".into(),
        base_commit: "a".repeat(40),
        commit: "b".repeat(40),
        branch: "codex/subagent-change".into(),
        target_branch: "main".into(),
        workspace_kind: SubagentWorkspaceKind::Git,
        changed_paths: Vec::new(),
        paths_truncated: false,
        status: SubagentChangeStatus::Pending,
        created_at: now,
        updated_at: now,
        applied_commit: None,
    }
}

#[tokio::test]
async fn normal_capture_failure_keeps_worktree_and_branch() {
    let fixture = coder_fixture().await;
    tokio::fs::write(fixture.worktree.join("unfinished.txt"), "keep me\n")
        .await
        .expect("unfinished change");
    super::fail_next_capture_for_child(&fixture.child.id).await;
    let capture = super::capture(
        fixture.repo.path(),
        &fixture.child.id,
        &fixture.execution,
        &fixture.worktree,
    )
    .await;
    assert!(capture.is_err());

    cleanup_execution(
        fixture.repo.path(),
        &fixture.child.id,
        &fixture.execution,
        fixture.child.subagent_worktree.as_deref(),
        true,
        true,
    )
    .await;

    assert!(fixture.worktree.is_dir());
    assert!(branch_exists(fixture.repo.path(), &fixture.execution));
    fixture.cleanup().await;
}

#[tokio::test]
async fn missing_project_keeps_recoverable_worktree() {
    let mut fixture = coder_fixture().await;
    fixture.child.project_id = None;
    fixture.child.working_dir = "/missing/beaver-subagent-project".into();
    crate::services::agent_local::session_store::save(&fixture.child)
        .await
        .expect("save missing project");

    assert!(recover_and_remove_orphan(&fixture.child).await.is_err());
    assert!(fixture.worktree.is_dir());
    assert!(branch_exists(fixture.repo.path(), &fixture.execution));
    fixture.cleanup().await;
}

#[tokio::test]
async fn empty_capture_removes_worktree_and_branch() {
    let fixture = coder_fixture().await;

    recover_and_remove_orphan(&fixture.child)
        .await
        .expect("empty recovery");

    assert!(!fixture.worktree.exists());
    assert!(!branch_exists(fixture.repo.path(), &fixture.execution));
    fixture.cleanup().await;
}

struct CoderFixture {
    repo: super::super::subagent_worktree_ownership_tests::TestRepository,
    parent: crate::services::agent_local::types_session::AgentSession,
    child: crate::services::agent_local::types_session::AgentSession,
    execution: String,
    worktree: std::path::PathBuf,
}

async fn coder_fixture() -> CoderFixture {
    use crate::services::agent_local::{session_store, subagent_worktree};

    let repo = super::super::subagent_worktree_ownership_tests::init_repo_with_commit().await;
    let parent = session_store::create_full("Capture parent", "model", "provider", false, None)
        .await
        .expect("parent");
    let mut child = session_store::create_full(
        "Capture coder",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.working_dir = repo.path().to_string_lossy().into_owned();
    let execution = uuid::Uuid::new_v4().to_string();
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    child.subagent_worktree = Some(worktree.to_string_lossy().into_owned());
    session_store::save(&child).await.expect("save coder");
    CoderFixture {
        repo,
        parent,
        child,
        execution,
        worktree,
    }
}

impl CoderFixture {
    async fn cleanup(self) {
        use crate::services::agent_local::{session_store, subagent_worktree};

        if self.worktree.exists() {
            let _ = subagent_worktree::remove_owned(
                &self.worktree.to_string_lossy(),
                &self.child.id,
                &self.execution,
            )
            .await;
        }
        if let Ok(branch) = subagent_worktree::branch_for_execution(&self.execution) {
            let _ = crate::services::agent_local::subagent_git_command::delete_branch(
                self.repo.path(),
                &branch,
            )
            .await;
        }
        let _ = crate::services::agent_local::subagent_change_store::remove(&self.child.id).await;
        session_store::delete_one(&self.child.id)
            .await
            .expect("delete child");
        session_store::delete_one(&self.parent.id)
            .await
            .expect("delete parent");
    }
}

fn branch_exists(repo: &std::path::Path, execution: &str) -> bool {
    let branch = crate::services::agent_local::subagent_worktree::branch_for_execution(execution)
        .expect("branch");
    std::process::Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["show-ref", "--verify", "--quiet"])
        .arg(format!("refs/heads/{branch}"))
        .status()
        .expect("git branch query")
        .success()
}
