use super::*;
use chrono::Utc;
use std::path::Path;
use tempfile::TempDir;

fn test_session(id: &str, name: &str, heartbeat: bool) -> AgentSession {
    AgentSession {
        id: id.into(),
        name: name.into(),
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
        is_heartbeat: heartbeat,
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
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_summary: None,
        clone_read_files: Vec::new(),
        clone_modified_files: Vec::new(),
        clone_root_session_id: None,
        git_branch: None,
    }
}

fn test_meta(id: &str, count: usize) -> AgentSessionMeta {
    AgentSessionMeta {
        id: id.into(),
        name: id.into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        reasoning_mode: None,
        message_count: count,
        is_heartbeat: false,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        parent_session_id: None,
        subagent_type: None,
        subagent_status: None,
        subagent_run_id: None,
        subagent_description: None,
        subagent_color_key: None,
        subagent_summary: None,
        subagent_last_activity: None,
        clone_parent_session_id: None,
        clone_parent_message_id: None,
        clone_mode: None,
        clone_root_session_id: None,
        git_branch: None,
    }
}

async fn persist(dir: &Path, s: &AgentSession) {
    let data = serde_json::to_string_pretty(s).unwrap();
    tokio::fs::write(dir.join(format!("{}.json", s.id)), &data)
        .await
        .unwrap();
}

async fn load_index(dir: &Path) -> Vec<AgentSessionMeta> {
    let data = tokio::fs::read_to_string(dir.join("index.json"))
        .await
        .unwrap();
    serde_json::from_str(&data).unwrap()
}

#[tokio::test]
async fn rebuild_produces_correct_index() {
    let tmp = TempDir::new().unwrap();
    persist(tmp.path(), &test_session("abc-123", "Test", false)).await;

    let entries = rebuild_index_from(tmp.path()).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "abc-123");
    assert_eq!(entries[0].name, "Test");
    assert_eq!(entries[0].message_count, 0);

    let saved = load_index(tmp.path()).await;
    assert_eq!(saved.len(), 1);
}

#[tokio::test]
async fn rebuild_skips_index_json() {
    let tmp = TempDir::new().unwrap();
    persist(tmp.path(), &test_session("real", "Real", false)).await;
    tokio::fs::write(tmp.path().join("index.json"), "[]")
        .await
        .unwrap();

    let entries = rebuild_index_from(tmp.path()).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "real");
}

#[tokio::test]
async fn rebuild_skips_non_json_and_corrupt() {
    let tmp = TempDir::new().unwrap();
    persist(tmp.path(), &test_session("good", "Good", false)).await;
    tokio::fs::write(tmp.path().join("notes.txt"), "text")
        .await
        .unwrap();
    tokio::fs::write(tmp.path().join("corrupt.json"), "{broken")
        .await
        .unwrap();

    let entries = rebuild_index_from(tmp.path()).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "good");
}

#[tokio::test]
async fn upsert_and_remove_via_write() {
    let tmp = TempDir::new().unwrap();
    write_index_to(tmp.path(), &[test_meta("t1", 5)])
        .await
        .unwrap();
    assert_eq!(load_index(tmp.path()).await[0].message_count, 5);

    let updated = AgentSessionMeta {
        message_count: 10,
        ..test_meta("t1", 0)
    };
    write_index_to(tmp.path(), &[updated]).await.unwrap();
    assert_eq!(load_index(tmp.path()).await[0].message_count, 10);

    write_index_to(tmp.path(), &[]).await.unwrap();
    assert!(load_index(tmp.path()).await.is_empty());
}

#[tokio::test]
async fn update_count_via_write() {
    let tmp = TempDir::new().unwrap();
    write_index_to(tmp.path(), &[test_meta("ct", 0)])
        .await
        .unwrap();

    let mut entries = load_index(tmp.path()).await;
    entries[0].message_count = 42;
    write_index_to(tmp.path(), &entries).await.unwrap();

    assert_eq!(load_index(tmp.path()).await[0].message_count, 42);
}

#[tokio::test]
async fn rebuild_empty_and_nonexistent() {
    let tmp = TempDir::new().unwrap();
    assert!(rebuild_index_from(tmp.path()).await.unwrap().is_empty());
    assert!(rebuild_index_from(Path::new("/tmp/nonexistent-cl-go"))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn meta_from_session_extracts_all_fields() {
    let mut s = test_session("mf", "Meta", true);
    let now = Utc::now();
    s.updated_at = Some(now);
    s.archived_at = Some(now);
    s.project_id = Some("p1".into());
    s.subagent_type = Some("worker".into());
    s.subagent_status = Some("running".into());
    s.subagent_run_id = Some("r1".into());
    s.subagent_description = Some("Analyse".into());
    s.subagent_color_key = Some("geminitor".into());
    s.subagent_summary = Some("Résumé".into());
    s.clone_parent_session_id = Some("parent".into());
    s.clone_parent_message_id = Some("msg".into());
    s.clone_mode = Some(crate::services::agent_local::types_session::CloneMode::Cut);
    s.clone_root_session_id = Some("root".into());
    s.git_branch = Some("clone-11111111".into());

    let meta = meta_from_session(&s);
    assert_eq!(meta.id, "mf");
    assert!(meta.is_heartbeat);
    assert_eq!(meta.updated_at, Some(now));
    assert_eq!(meta.archived_at, Some(now));
    assert_eq!(meta.project_id, Some("p1".into()));
    assert_eq!(meta.subagent_type, Some("worker".into()));
    assert_eq!(meta.subagent_status, Some("running".into()));
    assert_eq!(meta.subagent_run_id, Some("r1".into()));
    assert_eq!(meta.subagent_description, Some("Analyse".into()));
    assert_eq!(meta.subagent_color_key, Some("geminitor".into()));
    assert_eq!(meta.subagent_summary, Some("Résumé".into()));
    assert_eq!(meta.clone_parent_session_id, Some("parent".into()));
    assert_eq!(meta.clone_parent_message_id, Some("msg".into()));
    assert_eq!(
        meta.clone_mode,
        Some(crate::services::agent_local::types_session::CloneMode::Cut)
    );
    assert_eq!(meta.clone_root_session_id, Some("root".into()));
    assert_eq!(meta.git_branch, Some("clone-11111111".into()));
}

#[tokio::test]
async fn rebuild_multiple_sessions() {
    let tmp = TempDir::new().unwrap();
    for i in 0..5u8 {
        persist(
            tmp.path(),
            &test_session(&format!("s{i}"), &format!("S{i}"), i % 2 == 0),
        )
        .await;
    }
    let entries = rebuild_index_from(tmp.path()).await.unwrap();
    assert_eq!(entries.len(), 5);
    assert_eq!(entries.iter().filter(|e| e.is_heartbeat).count(), 3);
}

#[tokio::test]
async fn corrupt_index_triggers_rebuild() {
    let tmp = TempDir::new().unwrap();
    persist(tmp.path(), &test_session("s1", "S1", false)).await;
    rebuild_index_from(tmp.path()).await.unwrap();
    tokio::fs::write(tmp.path().join("index.json"), "NOT_JSON")
        .await
        .unwrap();

    // read_index uses the global path, so we test rebuild_index_from + read pattern
    let data = tokio::fs::read_to_string(tmp.path().join("index.json"))
        .await
        .unwrap();
    let result = serde_json::from_str::<Vec<AgentSessionMeta>>(&data);
    assert!(result.is_err());

    let entries = rebuild_index_from(tmp.path()).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, "s1");
}
