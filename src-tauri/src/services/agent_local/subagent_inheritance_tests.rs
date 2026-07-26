#[tokio::test]
async fn fresh_child_inherits_parent_model_and_reasoning() {
    let project = tempfile::tempdir().expect("project");
    let mut parent = super::session_store::create_full(
        "Parent",
        "reasoning-model",
        "provider-x",
        false,
        None,
    )
    .await
    .expect("parent");
    parent.thinking_enabled = true;
    parent.reasoning_mode = Some("high".into());
    parent.working_dir = project.path().to_string_lossy().to_string();
    super::session_store::save(&parent).await.expect("save parent");

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
    assert_eq!(child.working_dir, parent.working_dir);
    super::session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}
