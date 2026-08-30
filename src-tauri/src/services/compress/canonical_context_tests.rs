use super::canonical_context::rebuild_canonical_context;

#[tokio::test]
async fn unchanged_sources_rebuild_to_identical_bytes() {
    let session = super::snapshot_tests::session();
    let snapshot = super::snapshot_tests::snapshot(&session);

    let first = rebuild_canonical_context(&session, &snapshot)
        .await
        .unwrap();
    let second = rebuild_canonical_context(&session, &snapshot)
        .await
        .unwrap();

    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
}

#[tokio::test]
async fn persisted_system_text_is_never_copied_into_canonical_history() {
    let mut session = super::snapshot_tests::session();
    let mut system = session.messages[0].clone();
    system.id = uuid::Uuid::new_v4().to_string();
    system.role = "system".to_string();
    system.content = "stale permission mode".to_string();
    session.messages.push(system);
    let snapshot = super::snapshot_tests::snapshot(&session);

    let rebuilt = rebuild_canonical_context(&session, &snapshot)
        .await
        .unwrap();

    assert!(rebuilt
        .messages
        .iter()
        .all(|message| message.role != "system"));
    assert!(rebuilt
        .messages
        .iter()
        .all(|message| !message.content.contains("stale permission mode")));
}
