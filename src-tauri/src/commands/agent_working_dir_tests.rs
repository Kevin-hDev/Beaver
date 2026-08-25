use super::{
    canonical_dir, canonical_optional_dir, choose_project_root, is_home_directory,
    ResolvedWorkingDir,
};

#[test]
fn canonicalizes_existing_dir() {
    let temp = tempfile::tempdir().expect("tempdir");
    let nested = temp.path().join("nested");
    std::fs::create_dir_all(&nested).expect("nested");

    let resolved = canonical_dir(&nested.join(".").to_string_lossy()).expect("resolved");

    assert_eq!(
        resolved.path,
        dunce::canonicalize(&nested).expect("canonical")
    );
}

#[test]
fn rejects_a_missing_stored_root_instead_of_falling_back() {
    let missing = "/definitely/missing/beaver-project-root";

    assert!(canonical_optional_dir(Some(missing)).is_err());
}

#[test]
fn home_detection_only_answers_the_geographic_question() {
    let home = dirs::home_dir().expect("home");

    assert!(is_home_directory(home.to_string_lossy().as_ref()));
}

#[test]
fn project_wins_over_incoming_and_stored_directories() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let incoming = temp.path().join("incoming");
    let outside = temp.path().join("outside");
    std::fs::create_dir_all(&project).expect("project");
    std::fs::create_dir_all(&incoming).expect("incoming");
    std::fs::create_dir_all(&outside).expect("outside");

    let resolved = choose_project_root(
        Some(resolved(project.clone())),
        Some(resolved(incoming)),
        Some(resolved(outside)),
    )
    .expect("resolved");

    assert_eq!(resolved.path, project);
}

#[test]
fn project_root_wins_over_a_stored_subdirectory() {
    let temp = tempfile::tempdir().expect("tempdir");
    let project = temp.path().join("project");
    let nested = project.join("nested");
    std::fs::create_dir_all(&nested).expect("nested");

    let resolved = choose_project_root(
        Some(resolved(project.clone())),
        None,
        Some(resolved(nested)),
    )
    .expect("resolved");

    assert_eq!(resolved.path, project);
}

#[test]
fn projectless_session_prefers_incoming_then_stored_directory() {
    let incoming = resolved(std::path::PathBuf::from("/incoming"));
    let stored = resolved(std::path::PathBuf::from("/stored"));

    let selected =
        choose_project_root(None, Some(incoming), Some(stored)).expect("incoming directory");

    assert_eq!(selected.path, std::path::PathBuf::from("/incoming"));
}

#[tokio::test]
async fn projectless_session_gets_a_hidden_workspace_instead_of_home() {
    let session = crate::services::agent_local::session_store::create_full(
        "Workspace",
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create session");
    crate::services::agent_local::session_store::add_messages(
        &session.id,
        vec![user_message("Crée un rapport")],
        0,
    )
    .await
    .expect("save first message");

    let resolved = super::resolve_for_session(&session.id, None)
        .await
        .expect("resolve workspace");
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .expect("load session");

    assert!(resolved
        .path
        .starts_with(crate::services::paths::data_dir().join("session-workspaces")));
    assert!(resolved.path.ends_with("work"));
    assert!(resolved
        .outputs_dir
        .as_ref()
        .is_some_and(|path| path.ends_with("outputs")));
    assert!(saved.working_dir_managed);
    assert!(saved.project_id.is_none());
    assert_ne!(
        resolved.path,
        dirs::home_dir()
            .expect("home")
            .canonicalize()
            .expect("canonical home")
    );

    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn projectless_session_reuses_its_persisted_workspace() {
    let session = crate::services::agent_local::session_store::create_full(
        "Workspace reuse",
        "llama3",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create session");
    crate::services::agent_local::session_store::add_messages(
        &session.id,
        vec![user_message("Create a report")],
        0,
    )
    .await
    .expect("save first message");

    let first = super::resolve_for_session(&session.id, None)
        .await
        .expect("resolve first workspace");
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .expect("load persisted workspace");
    let saved_path = std::path::Path::new(&saved.working_dir);
    assert_eq!(saved_path, dunce::simplified(saved_path));
    let second = super::resolve_for_session(&session.id, None).await;

    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    let second = second.expect("reuse persisted workspace");
    assert_eq!(
        dunce::canonicalize(first.path).expect("canonical first workspace"),
        dunce::canonicalize(second.path).expect("canonical second workspace")
    );
}

#[tokio::test]
async fn user_selected_directory_is_never_marked_as_managed() {
    let session = crate::services::agent_local::session_store::create_full(
        "Selected", "llama3", "ollama", false, None,
    )
    .await
    .expect("create session");
    let selected = tempfile::tempdir().expect("selected");

    let resolved = super::resolve_for_session(
        &session.id,
        Some(selected.path().to_string_lossy().as_ref()),
    )
    .await
    .expect("resolve selected directory");
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .expect("load session");

    assert_eq!(
        resolved.path,
        dunce::canonicalize(selected.path()).expect("canonical")
    );
    assert!(!saved.working_dir_managed);

    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

fn resolved(path: std::path::PathBuf) -> ResolvedWorkingDir {
    ResolvedWorkingDir {
        path,
        outputs_dir: None,
    }
}

fn user_message(content: &str) -> crate::services::agent_local::types_session::AgentMessage {
    crate::services::agent_local::types_session::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: crate::services::agent_local::types_session::AgentMessage::new_turn_id(),
        role: "user".to_string(),
        content: content.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}
