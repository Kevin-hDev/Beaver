#[tokio::test]
async fn evidence_sections_are_wired_and_share_one_bounded_envelope() {
    let mut session = super::super::snapshot_tests::session();
    session.messages[0].files.push(
        crate::services::agent_local::types_message::FileAttachment {
            name: "requirements.txt".into(),
            path: String::new(),
            mime_type: "text/plain".into(),
            size: 12,
            thumbnail: None,
            access_grant: None,
        },
    );
    let mut snapshot = super::super::snapshot_tests::snapshot(&session)
        .with_runtime_context(Vec::new(), Vec::new(), 100_000)
        .unwrap();
    snapshot.profile.profile.compact.recent_file_count = 0;
    snapshot.profile.profile.compact.include_work_state = false;

    let collected = super::super::orchestrator_sections::collect(
        &snapshot,
        &[],
        std::path::Path::new("/"),
        1_000,
    )
    .await
    .unwrap();
    let names = collected
        .sections
        .iter()
        .map(|section| section.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"text_attachments"));
    assert!(names.contains(&"critical_references"));
    assert!(collected.evidence_tokens <= 4_000);
}
