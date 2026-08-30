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
