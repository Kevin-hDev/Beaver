use super::session_store::validate_session_id;
use super::session_security;
use super::session_index_io::{
    index_fingerprint, index_path, read_index_raw, write_index, IndexFingerprint,
};
pub(crate) use super::session_index_io::write_index_to;
use crate::services::agent_local::types_session::{AgentSession, AgentSessionMeta};
use std::path::Path;
use tokio::sync::Mutex;

static INDEX_LOCK: Mutex<()> = Mutex::const_new(());
static INDEX_RECONCILE_FINGERPRINT: Mutex<Option<IndexFingerprint>> = Mutex::const_new(None);

pub async fn read_index() -> Result<Vec<AgentSessionMeta>, String> {
    let mut last_fingerprint = INDEX_RECONCILE_FINGERPRINT.lock().await;
    let path = index_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => match serde_json::from_str::<Vec<AgentSessionMeta>>(&data) {
            Ok(entries) => {
                let fingerprint = index_fingerprint(&path).await;
                if last_fingerprint.as_ref() == fingerprint.as_ref() {
                    Ok(entries)
                } else {
                    let entries = reconcile_index(&path, entries).await?;
                    *last_fingerprint = index_fingerprint(&path).await;
                    Ok(entries)
                }
            }
            Err(_) => {
                let entries = rebuild_index().await?;
                *last_fingerprint = index_fingerprint(&path).await;
                Ok(entries)
            }
        },
        Err(_) => {
            let entries = rebuild_index().await?;
            *last_fingerprint = index_fingerprint(&path).await;
            Ok(entries)
        }
    }
}

async fn reconcile_index(
    index_path: &Path,
    entries: Vec<AgentSessionMeta>,
) -> Result<Vec<AgentSessionMeta>, String> {
    let Some(dir) = index_path.parent() else {
        return Ok(entries);
    };
    for meta in &entries {
        if validate_session_id(&meta.id).is_err() {
            return rebuild_index_from(dir).await;
        }
        let path = dir.join(format!("{}.json", meta.id));
        let Ok(data) = tokio::fs::read_to_string(&path).await else {
            return rebuild_index_from(dir).await;
        };
        let Ok(session) = serde_json::from_str::<AgentSession>(&data) else {
            continue;
        };
        if index_meta_drifted(meta, &session) {
            return rebuild_index_from(dir).await;
        }
    }
    Ok(entries)
}

fn index_meta_drifted(meta: &AgentSessionMeta, session: &AgentSession) -> bool {
    let expected = meta_from_session(session);
    meta.archived_at != session.archived_at
        || meta.parent_session_id != session.parent_session_id
        || meta.subagent_type != session.subagent_type
        || meta.subagent_status != session.subagent_status
        || meta.subagent_run_id != session.subagent_run_id
        || meta.subagent_description != expected.subagent_description
        || meta.subagent_color_key != session.subagent_color_key
        || meta.subagent_summary != expected.subagent_summary
        || meta.subagent_last_activity != expected.subagent_last_activity
        || meta.clone_parent_session_id != session.clone_parent_session_id
        || meta.clone_parent_message_id != session.clone_parent_message_id
        || meta.clone_mode != session.clone_mode
        || meta.clone_root_session_id != session.clone_root_session_id
        || meta.git_branch != session.git_branch
}

pub async fn rebuild_index() -> Result<Vec<AgentSessionMeta>, String> {
    let dir = crate::services::paths::data_dir().join("agent-sessions");
    rebuild_index_from(&dir).await
}

pub async fn rebuild_index_from(dir: &Path) -> Result<Vec<AgentSessionMeta>, String> {
    let mut entries = Vec::new();
    if !dir.exists() {
        return Ok(entries);
    }
    let mut read_dir = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("index.json") {
            continue;
        }
        if let Ok(data) = tokio::fs::read_to_string(&path).await {
            if let Ok(session) = serde_json::from_str::<AgentSession>(&data) {
                entries.push(meta_from_session(&session));
            }
        }
    }
    write_index_to(dir, &entries).await?;
    Ok(entries)
}

pub async fn upsert_entry(meta: AgentSessionMeta) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index_raw().await;
    if let Some(pos) = entries.iter().position(|e| e.id == meta.id) {
        entries[pos] = meta;
    } else {
        entries.push(meta);
    }
    write_index(&entries).await?;
    refresh_reconcile_fingerprint().await;
    Ok(())
}

pub async fn remove_entry(id: &str) -> Result<(), String> {
    let _guard = INDEX_LOCK.lock().await;
    let mut entries = read_index_raw().await;
    entries.retain(|e| e.id != id);
    write_index(&entries).await?;
    refresh_reconcile_fingerprint().await;
    Ok(())
}

pub fn meta_from_session(session: &AgentSession) -> AgentSessionMeta {
    AgentSessionMeta {
        id: session.id.clone(),
        name: crate::services::agent_local::sensitive_data::redact_high_confidence_text(
            &session.name,
        ),
        created_at: session.created_at,
        updated_at: session.updated_at,
        archived_at: session.archived_at,
        model: session.model.clone(),
        provider: session.provider.clone(),
        thinking_enabled: session.thinking_enabled,
        reasoning_mode: session.reasoning_mode.clone(),
        message_count: session.messages.len(),
        is_heartbeat: session.is_heartbeat,
        is_gateway: session.is_gateway,
        gateway_channel_key: session_security::redacted_optional(&session.gateway_channel_key),
        project_id: session.project_id.clone(),
        parent_session_id: session.parent_session_id.clone(),
        subagent_type: session.subagent_type.clone(),
        subagent_status: session.subagent_status.clone(),
        subagent_run_id: session.subagent_run_id.clone(),
        subagent_description: session_security::redacted_optional(&session.subagent_description),
        subagent_color_key: session.subagent_color_key.clone(),
        subagent_summary: session_security::redacted_optional(&session.subagent_summary),
        subagent_last_activity: session_security::redacted_activity(&session.subagent_last_activity),
        clone_parent_session_id: session.clone_parent_session_id.clone(),
        clone_parent_message_id: session.clone_parent_message_id.clone(),
        clone_mode: session.clone_mode.clone(),
        clone_root_session_id: session.clone_root_session_id.clone(),
        git_branch: session.git_branch.clone(),
    }
}

async fn refresh_reconcile_fingerprint() {
    let mut last_fingerprint = INDEX_RECONCILE_FINGERPRINT.lock().await;
    *last_fingerprint = index_fingerprint(&index_path()).await;
}

#[path = "session_index_tests.rs"]
#[cfg(test)]
mod tests;

#[path = "session_index_reconcile_tests.rs"]
#[cfg(test)]
mod reconcile_tests;
