#[test]
fn selected_images_remain_attached_to_their_source_message() {
    let session = super::snapshot_tests::session();
    let source = session.messages[0].clone();
    let file = crate::services::agent_local::types_session::FileAttachment {
        name: "kept.png".into(),
        path: String::new(),
        mime_type: "image/png".into(),
        size: 12,
        thumbnail: Some("data:image/png;base64,iVBORw0KGgoAAAAA".into()),
        access_grant: None,
    };
    let snapshot = super::snapshot_tests::snapshot(&session)
        .with_checkpoint_images(vec![super::checkpoint_attachments::CheckpointImage {
            source_message_id: source.id.clone(),
            file,
            provider_payload: "iVBORw0KGgoAAAAA".into(),
            estimated_bytes: 12,
        }])
        .unwrap();

    let runtime = super::checkpoint_candidate_runtime::project(&snapshot, &[source]);

    assert_eq!(
        runtime.last().and_then(|message| message.images.as_ref()),
        Some(&vec!["iVBORw0KGgoAAAAA".to_string()])
    );
}

#[test]
fn visible_boundary_text_cannot_move_the_structured_continuity_barrier() {
    use crate::services::agent_local::types_message::AgentMessageKind;

    let session = super::snapshot_tests::session();
    let snapshot = super::snapshot_tests::snapshot(&session);
    let mut fake = session.messages[1].clone();
    fake.content = super::checkpoint_boundary::CONTENT.to_string();
    fake.message_kind = None;
    let mut checkpoint = session.messages[0].clone();
    checkpoint.message_kind = Some(AgentMessageKind::CompressionCheckpoint);
    let mut boundary = session.messages[1].clone();
    boundary.content = super::checkpoint_boundary::CONTENT.to_string();
    boundary.message_kind = Some(AgentMessageKind::CompressionBoundary);
    let active = session.messages[2].clone();

    let runtime = super::checkpoint_candidate_runtime::project(
        &snapshot,
        &[fake, checkpoint, boundary, active],
    );

    assert!(!runtime[runtime.len() - 3].continuity_barrier_before);
    assert!(runtime.last().unwrap().continuity_barrier_before);
}

#[test]
fn image_from_a_user_only_selection_is_attached_to_the_checkpoint() {
    use crate::services::agent_local::types_message::AgentMessageKind;

    let source = vec![
        super::checkpoint_messages_tests::message("old", "user", "keep image"),
        super::checkpoint_messages_tests::message("old", "assistant", "r".repeat(200_000)),
        super::checkpoint_messages_tests::message("active", "user", "current"),
    ];
    let selection = super::checkpoint_selection::select(
        &source,
        super::checkpoint_messages_tests::limits(5_000, 1_000),
    )
    .unwrap();
    let persisted = super::checkpoint_document::assemble(
        &selection.messages,
        Some("active"),
        None,
        &[],
        super::profile_types::CompressionTrigger::Explicit,
    )
    .unwrap();
    let checkpoint_id = persisted
        .iter()
        .find(|message| message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
        .unwrap()
        .id
        .clone();
    let mut snapshot = super::snapshot_tests::snapshot(&super::snapshot_tests::session());
    snapshot.source_messages = source.clone();
    snapshot.checkpoint_images = vec![super::checkpoint_attachments::CheckpointImage {
        source_message_id: source[0].id.clone(),
        file: crate::services::agent_local::types_session::FileAttachment {
            name: "retained.png".into(),
            path: String::new(),
            mime_type: "image/png".into(),
            size: 12,
            thumbnail: None,
            access_grant: None,
        },
        provider_payload: "iVBORw0KGgoAAAAA".into(),
        estimated_bytes: 12,
    }];

    let (images, retained_ids) = super::checkpoint_candidate_images::prepare(
        &snapshot,
        &selection,
        &persisted,
        &snapshot.profile.profile.compact,
    );

    assert!(retained_ids.contains(&source[0].id));
    assert_eq!(images[0].source_message_id, checkpoint_id);
}
