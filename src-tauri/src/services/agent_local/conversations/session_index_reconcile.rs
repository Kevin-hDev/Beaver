use super::session_store::validate_session_id;
use super::types_session::{AgentSession, AgentSessionMeta};
use std::path::Path;

pub(super) async fn reconcile(
    index_path: &Path,
    entries: Vec<AgentSessionMeta>,
) -> Result<Vec<AgentSessionMeta>, String> {
    let Some(dir) = index_path.parent() else {
        return Ok(entries);
    };
    for meta in &entries {
        if validate_session_id(&meta.id).is_err() {
            return super::session_index::rebuild_index_from(dir).await;
        }
        let path = dir.join(format!("{}.json", meta.id));
        #[cfg(test)]
        super::session_index::test_support::record_document_read();
        let Ok(session) = super::session_store_document::read_from_path(path).await else {
            return super::session_index::rebuild_index_from(dir).await;
        };
        if meta_drifted(meta, &session) {
            return super::session_index::rebuild_index_from(dir).await;
        }
    }
    Ok(entries)
}

pub(super) fn meta_drifted(meta: &AgentSessionMeta, session: &AgentSession) -> bool {
    let expected = super::session_index_meta::from_session(session);
    meta.archived_at != session.archived_at
        || meta.fast_mode_enabled != session.fast_mode_enabled
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
        || meta.has_active_context_request != expected.has_active_context_request
}
