use super::cwd_resolver::{resolve, resolve_with};
use std::path::Path;

const INVALID: &str = "terminal-cwd-invalid";

#[tokio::test]
async fn default_group_returns_the_canonical_home() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("child")).unwrap();
    let home_with_parent = root.path().join("child/..");

    let resolved = resolve_with("__default__", &home_with_parent, unreachable_find)
        .await
        .unwrap();

    assert_eq!(resolved, dunce::canonicalize(root.path()).unwrap());
}

#[tokio::test]
async fn project_group_uses_only_the_canonical_registry_path() {
    let root = tempfile::tempdir().unwrap();
    let key_source = root.path().join("not-the-project");
    let registered = root.path().join("Projet espace é");
    std::fs::create_dir_all(&key_source).unwrap();
    std::fs::create_dir_all(&registered).unwrap();
    let group_key = key_source.to_string_lossy().into_owned();
    let expected_key = group_key.clone();
    let registered_string = registered.to_string_lossy().into_owned();

    let resolved = resolve_with(&group_key, root.path(), move |received| {
        assert_eq!(received, expected_key);
        async move { Ok(Some(registered_string)) }
    })
    .await
    .unwrap();

    assert_eq!(resolved, dunce::canonicalize(registered).unwrap());
}

#[tokio::test]
async fn directoryless_session_group_uses_its_workspace_instead_of_the_global_home() {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("session-workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let workspace_string = workspace.to_string_lossy().into_owned();
    let session_id = "550e8400-e29b-41d4-a716-446655440000";

    let resolved = resolve_with(
        &format!("session:{session_id}"),
        root.path(),
        move |received| {
            assert_eq!(received, session_id);
            async move { Ok(Some(workspace_string)) }
        },
    )
    .await
    .unwrap();

    assert_eq!(resolved, dunce::canonicalize(workspace).unwrap());
}

#[tokio::test]
async fn session_group_survives_a_deleted_project_reference() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Deleted project terminal",
        "model",
        "provider",
        false,
        Some("deleted-project".into()),
    )
    .await
    .expect("create session");
    session.messages.push(
        serde_json::from_value(serde_json::json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "turn_id": uuid::Uuid::new_v4().to_string(),
            "role": "user",
            "content": "terminal workspace",
            "files": [],
            "timestamp": "2026-09-01T00:00:00Z",
            "tokens": 0
        }))
        .expect("user message"),
    );
    crate::services::agent_local::session_store::save(&session)
        .await
        .expect("save session message");

    let resolved = resolve(&format!("session:{}", session.id))
        .await
        .expect("terminal workspace remains available");

    assert!(resolved.is_dir());
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("cleanup session");
}

#[tokio::test]
async fn empty_group_key_is_invalid() {
    assert_invalid(resolve_with("", Path::new("/"), unreachable_find).await);
}

#[tokio::test]
async fn relative_registry_path_is_invalid() {
    let result = resolve_with("project-a", Path::new("/"), |_| async {
        Ok(Some("relative/project".to_string()))
    })
    .await;

    assert_invalid(result);
}

#[tokio::test]
async fn group_key_larger_than_128_bytes_is_invalid() {
    let key = "k".repeat(129);

    assert_invalid(resolve_with(&key, Path::new("/"), unreachable_find).await);
}

#[tokio::test]
async fn group_key_control_characters_are_invalid() {
    for key in ["bad\0key", "bad\rkey", "bad\nkey"] {
        assert_invalid(resolve_with(key, Path::new("/"), unreachable_find).await);
    }
}

#[tokio::test]
async fn missing_project_is_invalid() {
    let result = resolve_with("project-a", Path::new("/"), |_| async { Ok(None) }).await;

    assert_invalid(result);
}

#[tokio::test]
async fn registry_file_path_is_invalid() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("project-file");
    std::fs::write(&file, b"not a directory").unwrap();
    let file = file.to_string_lossy().into_owned();
    let result = resolve_with("project-a", root.path(), |_| async move { Ok(Some(file)) }).await;

    assert_invalid(result);
}

#[tokio::test]
async fn registry_failure_is_invalid() {
    let result = resolve_with("project-a", Path::new("/"), |_| async {
        Err("internal registry detail".to_string())
    })
    .await;

    assert_invalid(result);
}

async fn unreachable_find(_: String) -> Result<Option<String>, String> {
    panic!("invalid keys and the default group must not query the project registry")
}

fn assert_invalid(result: Result<std::path::PathBuf, String>) {
    assert_eq!(result, Err(INVALID.to_string()));
}
