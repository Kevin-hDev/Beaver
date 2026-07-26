use super::cleanup_orphans_in_dir;
use crate::services::agent_local::session_index;
use crate::services::agent_local::subagent_status;
use crate::services::agent_local::types_session::AgentSession;
use chrono::{Duration, Utc};
use tempfile::TempDir;
use uuid::Uuid;

fn session(id: &str, status: &str, parent: bool, offset_secs: i64) -> AgentSession {
    let created_at = Utc::now() + Duration::seconds(offset_secs);
    AgentSession {
        id: id.to_string(),
        name: id.to_string(),
        created_at,
        updated_at: Some(created_at),
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
