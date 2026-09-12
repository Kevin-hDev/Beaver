use super::{cleanup_orphans_in_dir, orphan_candidates};
use crate::services::agent_local::session_index;
use crate::services::agent_local::subagent_status;
use crate::services::agent_local::types_session::AgentSession;
use chrono::{Duration, Utc};
use tempfile::TempDir;
use uuid::Uuid;

fn session(id: &str, status: &str, parent: bool, offset_secs: i64) -> AgentSession {
    let created_at = Utc::now() + Duration::seconds(offset_secs);
    AgentSession {
        schema_version:
            crate::services::agent_local::session_limits::CURRENT_SESSION_SCHEMA_VERSION,
        id: id.to_string(),
        name: id.to_string(),
        created_at,
        updated_at: Some(created_at),
        archived_at: None,
        pinned_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        fast_mode_enabled: false,
        reasoning_mode: None,
        preserve_reasoning: Default::default(),
        accumulated_tokens: 0,
        context_tokens: None,
        context_usage: Default::default(),
        compression_profile_selection: None,
        compression_count: 0,
        automatic_compression_guard: Default::default(),
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
        parent_session_id: parent.then(|| Uuid::new_v4().to_string()),
        subagent_type: Some("coder".into()),
        subagent_worktree: None,
        subagent_prompt: None,
        subagent_status: Some(status.to_string()),
        subagent_run_id: Some("run-1".into()),
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
        clone_read_files: vec![],
        clone_modified_files: vec![],
        clone_root_session_id: None,
        git_branch: None,
    }
}

async fn write_session(dir: &TempDir, session: &AgentSession) {
    let path = dir.path().join(format!("{}.json", session.id));
    let data = serde_json::to_string_pretty(session).expect("serialize");
    tokio::fs::write(path, data).await.expect("write session");
}

async fn read_session(dir: &TempDir, id: &str) -> AgentSession {
    let data = tokio::fs::read_to_string(dir.path().join(format!("{id}.json")))
        .await
        .expect("read session");
    serde_json::from_str(&data).expect("parse session")
}

#[tokio::test]
async fn cleanup_reclassifies_old_running_orphans() {
    let dir = TempDir::new().expect("tempdir");
    let cutoff = Utc::now();
    let orphan = session(
        "11111111-1111-1111-1111-111111111111",
        subagent_status::RUNNING,
        true,
        -5,
    );
    let done = session(
        "22222222-2222-2222-2222-222222222222",
        subagent_status::COMPLETED,
        true,
        -5,
    );
    write_session(&dir, &orphan).await;
    write_session(&dir, &done).await;

    let cleaned = cleanup_orphans_in_dir(dir.path(), cutoff, false)
        .await
        .expect("cleanup");

    assert_eq!(cleaned, 1);
    let after_orphan = read_session(&dir, &orphan.id).await;
    let after_done = read_session(&dir, &done.id).await;
    assert_eq!(
        after_orphan.subagent_status.as_deref(),
        Some(subagent_status::INTERRUPTED)
    );
    assert_eq!(
        after_done.subagent_status.as_deref(),
        Some(subagent_status::COMPLETED)
    );
}

#[tokio::test]
async fn cleanup_ignores_running_subagents_newer_than_cutoff() {
    let dir = TempDir::new().expect("tempdir");
    let cutoff = Utc::now();
    let recent = session(
        "33333333-3333-3333-3333-333333333333",
        subagent_status::RUNNING,
        true,
        5,
    );
    write_session(&dir, &recent).await;

    let cleaned = cleanup_orphans_in_dir(dir.path(), cutoff, false)
        .await
        .expect("cleanup");

    assert_eq!(cleaned, 0);
    let after = read_session(&dir, &recent.id).await;
    assert_eq!(
        after.subagent_status.as_deref(),
        Some(subagent_status::RUNNING)
    );
}

#[tokio::test]
async fn cleanup_uses_rebuilt_index_when_sidecar_is_stale() {
    let dir = TempDir::new().expect("tempdir");
    let cutoff = Utc::now();
    let orphan = session(
        "44444444-4444-4444-4444-444444444444",
        subagent_status::RUNNING,
        true,
        -5,
    );
    write_session(&dir, &orphan).await;

    let mut stale_meta = session_index::meta_from_session(&orphan);
    stale_meta.subagent_status = Some(subagent_status::COMPLETED.into());
    session_index::write_index_to(dir.path(), &[stale_meta])
        .await
        .expect("write stale index");

    let cleaned = cleanup_orphans_in_dir(dir.path(), cutoff, false)
        .await
        .expect("cleanup");

    assert_eq!(cleaned, 1);
    let after = read_session(&dir, &orphan.id).await;
    assert_eq!(
        after.subagent_status.as_deref(),
        Some(subagent_status::INTERRUPTED)
    );
}

#[cfg(windows)]
#[test]
fn startup_git_prune_uses_the_background_command_boundary() {
    let source = include_str!("subagent_startup_cleanup.rs");
    assert!(
        source.contains("background_command::new_tokio(\"git\")"),
        "startup Git prune must not create a Windows console"
    );
}

#[test]
fn cleanup_uses_the_session_store_and_propagates_index_rebuild_failure() {
    let source = include_str!("subagent_startup_cleanup.rs");

    assert!(source.contains("session_store::read_from_dir"));
    assert!(source.contains("session_store::write_to_dir"));
    assert!(!source.contains("let _ = session_index::rebuild_index_from"));
}

#[test]
fn second_pass_is_bounded_and_excludes_non_candidates() {
    let cutoff = Utc::now();
    let completed = session("completed", subagent_status::COMPLETED, true, -5);
    let mut metas = vec![session_index::meta_from_session(&completed)];
    for index in 0..=crate::services::agent_local::session_limits::MAX_SESSION_FILES {
        let candidate = session(
            &format!("candidate-{index}"),
            subagent_status::INTERRUPTED,
            true,
            -5,
        );
        metas.push(session_index::meta_from_session(&candidate));
    }

    let selected = orphan_candidates(&metas, cutoff)
        .map(|meta| meta.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        selected.len(),
        crate::services::agent_local::session_limits::MAX_SESSION_FILES
    );
    assert!(!selected.contains(&completed.id.as_str()));
}

#[tokio::test]
async fn failed_capture_is_retried_and_cleared_on_next_startup() {
    use crate::services::agent_local::{
        session_store, subagent_change_store, subagent_git_command, subagent_task_change,
        subagent_worktree,
    };

    let dir = TempDir::new().expect("session dir");
    let repo = super::super::subagent_worktree_ownership_tests::init_repo_with_commit().await;
    let parent = session_store::create_full("Recovery parent", "model", "provider", false, None)
        .await
        .expect("parent");
    let mut child = session_store::create_full(
        "Recovery coder",
        "model",
        "provider",
        false,
        Some("project".into()),
    )
    .await
    .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("coder".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.working_dir = repo.path().to_string_lossy().into_owned();
    child.updated_at = Some(Utc::now() - Duration::seconds(5));
    let execution = Uuid::new_v4().to_string();
    let worktree = subagent_worktree::create_for_execution(repo.path(), &child.id, &execution)
        .await
        .expect("worktree");
    child.subagent_worktree = Some(worktree.to_string_lossy().into_owned());
    session_store::save(&child).await.expect("save global child");
    write_session(&dir, &child).await;
    tokio::fs::write(worktree.join("recovered.txt"), "recover\n")
        .await
        .expect("recovered change");
    subagent_task_change::fail_next_capture_for_child(&child.id).await;

    cleanup_orphans_in_dir(dir.path(), Utc::now(), true)
        .await
        .expect("first cleanup");
    let interrupted = read_session(&dir, &child.id).await;
    assert_eq!(
        interrupted.subagent_status.as_deref(),
        Some(subagent_status::INTERRUPTED)
    );
    assert!(interrupted.subagent_worktree.is_some());
    assert!(worktree.is_dir());

    cleanup_orphans_in_dir(dir.path(), Utc::now(), true)
        .await
        .expect("retry cleanup");
    let recovered = read_session(&dir, &child.id).await;
    assert_eq!(recovered.subagent_worktree, None);
    assert!(!worktree.exists());
    let change = subagent_change_store::load(&child.id)
        .await
        .expect("durable change");
    let commits = subagent_git_command::text(
        repo.path(),
        &["rev-list", "--count", &format!("{}..{}", change.base_commit, change.branch)],
    )
    .await
    .expect("captured commits");
    assert_eq!(commits, "1");

    let _ = subagent_git_command::delete_branch(repo.path(), &change.branch).await;
    let _ = subagent_change_store::remove(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}

#[tokio::test]
async fn missing_worktree_path_is_cleared_idempotently() {
    let dir = TempDir::new().expect("session dir");
    let mut orphan = session(
        "55555555-5555-4555-8555-555555555555",
        subagent_status::INTERRUPTED,
        true,
        -5,
    );
    let execution = Uuid::new_v4().to_string();
    orphan.subagent_worktree = Some(
        crate::services::agent_local::subagent_worktree::path_for_execution(
            &orphan.id,
            &execution,
        )
        .expect("managed path")
        .to_string_lossy()
        .into_owned(),
    );
    write_session(&dir, &orphan).await;

    cleanup_orphans_in_dir(dir.path(), Utc::now(), true)
        .await
        .expect("idempotent cleanup");

    assert_eq!(read_session(&dir, &orphan.id).await.subagent_worktree, None);
}
