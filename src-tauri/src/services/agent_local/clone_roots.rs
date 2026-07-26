use super::types_session::{AgentSession, AgentSessionMeta};
use std::collections::HashSet;

const MAX_CLONE_ANCESTORS: usize = 64;

pub async fn resolve_source_root_id(source: &AgentSession) -> Result<String, String> {
    let Some(parent_id) = source.clone_parent_session_id.as_deref() else {
        return Ok(source.id.clone());
    };
    let metas = super::session_index::read_index().await?;
    if !metas.iter().any(|meta| meta.id == source.id) {
        return Ok(source
            .clone_root_session_id
            .clone()
            .unwrap_or_else(|| parent_id.to_string()));
    }
    root_id_from_metas(&metas, &source.id, Some(parent_id))
}

pub fn root_id_from_metas(
    metas: &[AgentSessionMeta],
    session_id: &str,
    clone_parent_session_id: Option<&str>,
) -> Result<String, String> {
    let Some(mut parent_id) = clone_parent_session_id else {
        return Ok(session_id.to_string());
    };
    let mut seen = HashSet::new();
    for _ in 0..MAX_CLONE_ANCESTORS {
        if !seen.insert(parent_id.to_string()) {
            return Err("Action impossible".into());
        }
        let Some(parent) = metas.iter().find(|meta| meta.id == parent_id) else {
            return Err("Action impossible".into());
        };
        match parent.clone_parent_session_id.as_deref() {
            Some(next_parent_id) => parent_id = next_parent_id,
            None => return Ok(parent.id.clone()),
        }
    }
    Err("Action impossible".into())
}

pub fn git_branch_shared_with_ancestor(
    metas: &[AgentSessionMeta],
    _session_id: &str,
    clone_parent_session_id: Option<&str>,
    git_branch: &str,
) -> bool {
    let Some(mut parent_id) = clone_parent_session_id else {
        return false;
    };
    let mut seen = HashSet::new();
    for _ in 0..MAX_CLONE_ANCESTORS {
        if !seen.insert(parent_id.to_string()) {
            return false;
        }
        let Some(parent) = metas.iter().find(|meta| meta.id == parent_id) else {
            return false;
        };
        if parent.git_branch.as_deref() == Some(git_branch) {
            return true;
        }
        match parent.clone_parent_session_id.as_deref() {
            Some(next_parent_id) => parent_id = next_parent_id,
            None => return false,
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn meta(id: &str, clone_parent: Option<&str>, clone_root: Option<&str>) -> AgentSessionMeta {
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
            message_count: 0,
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
            clone_parent_session_id: clone_parent.map(str::to_string),
            clone_parent_message_id: None,
            clone_mode: None,
            clone_root_session_id: clone_root.map(str::to_string),
            git_branch: None,
        }
    }

    #[test]
    fn root_id_from_metas_walks_legacy_clone_chain() {
        let metas = vec![
            meta("root", None, None),
            meta("clone-a", Some("root"), None),
            meta("clone-b", Some("clone-a"), Some("clone-a")),
        ];

        let root = root_id_from_metas(&metas, "clone-b", Some("clone-a")).expect("root id");

        assert_eq!(root, "root");
    }

    #[test]
    fn git_branch_shared_with_ancestor_detects_inherited_branch() {
        let mut root = meta("root", None, None);
        root.git_branch = None;
        let mut clone_a = meta("clone-a", Some("root"), None);
        clone_a.git_branch = Some("clone-11111111".into());
        let mut clone_b = meta("clone-b", Some("clone-a"), Some("clone-a"));
        clone_b.git_branch = Some("clone-11111111".into());
        let metas = vec![root, clone_a, clone_b];

        assert!(git_branch_shared_with_ancestor(
            &metas,
            "clone-b",
            Some("clone-a"),
            "clone-11111111",
        ));
    }
}
