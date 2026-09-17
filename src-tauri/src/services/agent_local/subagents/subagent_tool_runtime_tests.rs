use serde_json::json;

#[tokio::test]
async fn backend_rejects_removed_tools_for_child_sessions() {
    let root = tempfile::tempdir().expect("root");
    let parent = super::session_store::create_full("Parent", "llama3", "ollama", false, None)
        .await
        .expect("parent");
    let mut child = super::session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".to_string());
    super::session_store::save(&child)
        .await
        .expect("save child");

    let denied = super::tool_dispatcher::dispatch(
        "write_file",
        &json!({"path": "blocked.txt", "content": "blocked"}),
        root.path(),
        &child.id,
        tokio_util::sync::CancellationToken::new(),
    )
    .await;
    let work = super::agent_work_supervision::ShellWork::new(
        crate::app_exit::AppExitCoordinator::initialize()
            .expect("exit coordinator")
            .work_supervisor(),
    );
    let pwd = super::tool_dispatcher_shell::execute_command_with_work(
        &json!({"command": "pwd"}),
        root.path(),
        &child.id,
        tokio_util::sync::CancellationToken::new(),
        Some(super::subagent_tool_profile::SubagentToolProfile::Explorer),
        None,
        work,
    )
    .await;

    assert!(denied.is_error);
    assert!(!root.path().join("blocked.txt").exists());
    let pwd = pwd.expect("explorer shell remains available");
    assert_eq!(
        pwd.stdout.trim(),
        dunce::canonicalize(root.path())
            .expect("canonical root")
            .to_string_lossy()
    );
    super::session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    super::session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

#[tokio::test]
async fn corrupted_child_profile_fails_closed() {
    let root = tempfile::tempdir().expect("root");
    let mut child = super::session_store::create_full("Child", "llama3", "ollama", false, None)
        .await
        .expect("child");
    child.parent_session_id = Some(uuid::Uuid::new_v4().to_string());
    super::session_store::save(&child)
        .await
        .expect("save child");
    let result = super::tool_dispatcher::dispatch(
        "read_file",
        &json!({"path": "missing.txt"}),
        root.path(),
        &child.id,
        tokio_util::sync::CancellationToken::new(),
    )
    .await;
    assert!(result.is_error);
    assert!(result.content.contains("Profil"));
    super::session_store::delete_one(&child.id)
        .await
        .expect("delete child");
}
