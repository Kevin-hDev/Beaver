use super::*;
use crate::services::agent_local::memory_runtime;
use crate::services::agent_local::memory_types::MemoryMode;

fn topic(id: &str) -> String {
    format!(
        "---\n\
         id: {id}\n\
         scope: global\n\
         type: preference\n\
         status: confirmed\n\
         title: Interface compacte\n\
         summary: Préférence durable pour une interface compacte.\n\
         created_at: 2026-07-24T20:00:00Z\n\
         updated_at: 2026-07-24T20:10:00Z\n\
         tags: [ui]\n\
         source: user\n\
         session_id: 019f951b-38a1-7882-bf2f-0784e266c911\n\
         ---\n\
         # Interface compacte\n\nUtiliser des contrôles compacts."
    )
}

#[test]
fn only_canonical_feature_roots_are_classified() {
    let data = crate::services::paths::data_dir();
    let args = serde_json::json!({"path": data.join("memory/global/MEMORY.md")});
    assert!(is_memory_operation("read_file", &args, None));
    let core = serde_json::json!({"path": data.join("memory/core/user.md")});
    assert!(!is_memory_operation("read_file", &core, None));
}

#[test]
fn traversal_into_memory_is_classified_for_authorization() {
    let data = crate::services::paths::data_dir();
    let working_dir = data.join("scratch");
    let args = serde_json::json!({"path": "../memory/global/MEMORY.md"});

    assert!(is_memory_operation(
        "read_file",
        &args,
        Some(&working_dir)
    ));
}

#[test]
fn ordinary_relative_paths_are_not_classified_as_memory() {
    let working_dir = tempfile::tempdir().unwrap();
    for path in [".", "./src"] {
        let args = serde_json::json!({"path": path});
        assert!(!is_memory_operation(
            "list_dir",
            &args,
            Some(working_dir.path())
        ));
    }
}

#[test]
fn a_relative_path_without_a_working_directory_has_no_memory_domain() {
    let args = serde_json::json!({"path": "src/main.rs"});

    assert!(!is_memory_operation("read_file", &args, None));
    assert!(event_domain("read_file", &args).is_none());
}

#[test]
fn runtime_authorization_replaces_the_general_prompt_only_for_memory_writes() {
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = memory_runtime::begin(&session, MemoryMode::Automatic, true, 3_000, 0);
    let memory_path = crate::services::paths::data_dir()
        .join("memory/global/topics/00000000-0000-4000-8000-000000000000.md");
    let memory_args = serde_json::json!({"path": memory_path});
    let project_args = serde_json::json!({"path": "/tmp/project/file.md"});

    assert_eq!(
        write_authorization("write_file", &memory_args, std::path::Path::new("/tmp"), &session),
        Some(true)
    );
    assert_eq!(
        write_authorization("write_file", &project_args, std::path::Path::new("/tmp"), &session),
        None
    );
}

#[tokio::test]
async fn list_current_project_directory_is_not_intercepted_as_memory() {
    let dir = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(dir.path().join("memory"));
    let session = uuid::Uuid::new_v4().to_string();

    for tool_name in ["list_dir", "glob"] {
        let args = serde_json::json!({"path": ".", "pattern": "**/*"});
        let result =
            dispatch_with_layout(tool_name, &args, dir.path(), &session, ".", &layout).await;

        assert!(result.is_none(), "{tool_name} was intercepted");
    }
}

#[tokio::test]
async fn manual_mode_blocks_a_write_without_an_explicit_request() {
    let dir = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(dir.path().join("memory"));
    let id = uuid::Uuid::new_v4().to_string();
    let path = layout.global_scope().topics_dir().join(format!("{id}.md"));
    let args = serde_json::json!({"path": path, "content": topic(&id)});
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = memory_runtime::begin(&session, MemoryMode::Manual, false, 3_000, 0);

    let result = dispatch_with_layout(
        "write_file",
        &args,
        dir.path(),
        &session,
        path.to_str().unwrap(),
        &layout,
    )
    .await
    .unwrap();

    assert!(result.is_error);
    assert!(!path.exists());
}

#[tokio::test]
async fn search_in_an_empty_scope_returns_no_memory_instead_of_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(dir.path().join("memory"));
    let topics = layout.global_scope().topics_dir();
    let args = serde_json::json!({"path": topics, "pattern": "css"});
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = memory_runtime::begin(&session, MemoryMode::Automatic, true, 3_000, 0);

    let result = dispatch_with_layout(
        "grep",
        &args,
        dir.path(),
        &session,
        topics.to_str().unwrap(),
        &layout,
    )
    .await
    .unwrap();

    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("Aucune mémoire"));
    assert!(!layout.global_scope().root.exists());
}

#[tokio::test]
async fn automatic_mode_writes_a_valid_topic_and_rebuilds_indexes() {
    let dir = tempfile::tempdir().unwrap();
    let layout = MemoryLayout::at(dir.path().join("memory"));
    let id = uuid::Uuid::new_v4().to_string();
    let path = layout.global_scope().topics_dir().join(format!("{id}.md"));
    let args = serde_json::json!({"path": path, "content": topic(&id)});
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = memory_runtime::begin(&session, MemoryMode::Automatic, true, 3_000, 0);

    let result = dispatch_with_layout(
        "write_file",
        &args,
        dir.path(),
        &session,
        path.to_str().unwrap(),
        &layout,
    )
    .await
    .unwrap();

    assert!(!result.is_error, "{}", result.content);
    assert!(path.exists());
    assert!(layout.global_scope().registry_path().exists());
    assert!(layout.global_scope().summary_path().exists());
}
