use super::{session_store, subagent_git_actions, subagent_git_run, subagent_worktree};
use std::process::Command;

#[tokio::test]
async fn apply_preserves_unrelated_parent_changes() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let parent = session_store::create_full(
        "Parent dirty",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("parent");
    let mut child = session_store::create_full(
        "Child dirty",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    session_store::save(&child).await.expect("save child");
    let execution = uuid::Uuid::new_v4().to_string();
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    tokio::fs::write(worktree.join("child.txt"), "from child\n")
        .await
        .expect("child file");
    let change = subagent_git_run::capture(repo.path(), &child.id, &execution, &worktree)
        .await
        .expect("capture")
        .expect("change");
    subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child.id, &execution)
        .await
        .expect("remove worktree");

    std::fs::write(repo.path().join("README.md"), "local edit\n").expect("tracked edit");
    std::fs::write(repo.path().join("local.txt"), "untracked\n").expect("untracked file");
    let before = status(repo.path(), &["README.md", "local.txt"]);

    subagent_git_actions::apply(repo.path(), &parent.id, &child.id, &change.id)
        .await
        .expect("apply with unrelated changes");

    assert_eq!(status(repo.path(), &["README.md", "local.txt"]), before);
    assert_eq!(
        std::fs::read_to_string(repo.path().join("child.txt")).unwrap(),
        "from child\n"
    );
    let _ = super::subagent_change_store::remove(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}

#[tokio::test]
async fn conflict_restores_unrelated_parent_changes_exactly() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let (parent, child) = sessions().await;
    let execution = uuid::Uuid::new_v4().to_string();
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    tokio::fs::write(worktree.join("README.md"), "child edit\n")
        .await
        .expect("child edit");
    let change = subagent_git_run::capture(repo.path(), &child.id, &execution, &worktree)
        .await
        .expect("capture")
        .expect("change");
    subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child.id, &execution)
        .await
        .expect("remove worktree");
    std::fs::write(repo.path().join("README.md"), "parent edit\n").expect("parent edit");
    commit_all(repo.path(), "parent conflict");
    std::fs::write(repo.path().join("local.txt"), "keep me\n").expect("local file");
    let head = git(repo.path(), &["rev-parse", "HEAD"]);
    let before = status(repo.path(), &["README.md", "local.txt"]);

    assert!(subagent_git_actions::apply(repo.path(), &parent.id, &child.id, &change.id)
        .await
        .is_err());

    assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(status(repo.path(), &["README.md", "local.txt"]), before);
    assert_eq!(std::fs::read_to_string(repo.path().join("local.txt")).unwrap(), "keep me\n");
    assert_eq!(
        super::subagent_change_store::load(&child.id).await.unwrap().status,
        super::types_subagent_change::SubagentChangeStatus::Conflict
    );
    cleanup(&parent.id, &child.id).await;
}

async fn sessions() -> (super::types_session::AgentSession, super::types_session::AgentSession) {
    let parent = session_store::create_full(
        "Parent dirty",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("parent");
    let mut child = session_store::create_full(
        "Child dirty",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    session_store::save(&child).await.expect("save child");
    (parent, child)
}

fn status(repo: &std::path::Path, paths: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["status", "--short", "--"])
        .args(paths)
        .output()
        .expect("git status");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("utf8")
}

fn git(repo: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(args)
        .output()
        .expect("git");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("utf8").trim().to_string()
}

fn commit_all(repo: &std::path::Path, message: &str) {
    assert!(Command::new("git").args(["-C"]).arg(repo).args(["add", "-A"]).status().unwrap().success());
    assert!(Command::new("git").args(["-C"]).arg(repo).args([
        "-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-m", message,
    ]).status().unwrap().success());
}

async fn cleanup(parent_id: &str, child_id: &str) {
    let _ = super::subagent_change_store::remove(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
