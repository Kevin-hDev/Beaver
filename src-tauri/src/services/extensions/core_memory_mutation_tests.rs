use super::support::{context, invoke, topic};
use crate::services::agent_local::memory_paths::MemoryLayout;
use crate::services::agent_local::memory_types::MemoryMode;
use serde_json::json;

#[tokio::test]
async fn stale_memory_write_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let session = uuid::Uuid::new_v4().to_string();
    let context = context(&session, project.path());
    let _guard = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Automatic,
        false,
        3_000,
        0,
    );
    let created = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "content": topic("first")}),
        &layout,
    )
    .await
    .unwrap();
    let id = created["topic"]["id"].as_str().unwrap();
    let revision = created["topic"]["updatedAt"].as_str().unwrap();
    invoke(
        &context,
        "memory.write",
        json!({
            "scope": "global",
            "topicId": id,
            "expectedUpdatedAt": revision,
            "content": topic("second"),
        }),
        &layout,
    )
    .await
    .unwrap();
    let stale = invoke(
        &context,
        "memory.write",
        json!({
            "scope": "global",
            "topicId": id,
            "expectedUpdatedAt": revision,
            "content": topic("third"),
        }),
        &layout,
    )
    .await;
    assert_eq!(
        stale,
        Err(super::super::super::core_bridge::ExtensionBridgeError::Backend(
            "core_memory_stale"
        ))
    );
}

#[tokio::test]
async fn memory_applied_but_unindexed_is_not_retried() {
    let root = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let session = uuid::Uuid::new_v4().to_string();
    let context = context(&session, project.path());
    let _guard = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Automatic,
        false,
        3_000,
        0,
    );
    let created = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "content": topic("archive")}),
        &layout,
    )
    .await
    .unwrap();
    let id = created["topic"]["id"].as_str().unwrap().to_string();
    let scope = layout.global_scope();
    tokio::fs::remove_file(scope.summary_path()).await.unwrap();
    tokio::fs::create_dir(scope.summary_path()).await.unwrap();

    let archived = invoke(
        &context,
        "memory.archive",
        json!({"scope": "global", "topicId": id}),
        &layout,
    )
    .await
    .unwrap();
    assert_eq!(archived["applied"], true);
    assert_eq!(archived["indexUpdated"], false);
    assert!(archived["topic"]["content"]
        .as_str()
        .unwrap()
        .contains("status: archived"));
    assert_eq!(invoke(
        &context,
        "memory.archive",
        json!({"scope": "global", "topicId": id}),
        &layout,
    )
    .await,
    Err(super::super::super::core_bridge::ExtensionBridgeError::Backend(
        "core_memory_not_found"
    )));
}

#[tokio::test]
async fn concurrent_memory_updates_accept_only_one_revision() {
    let root = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let session = uuid::Uuid::new_v4().to_string();
    let context = context(&session, project.path());
    let _guard = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Automatic,
        false,
        3_000,
        0,
    );
    let created = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "content": topic("initial")}),
        &layout,
    )
    .await
    .unwrap();
    let id = created["topic"]["id"].as_str().unwrap().to_string();
    let revision = created["topic"]["updatedAt"].as_str().unwrap().to_string();
    let first = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "topicId": id, "expectedUpdatedAt": revision, "content": topic("first")}),
        &layout,
    );
    let second = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "topicId": id, "expectedUpdatedAt": revision, "content": topic("second")}),
        &layout,
    );
    let (first, second) = tokio::join!(first, second);
    assert_ne!(first.is_ok(), second.is_ok());
}
