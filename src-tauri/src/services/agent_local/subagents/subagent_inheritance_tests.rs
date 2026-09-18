#[tokio::test]
async fn fresh_child_inherits_parent_model_and_reasoning() {
    let project = tempfile::tempdir().expect("project");
    let mut parent =
        super::session_store::create_full("Parent", "reasoning-model", "provider-x", false, None)
            .await
            .expect("parent");
    parent.thinking_enabled = true;
    parent.reasoning_mode = Some("high".into());
    parent.preserve_reasoning = super::types_session::PreserveReasoningSetting::Local;
    parent.fast_mode_enabled = true;
    parent.working_dir = project.path().to_string_lossy().to_string();
    super::session_store::save(&parent)
        .await
        .expect("save parent");

    let child = super::tool_delegate_child::create_child(
        &parent,
        &parent.id,
        "explorer",
        "mission",
        "Geminitor",
        "description",
        "explorer",
        "run-id",
    )
    .await
    .expect("child");

    assert_eq!(child.model, parent.model);
    assert_eq!(child.provider, parent.provider);
    assert!(child.thinking_enabled);
    assert_eq!(child.reasoning_mode.as_deref(), Some("high"));
    assert_eq!(child.preserve_reasoning, parent.preserve_reasoning);
    assert!(!child.fast_mode_enabled);
    assert_eq!(child.working_dir, parent.working_dir);
    super::session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
#[tokio::test]
async fn prepared_extension_child_remains_owned_and_revocable() {
    use super::types_session::{SubagentExtensionOwner, SubagentExtensionOwnership};
    use super::{session_store, subagent_registry, tool_delegate_child};
    use tokio_util::sync::CancellationToken;

    let parent = session_store::create_full("Parent", "model", "ollama", false, None)
        .await
        .expect("parent");
    let mut child = tool_delegate_child::create_child(
        &parent,
        &parent.id,
        "explorer",
        "mission",
        "Child",
        "description",
        "explorer",
        "run",
    )
    .await
    .expect("child");
    let owner = SubagentExtensionOwner {
        extension_id: format!("test.{}", uuid::Uuid::new_v4()),
        extension_version: "1.0.0".into(),
        extension_fingerprint: "a".repeat(64),
    };
    child.subagent_extension_owner = Some(SubagentExtensionOwnership::Valid(owner.clone()));
    tool_delegate_child::inherit_parent_context(&mut child, &parent)
        .await
        .expect("prepare child context");
    let saved = session_store::get(&child.id).await.expect("saved child");
    assert_eq!(
        saved
            .subagent_extension_owner
            .as_ref()
            .and_then(SubagentExtensionOwnership::valid),
        Some(&owner)
    );
    let cancel = CancellationToken::new();
    subagent_registry::register(&parent.id, &child.id, cancel.clone())
        .await
        .expect("register");
    subagent_registry::cancel_children_for_extension(&owner.extension_id).await;
    assert!(
        cancel.is_cancelled(),
        "revocation must cancel the prepared child"
    );
    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
