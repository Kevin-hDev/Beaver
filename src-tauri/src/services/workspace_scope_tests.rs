use super::agent_local::types_session::AgentSessionMeta;
use super::workspace_scope::{resolve_from_metas, WorkspaceScope};

fn session(id: &str) -> AgentSessionMeta {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "name": id,
        "created_at": "2026-09-01T00:00:00Z",
        "model": "test",
        "provider": "test",
        "message_count": 0
    }))
    .unwrap()
}

#[test]
fn project_sessions_share_one_scope() {
    let mut first = session("550e8400-e29b-41d4-a716-446655440001");
    let mut second = session("550e8400-e29b-41d4-a716-446655440002");
    first.project_id = Some("project-a".into());
    second.project_id = Some("project-a".into());
    let metas = [first.clone(), second.clone()];

    assert_eq!(
        resolve_from_metas(&first.id, &metas).unwrap(),
        WorkspaceScope::Project("project-a".into())
    );
    assert_eq!(
        resolve_from_metas(&second.id, &metas).unwrap(),
        WorkspaceScope::Project("project-a".into())
    );
}

#[test]
fn directoryless_descendants_inherit_the_root_discussion_scope() {
    let root = session("550e8400-e29b-41d4-a716-446655440001");
    let mut clone = session("550e8400-e29b-41d4-a716-446655440002");
    clone.clone_parent_session_id = Some(root.id.clone());
    clone.clone_root_session_id = Some(root.id.clone());
    let mut child = session("550e8400-e29b-41d4-a716-446655440003");
    child.parent_session_id = Some(clone.id.clone());
    let metas = [root.clone(), clone, child.clone()];

    assert_eq!(
        resolve_from_metas(&child.id, &metas).unwrap(),
        WorkspaceScope::Session(root.id)
    );
}

#[test]
fn corrupt_ancestry_fails_closed() {
    let mut orphan = session("550e8400-e29b-41d4-a716-446655440001");
    orphan.parent_session_id = Some("550e8400-e29b-41d4-a716-446655440099".into());
    let orphan_id = orphan.id.clone();

    assert!(resolve_from_metas(&orphan_id, &[orphan]).is_err());
}
