use super::{session_store, subagent_working_dir, subagent_worktree};

#[tokio::test]
async fn corrupted_session_never_removes_another_child_worktree() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let mut child_a = session("Owner A").await;
    let child_b = session("Owner B").await;
    let execution_b = id();
    let target_b = subagent_worktree::create_for_execution(repo.path(), &child_b.id, &execution_b)
        .await
        .expect("create B worktree");
    let target_b_text = target_b.to_string_lossy().to_string();
    child_a.subagent_worktree = Some(target_b_text.clone());
    session_store::save(&child_a).await.expect("corrupt A ownership");

    subagent_working_dir::cleanup_owned(&child_a.id, &execution_b, Some(&target_b_text)).await;

    assert!(target_b.exists(), "A ne doit jamais supprimer le worktree de B");
    let _ = subagent_worktree::remove(&target_b_text).await;
    delete_sessions(&[&child_a.id, &child_b.id]).await;
}

#[tokio::test]
async fn managed_subdirectory_is_never_accepted_as_a_worktree() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let target = subagent_worktree::create_for_execution(repo.path(), &id(), &id())
        .await
        .expect("create worktree");
    let nested = target.join("nested");
    tokio::fs::create_dir_all(&nested)
        .await
        .expect("create nested directory");

    let result = subagent_worktree::remove(&nested.to_string_lossy()).await;

    assert!(result.is_err(), "un sous-dossier doit être refusé");
    assert!(nested.exists(), "le sous-dossier doit rester intact");
    let _ = subagent_worktree::remove(&target.to_string_lossy()).await;
}

#[tokio::test]
async fn wrong_expected_execution_never_removes_the_real_worktree() {
    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let child_id = id();
    let real_execution = id();
    let target = subagent_worktree::create_for_execution(repo.path(), &child_id, &real_execution)
        .await
        .expect("create real worktree");

    let result = subagent_worktree::remove_owned(
        &target.to_string_lossy(),
        &child_id,
        &id(),
    )
    .await;

    assert!(result.is_err(), "une autre exécution doit être refusée");
    assert!(target.exists(), "le worktree réel doit rester intact");
    let _ = subagent_worktree::remove(&target.to_string_lossy()).await;
}

#[tokio::test]
async fn non_uuid_managed_components_are_rejected_without_mutation() {
    let root = crate::services::paths::data_dir().join("subagent-worktrees");
    let invalid_child = root.join(format!("invalid-{}", id())).join(id());
    let invalid_execution = root.join(id()).join(format!("invalid-{}", id()));
    for target in [&invalid_child, &invalid_execution] {
        tokio::fs::create_dir_all(target)
            .await
            .expect("create invalid managed path");
        tokio::fs::write(target.join("keep.txt"), "keep")
            .await
            .expect("write marker");

        let result = subagent_worktree::remove(&target.to_string_lossy()).await;

        assert!(result.is_err(), "un composant non-UUID doit être refusé");
        assert!(target.join("keep.txt").exists(), "la cible doit rester intacte");
        let child_dir = target.parent().expect("invalid child directory");
        tokio::fs::remove_dir_all(child_dir)
            .await
            .expect("cleanup invalid path");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn managed_symlink_never_targets_another_worktree() {
    use std::os::unix::fs::symlink;

    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let target_b = subagent_worktree::create_for_execution(repo.path(), &id(), &id())
        .await
        .expect("create target B");
    let link = subagent_worktree::path_for_execution(&id(), &id()).expect("managed link path");
    tokio::fs::create_dir_all(link.parent().expect("link parent"))
        .await
        .expect("create link parent");
    symlink(&target_b, &link).expect("create managed symlink");

    let result = subagent_worktree::remove(&link.to_string_lossy()).await;

    assert!(result.is_err(), "un lien symbolique doit être refusé");
    assert!(target_b.exists(), "la cible d'un autre run doit rester intacte");
    let _ = tokio::fs::remove_file(&link).await;
    let _ = tokio::fs::remove_dir(link.parent().expect("link parent")).await;
    let _ = subagent_worktree::remove(&target_b.to_string_lossy()).await;
}

#[cfg(unix)]
#[tokio::test]
async fn managed_child_symlink_never_targets_another_child() {
    use std::os::unix::fs::symlink;

    let repo = super::subagent_worktree_ownership_tests::init_repo_with_commit();
    let execution_b = id();
    let target_b = subagent_worktree::create_for_execution(repo.path(), &id(), &execution_b)
        .await
        .expect("create target B");
    let linked_child = target_b
        .parent()
        .and_then(std::path::Path::parent)
        .expect("managed root")
        .join(id());
    symlink(target_b.parent().expect("target B child"), &linked_child)
        .expect("create child symlink");
    let linked_execution = linked_child.join(&execution_b);

    let result = subagent_worktree::remove(&linked_execution.to_string_lossy()).await;

    assert!(result.is_err(), "un dossier enfant symbolique doit être refusé");
    assert!(target_b.exists(), "le worktree de l'autre enfant doit rester intact");
    let _ = tokio::fs::remove_file(&linked_child).await;
    let _ = subagent_worktree::remove(&target_b.to_string_lossy()).await;
}

async fn session(name: &str) -> super::types_session::AgentSession {
    session_store::create_full(name, "llama3", "ollama", false, None)
        .await
        .expect("create session")
}

async fn delete_sessions(ids: &[&str]) {
    for id in ids {
        session_store::delete_one(id).await.expect("delete session");
    }
}

fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
