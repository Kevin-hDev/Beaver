use super::{session_store, subagent_registry, subagent_working_dir, subagent_worktree};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn prepared_dir_survives_internal_turn_and_new_execution_is_distinct() {
    let repo = init_repo_with_commit();
    let parent = session("Parent").await;
    let mut child = child_session(&parent.id, "Coder").await;
    let first_run = register(&parent.id, &child.id).await;
    save_run(&mut child, &first_run.run_id).await;
    let prepared = subagent_working_dir::create_coder_worktree_for_test(
        repo.path(),
        &child.id,
        &first_run.run_id,
        &first_run.execution_id,
    )
    .await
    .expect("prepare once");
    let uncommitted = prepared.path().join("uncommitted.txt");
    tokio::fs::write(&uncommitted, "must survive")
        .await
        .expect("write uncommitted file");

    let second_turn_dir = prepared.path();
    assert_eq!(tokio::fs::read_to_string(second_turn_dir.join("uncommitted.txt")).await.unwrap(), "must survive");

    subagent_registry::unregister(&child.id).await;
    let next_run = register(&parent.id, &child.id).await;
    save_run(&mut child, &next_run.run_id).await;
    let next = subagent_working_dir::create_coder_worktree_for_test(
        repo.path(),
        &child.id,
        &next_run.run_id,
        &next_run.execution_id,
    )
    .await
    .expect("new execution");
    assert_ne!(prepared.path(), next.path());

    subagent_registry::unregister(&child.id).await;
    subagent_working_dir::cleanup_owned(
        &child.id,
        &first_run.execution_id,
        prepared.worktree_path(),
    )
    .await;
    subagent_working_dir::cleanup_owned(&child.id, &next_run.execution_id, next.worktree_path())
        .await;
    delete_sessions(&[&child.id, &parent.id]).await;
}

#[tokio::test]
async fn preexisting_execution_target_is_rejected() {
    let repo = init_repo_with_commit();
    let child_id = uuid::Uuid::new_v4().to_string();
    let execution_id = uuid::Uuid::new_v4().to_string();
    let target = subagent_worktree::path_for_execution(&child_id, &execution_id).unwrap();
    tokio::fs::create_dir_all(&target).await.expect("create collision");
    tokio::fs::write(target.join("owner.txt"), "existing")
        .await
        .expect("write owner");

    let result = subagent_worktree::create_for_execution(repo.path(), &child_id, &execution_id).await;

    assert!(result.is_err());
    assert_eq!(tokio::fs::read_to_string(target.join("owner.txt")).await.unwrap(), "existing");
    tokio::fs::remove_dir_all(target).await.expect("cleanup collision");
}

#[tokio::test]
async fn old_cleanup_removes_only_old_path_after_new_execution_ended() {
    let repo = init_repo_with_commit();
    let parent = session("Parent cleanup").await;
    let mut child = child_session(&parent.id, "Coder cleanup").await;
    let sibling = child_session(&parent.id, "Sibling").await;
    let old = register(&parent.id, &child.id).await;
    let _sibling_run = register(&parent.id, &sibling.id).await;
    save_run(&mut child, &old.run_id).await;
    let old_dir = prepare(repo.path(), &child, &old).await;
    subagent_registry::unregister(&child.id).await;
    let new = register(&parent.id, &child.id).await;
    save_run(&mut child, &new.run_id).await;
    let new_dir = prepare(repo.path(), &child, &new).await;
    subagent_registry::unregister(&child.id).await;
    subagent_registry::unregister(&sibling.id).await;

    subagent_working_dir::cleanup_owned(&child.id, &old.execution_id, old_dir.worktree_path())
        .await;
    let saved = session_store::get(&child.id).await.expect("saved child");
    assert!(!old_dir.path().exists());
    assert!(new_dir.path().exists());
    assert_eq!(saved.subagent_worktree.as_deref(), new_dir.worktree_path());

    subagent_working_dir::cleanup_owned(&child.id, &new.execution_id, new_dir.worktree_path())
        .await;
    delete_sessions(&[&child.id, &sibling.id, &parent.id]).await;
}

#[tokio::test]
async fn missing_session_after_creation_never_leaks_the_worktree() {
    let repo = init_repo_with_commit();
    let parent = session("Parent missing session").await;
    let mut child = child_session(&parent.id, "Coder missing session").await;
    let run = register(&parent.id, &child.id).await;
    save_run(&mut child, &run.run_id).await;
    let target = subagent_worktree::path_for_execution(&child.id, &run.execution_id)
        .expect("execution target");
    session_store::delete_one(&child.id)
        .await
        .expect("delete child before preparation");

    let result = subagent_working_dir::create_coder_worktree_for_test(
        repo.path(),
        &child.id,
        &run.run_id,
        &run.execution_id,
    )
    .await;
    let leaked = target.exists();
    if leaked {
        let _ = subagent_worktree::remove(&target.to_string_lossy()).await;
    }
    subagent_registry::unregister(&child.id).await;
    delete_sessions(&[&parent.id]).await;

    assert!(result.is_err());
    assert!(!leaked);
}

async fn prepare(
    repo: &std::path::Path,
    child: &super::types_session::AgentSession,
    run: &subagent_registry::RegisteredSubagent,
) -> subagent_working_dir::PreparedWorkingDir {
    subagent_working_dir::create_coder_worktree_for_test(repo, &child.id, &run.run_id, &run.execution_id)
        .await
        .expect("prepare worktree")
}

async fn register(parent_id: &str, child_id: &str) -> subagent_registry::RegisteredSubagent {
    subagent_registry::register_execution(parent_id, child_id, CancellationToken::new())
        .await
        .expect("register execution")
}

async fn save_run(child: &mut super::types_session::AgentSession, run_id: &str) {
    child.subagent_run_id = Some(run_id.to_string());
    session_store::save(child).await.expect("save run");
}

async fn session(name: &str) -> super::types_session::AgentSession {
    session_store::create_full(name, "llama3", "ollama", false, None).await.unwrap()
}

async fn child_session(parent_id: &str, name: &str) -> super::types_session::AgentSession {
    let mut child = session(name).await;
    child.parent_session_id = Some(parent_id.into());
    child.subagent_type = Some("coder".into());
    child.subagent_status = Some(super::subagent_status::RUNNING.into());
    session_store::save(&child).await.expect("save child");
    child
}

async fn delete_sessions(ids: &[&str]) {
    for id in ids {
        session_store::delete_one(id).await.expect("delete session");
    }
}

pub(super) fn init_repo_with_commit() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp repo");
    let repo = git2::Repository::init(tmp.path()).expect("init repo");
    std::fs::write(tmp.path().join("README.md"), "init").expect("write file");
    let mut index = repo.index().expect("index");
    index.add_path(std::path::Path::new("README.md")).expect("add file");
    index.write().expect("write index");
    let tree = repo.find_tree(index.write_tree().expect("write tree")).expect("find tree");
    let signature = git2::Signature::now("CL-GO Test", "test@example.com").expect("signature");
    repo.commit(Some("HEAD"), &signature, &signature, "init", &tree, &[]).expect("commit");
    drop(tree);
    drop(repo);
    tmp
}
