use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use chrono::Utc;
use std::cell::Cell;
use std::path::Path;

tokio::task_local! {
    static DOCUMENT_READS: Cell<usize>;
}

pub(crate) fn record_document_read() {
    let _ = DOCUMENT_READS.try_with(|count| count.set(count.get() + 1));
}

pub(super) async fn measure_rebuild_then_read(dir: &Path) -> Result<(usize, usize), String> {
    let _guard = super::INDEX_LOCK.lock().await;
    DOCUMENT_READS
        .scope(Cell::new(0), async {
            let revision =
                super::SESSION_SOURCE_REVISION.load(std::sync::atomic::Ordering::Acquire);
            super::rebuild_index_from(dir).await?;
            let rebuild_reads = DOCUMENT_READS.with(Cell::get);
            let path = dir.join("index.json");
            super::refresh_reconcile_state(&path, revision).await;
            super::read_index_once(&path, revision).await?;
            let repeated_reads = DOCUMENT_READS.with(Cell::get) - rebuild_reads;
            *super::INDEX_RECONCILE_FINGERPRINT.lock().await = None;
            super::INDEX_SOURCE_REVISION.store(u64::MAX, std::sync::atomic::Ordering::Release);
            Ok((rebuild_reads, repeated_reads))
        })
        .await
}

pub(super) fn test_session(id: &str, name: &str, heartbeat: bool) -> AgentSession {
    AgentSession {
        schema_version:
            crate::services::agent_local::session_limits::CURRENT_SESSION_SCHEMA_VERSION,
        id: id.into(),
        name: name.into(),
        created_at: Utc::now(),
        updated_at: None,
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
        is_heartbeat: heartbeat,
        is_gateway: false,
        gateway_channel_key: None,
        project_id: None,
        working_dir: String::new(),
        working_dir_managed: false,
        parent_session_id: None,
        subagent_extension_owner: None,
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

pub(super) fn test_meta(id: &str, count: usize) -> AgentSessionMeta {
    AgentSessionMeta {
        id: id.into(),
        name: id.into(),
        created_at: Utc::now(),
        updated_at: None,
        archived_at: None,
        pinned_at: None,
        model: "llama3".into(),
        provider: "ollama".into(),
        thinking_enabled: false,
        fast_mode_enabled: false,
        reasoning_mode: None,
        message_count: count,
        is_heartbeat: false,
        is_gateway: false,
        has_active_context_request: false,
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

pub(super) async fn persist(dir: &Path, session: &AgentSession) {
    let data = serde_json::to_string_pretty(session).unwrap();
    tokio::fs::write(dir.join(format!("{}.json", session.id)), &data)
        .await
        .unwrap();
}

pub(super) async fn load_index(dir: &Path) -> Vec<AgentSessionMeta> {
    let data = tokio::fs::read_to_string(dir.join("index.json"))
        .await
        .unwrap();
    serde_json::from_str(&data).unwrap()
}
