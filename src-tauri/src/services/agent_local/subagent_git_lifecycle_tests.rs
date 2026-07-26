use super::{session_store, subagent_git_actions, subagent_git_run, subagent_worktree};
use std::process::Command;

#[tokio::test]
async fn captures_applies_and_cleans_temporary_branch() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    std::fs::write(repo.path().join("delete.txt"), "delete me\n").expect("seed delete");
    commit_all(repo.path(), "seed delete");
    let (parent, child, execution) = sessions().await;
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    tokio::fs::write(worktree.join("added.txt"), "from child\n")
        .await
        .expect("write");
    tokio::fs::write(worktree.join("README.md"), "changed\n")
        .await
        .expect("modify");
    tokio::fs::remove_file(worktree.join("delete.txt"))
        .await
        .expect("delete");

    let change = subagent_git_run::capture(repo.path(), &child.id, &execution, &worktree)
        .await
        .expect("capture")
        .expect("change");
    let paths = change
        .changed_paths
        .iter()
        .map(|entry| (entry.path.as_str(), entry.kind.as_str()))
        .collect::<Vec<_>>();
    assert!(paths.contains(&("added.txt", "A")));
    assert!(paths.contains(&("README.md", "M")));
    assert!(paths.contains(&("delete.txt", "D")));
    subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child.id, &execution)
        .await
        .expect("remove worktree");
    assert!(!worktree.exists());

    let applied = subagent_git_actions::apply(repo.path(), &parent.id, &child.id, &change.id)
        .await
        .expect("apply");
    assert_eq!(applied.status, super::types_subagent_change::SubagentChangeStatus::Applied);
    assert_eq!(std::fs::read_to_string(repo.path().join("added.txt")).unwrap(), "from child\n");
    assert!(!branch_exists(repo.path(), &change.branch));
    cleanup(&parent.id, &child.id).await;
}

#[tokio::test]
async fn conflict_aborts_without_changing_parent_head_or_files() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let (parent, child, execution) = sessions().await;
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    tokio::fs::write(worktree.join("README.md"), "child\n")
        .await
        .expect("child edit");
    let change = subagent_git_run::capture(repo.path(), &child.id, &execution, &worktree)
        .await
        .expect("capture")
        .expect("change");
    subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child.id, &execution)
        .await
        .expect("remove");
    std::fs::write(repo.path().join("README.md"), "parent\n").expect("parent edit");
    commit_all(repo.path(), "parent change");
    let head = git_text(repo.path(), &["rev-parse", "HEAD"]);

    assert!(subagent_git_actions::apply(repo.path(), &parent.id, &child.id, &change.id)
        .await
        .is_err());
    assert_eq!(git_text(repo.path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(std::fs::read_to_string(repo.path().join("README.md")).unwrap(), "parent\n");
    assert!(git_text(repo.path(), &["status", "--porcelain"]).is_empty());
    cleanup(&parent.id, &child.id).await;
}

#[tokio::test]
async fn correction_replays_pending_change_from_new_parent_head() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let (parent, child, first_execution) = sessions().await;
    let first = subagent_worktree::create_for_execution(repo.path(), &child.id, &first_execution)
        .await
        .expect("first worktree");
    tokio::fs::write(first.join("child.txt"), "first\n").await.expect("write first");
    let old = subagent_git_run::capture(repo.path(), &child.id, &first_execution, &first)
        .await.expect("capture first").expect("change");
    subagent_worktree::remove_owned(&first.to_string_lossy(), &child.id, &first_execution)
        .await.expect("remove first");
    std::fs::write(repo.path().join("parent.txt"), "advanced\n").expect("advance parent");
    commit_all(repo.path(), "advance parent");

    let next_execution = uuid::Uuid::new_v4().to_string();
    let next = subagent_worktree::create_for_execution(repo.path(), &child.id, &next_execution)
        .await.expect("next worktree");
    subagent_git_run::seed_pending(repo.path(), &child.id, &next_execution, &next)
        .await.expect("replay pending");
    assert_eq!(std::fs::read_to_string(next.join("child.txt")).unwrap(), "first\n");
    assert_eq!(std::fs::read_to_string(next.join("parent.txt")).unwrap(), "advanced\n");
    let migrated = super::subagent_change_store::load(&child.id).await.expect("migrated");
    assert_eq!(migrated.id, old.id);
    assert_ne!(migrated.branch, old.branch);
    assert!(!branch_exists(repo.path(), &old.branch));
    subagent_worktree::remove_owned(&next.to_string_lossy(), &child.id, &next_execution)
        .await.expect("remove next");
    subagent_git_actions::discard(repo.path(), &parent.id, &child.id, &migrated.id)
        .await.expect("discard");
    cleanup(&parent.id, &child.id).await;
}

#[tokio::test]
async fn discard_keeps_parent_unchanged_and_deletes_branch() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let (parent, child, execution) = sessions().await;
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    tokio::fs::write(worktree.join("discarded.txt"), "discard\n")
        .await
        .expect("write");
    let change = subagent_git_run::capture(repo.path(), &child.id, &execution, &worktree)
        .await
        .expect("capture")
        .expect("change");
    subagent_worktree::remove_owned(&worktree.to_string_lossy(), &child.id, &execution)
        .await
        .expect("remove");

    subagent_git_actions::discard(repo.path(), &parent.id, &child.id, &change.id)
        .await
        .expect("discard");
    assert!(!repo.path().join("discarded.txt").exists());
    assert!(!branch_exists(repo.path(), &change.branch));
    cleanup(&parent.id, &child.id).await;
}

async fn sessions() -> (super::types_session::AgentSession, super::types_session::AgentSession, String) {
    let parent = session_store::create_full("Parent", "model", "provider", false, Some("project".into()))
        .await
        .expect("parent");
    let mut child = session_store::create_full("Child", "model", "provider", false, Some("project".into()))
        .await
        .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    session_store::save(&child).await.expect("save child");
    (parent, child, uuid::Uuid::new_v4().to_string())
}

fn branch_exists(repo: &std::path::Path, branch: &str) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["show-ref", "--verify", "--quiet"])
        .arg(format!("refs/heads/{branch}"))
        .status()
        .expect("git")
        .success()
}

fn commit_all(repo: &std::path::Path, message: &str) {
    assert!(Command::new("git").args(["-C"]).arg(repo).args(["add", "-A"]).status().unwrap().success());
    assert!(Command::new("git")
        .args(["-C"]).arg(repo)
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-m", message])
        .status().unwrap().success());
}

fn git_text(repo: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git").args(["-C"]).arg(repo).args(args).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

async fn cleanup(parent_id: &str, child_id: &str) {
    let _ = super::subagent_change_store::remove(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
