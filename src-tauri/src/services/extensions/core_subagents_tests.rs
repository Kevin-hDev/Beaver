use crate::services::agent_local::types_session::{
    AgentSession, SubagentExtensionOwner, SubagentExtensionOwnership,
};

#[test]
fn subagents_require_parent_agent_mode_and_declared_parameters() {
    assert!(!super::core_subagents::scope_allows_subagents(
        false, false, "auto"
    ));
    assert!(!super::core_subagents::scope_allows_subagents(
        true, true, "auto"
    ));
    assert!(!super::core_subagents::scope_allows_subagents(
        true, false, "chat"
    ));
    assert!(!super::core_subagents::scope_allows_subagents(
        true, false, "future"
    ));
    assert!(super::core_subagents::scope_allows_subagents(
        true, false, "manual"
    ));
    assert!(super::core_subagents::validate_keys(
        "subagents.get",
        &serde_json::json!({"subagentId": "child", "extensionId": "spoof"}),
    )
    .is_err());
}

#[test]
fn extension_cannot_control_another_owners_child() {
    let mut child: AgentSession = serde_json::from_value(serde_json::json!({
        "schema_version": 7,
        "id": "00000000-0000-4000-8000-000000000001",
        "name": "child",
        "created_at": "2026-09-16T00:00:00Z",
        "model": "model",
        "accumulated_tokens": 0,
        "messages": [],
        "parent_session_id": "parent"
    }))
    .unwrap();
    child.subagent_extension_owner =
        Some(SubagentExtensionOwnership::Valid(SubagentExtensionOwner {
            extension_id: "owner.one".into(),
            extension_version: "1.0.0".into(),
            extension_fingerprint: "ab".repeat(32),
        }));
    let neighbor = SubagentExtensionOwner {
        extension_id: "owner.two".into(),
        extension_version: "1.0.0".into(),
        extension_fingerprint: "ab".repeat(32),
    };
    assert!(
        !crate::services::agent_local::subagent_extension_api::owner_matches(
            &child, "parent", &neighbor,
        )
    );
}

#[tokio::test]
async fn revoked_owner_cannot_send_after_confirmation() {
    use crate::services::agent_local::{session_store, subagent_registry, subagent_status};
    use tokio_util::sync::CancellationToken;

    let parent = session_store::create_full("parent", "model", "ollama", false, None)
        .await
        .unwrap();
    let mut child = session_store::create_full("child", "model", "ollama", false, None)
        .await
        .unwrap();
    let approved = SubagentExtensionOwner {
        extension_id: "owner.one".into(),
        extension_version: "1.0.0".into(),
        extension_fingerprint: "ab".repeat(32),
    };
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_extension_owner = Some(SubagentExtensionOwnership::Valid(approved.clone()));
    child.subagent_status = Some(subagent_status::RUNNING.into());
    let run_id = subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .unwrap();
    child.subagent_run_id = Some(run_id);
    session_store::save(&child).await.unwrap();

    let revoked = SubagentExtensionOwner {
        extension_fingerprint: "cd".repeat(32),
        ..approved
    };
    let result = crate::services::agent_local::subagent_extension_api::send(
        &child.id,
        "continue",
        &parent.id,
        &revoked,
        CancellationToken::new(),
    )
    .await;
    let saved = session_store::get(&child.id).await.unwrap();

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.unwrap();
    session_store::delete_one(&parent.id).await.unwrap();
    assert!(result.is_err());
    assert!(saved.subagent_queued_prompts.is_empty());
}

#[tokio::test]
async fn revocation_cancels_only_owned_subagents() {
    use crate::services::agent_local::{session_store, subagent_registry};
    use tokio_util::sync::CancellationToken;

    let parent = session_store::create_full("parent", "model", "ollama", false, None)
        .await
        .unwrap();
    let first = owned_child(&parent.id, "owner.one").await;
    let neighbor = owned_child(&parent.id, "owner.two").await;
    let first_cancel = CancellationToken::new();
    let neighbor_cancel = CancellationToken::new();
    subagent_registry::register(&parent.id, &first.id, first_cancel.clone())
        .await
        .unwrap();
    subagent_registry::register(&parent.id, &neighbor.id, neighbor_cancel.clone())
        .await
        .unwrap();

    subagent_registry::cancel_children_for_extension("owner.one").await;

    assert!(first_cancel.is_cancelled());
    assert!(!neighbor_cancel.is_cancelled());
    for child in [&first, &neighbor] {
        subagent_registry::unregister(&child.id).await;
        session_store::delete_one(&child.id).await.unwrap();
    }
    session_store::delete_one(&parent.id).await.unwrap();
}

async fn owned_child(parent_id: &str, extension_id: &str) -> AgentSession {
    use crate::services::agent_local::session_store;

    let mut child = session_store::create_full("child", "model", "ollama", false, None)
        .await
        .unwrap();
    child.parent_session_id = Some(parent_id.to_string());
    child.subagent_extension_owner =
        Some(SubagentExtensionOwnership::Valid(SubagentExtensionOwner {
            extension_id: extension_id.into(),
            extension_version: "1.0.0".into(),
            extension_fingerprint: "ab".repeat(32),
        }));
    session_store::save(&child).await.unwrap();
    child
}
