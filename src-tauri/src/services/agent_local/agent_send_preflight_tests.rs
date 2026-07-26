use super::agent_send_preflight::{MissingDirectoryAction, PrepareAgentSend};

#[tokio::test]
async fn missing_directory_returns_nearest_existing_parent() {
    let root = tempfile::tempdir().expect("root");
    let missing = root.path().join("deleted/nested");
    let mut session = super::session_store::create_full("Missing", "model", "provider", false, None)
        .await
        .expect("session");
    session.working_dir = missing.to_string_lossy().to_string();
    super::session_store::save(&session).await.expect("save");

    let prepared = super::agent_send_preflight::prepare(&session.id, None)
        .await
        .expect("prepare");

    assert_eq!(
        prepared,
        PrepareAgentSend::Missing {
            missing_path: missing.to_string_lossy().to_string(),
            nearest_parent: root.path().canonicalize().unwrap().to_string_lossy().to_string(),
        }
    );
    super::session_store::delete_one(&session.id).await.expect("cleanup");
}

#[tokio::test]
async fn create_rebuilds_only_empty_path_and_switch_updates_session() {
    let root = tempfile::tempdir().expect("root");
    let missing = root.path().join("gone/path");
    let mut session = super::session_store::create_full("Recover", "model", "provider", false, None)
        .await
        .expect("session");
    session.project_id = Some("deleted-project".into());
    session.working_dir = missing.to_string_lossy().to_string();
    super::session_store::save(&session).await.expect("save");

    let created = super::agent_send_preflight::resolve(
        &session.id,
        &missing.to_string_lossy(),
        MissingDirectoryAction::Create,
    )
    .await
    .expect("create");
    assert!(missing.is_dir());
    assert_eq!(std::fs::read_dir(&missing).unwrap().count(), 0);
    assert_eq!(created, missing.canonicalize().unwrap().to_string_lossy());

    std::fs::remove_dir_all(root.path().join("gone")).expect("remove again");
    let recreated_path = super::session_store::get(&session.id)
        .await
        .expect("created session")
        .working_dir;
    let switched = super::agent_send_preflight::resolve(
        &session.id,
        &recreated_path,
        MissingDirectoryAction::Switch,
    )
    .await
    .expect("switch");
    assert_eq!(switched, root.path().canonicalize().unwrap().to_string_lossy());
    let saved = super::session_store::get(&session.id).await.expect("saved");
    assert_eq!(saved.working_dir, switched);
    let project_id = saved.project_id.expect("switched project id");
    let switched_project = super::project_store::list()
        .await
        .expect("projects")
        .into_iter()
        .find(|project| project.id == project_id)
        .expect("registered parent project");
    assert_eq!(switched_project.path, switched);
    super::session_store::delete_one(&session.id).await.expect("cleanup");
}
