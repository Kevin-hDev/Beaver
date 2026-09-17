use super::support::{context, invoke, topic};
use crate::services::agent_local::memory_paths::MemoryLayout;
use crate::services::agent_local::memory_types::MemoryMode;
use serde_json::json;

#[tokio::test]
async fn memory_api_obeys_disabled_manual_automatic_modes() {
    let root = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let session = uuid::Uuid::new_v4().to_string();
    let context = context(&session, project.path());

    let disabled = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Disabled,
        false,
        3_000,
        0,
    );
    assert!(
        invoke(&context, "memory.list", json!({"scope": "global"}), &layout)
            .await
            .is_err()
    );
    drop(disabled);

    let manual = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Manual,
        false,
        3_000,
        0,
    );
    assert!(invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "content": topic("manual")}),
        &layout,
    )
    .await
    .is_err());
    drop(manual);

    let automatic = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Automatic,
        false,
        3_000,
        0,
    );
    let created = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "content": topic("automatic")}),
        &layout,
    )
    .await
    .unwrap();
    assert_eq!(created["applied"], true);
    assert!(created["topic"]["content"]
        .as_str()
        .unwrap()
        .contains("status: inferred"));
    drop(automatic);
}

#[tokio::test]
async fn memory_api_shares_turn_budget_with_native_tools() {
    let root = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(root.path().join("memory"));
    let session = uuid::Uuid::new_v4().to_string();
    let context = context(&session, project.path());
    let _guard = crate::services::agent_local::memory_runtime::begin(
        &session,
        MemoryMode::Automatic,
        false,
        1_000,
        0,
    );
    let created = invoke(
        &context,
        "memory.write",
        json!({"scope": "global", "content": topic("durable preference")}),
        &layout,
    )
    .await
    .unwrap();
    let topic_id = created["topic"]["id"].as_str().unwrap();
    let _ = crate::services::agent_local::memory_runtime::consume_result(
        &session,
        &"native result ".repeat(1_000),
    );
    let read = invoke(
        &context,
        "memory.read",
        json!({"scope": "global", "topicId": topic_id}),
        &layout,
    )
    .await
    .unwrap();
    let page = invoke(&context, "memory.list", json!({"scope": "global"}), &layout)
        .await
        .unwrap();

    assert!(read["content"].as_str().unwrap().contains("budget épuisé"));
    assert_eq!(page["items"], json!([]));
    assert_eq!(page["nextCursor"], "0");
}

#[tokio::test]
async fn memory_api_cannot_change_project_or_write_index() {
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
    assert!(invoke(
        &context,
        "memory.write",
        json!({"scope": "project", "path": "/tmp/other", "content": topic("x")}),
        &layout,
    )
    .await
    .is_err());
    assert!(invoke(
        &context,
        "memory.read",
        json!({"scope": "global", "topicId": "memory_summary"}),
        &layout,
    )
    .await
    .is_err());
}

#[tokio::test]
async fn disappearing_project_and_symlinked_scope_fail_closed() {
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
    std::fs::remove_dir(project.path()).unwrap();
    assert!(invoke(
        &context,
        "memory.list",
        json!({"scope": "project"}),
        &layout
    )
    .await
    .is_err());

    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(layout.root()).unwrap();
        std::os::unix::fs::symlink(outside.path(), layout.root().join("global")).unwrap();
        let result = invoke(
            &context,
            "memory.read",
            json!({"scope": "global", "topicId": uuid::Uuid::new_v4()}),
            &layout,
        )
        .await;
        assert!(result.is_err());
    }
}
