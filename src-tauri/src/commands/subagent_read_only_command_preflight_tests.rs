use super::subagent_read_only_command_test_support::{
    assert_rejected, child_session, cleanup, snapshot, SUBAGENT_READ_ONLY,
};
use crate::services::agent_local::session_store;

#[tokio::test]
async fn prepare_agent_send_rejects_a_child_without_persisting_changes() {
    let session = child_session("Prepare").await;
    let before = snapshot(&session.id).await;

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::prepare_agent_send(session.id.clone(), None),
    )
    .await;
}

#[tokio::test]
async fn resolve_missing_session_directory_rejects_a_child_before_creating_it() {
    let root = tempfile::tempdir().expect("temporary root");
    let missing = root.path().join("missing-child-directory");
    let mut session = child_session("Directory").await;
    session.working_dir = missing.to_string_lossy().to_string();
    session_store::save(&session)
        .await
        .expect("save missing directory");
    let before = snapshot(&session.id).await;
    let error = super::agent_sessions::resolve_missing_session_directory(
        session.id.clone(),
        missing.to_string_lossy().to_string(),
        crate::services::agent_local::agent_send_preflight::MissingDirectoryAction::Create,
    )
    .await
    .err();
    let after = snapshot(&session.id).await;
    cleanup(&session).await;

    assert_eq!(error.as_deref(), Some(SUBAGENT_READ_ONLY));
    assert_eq!(after, before);
    assert!(
        !missing.exists(),
        "the rejected command created a directory"
    );
}
