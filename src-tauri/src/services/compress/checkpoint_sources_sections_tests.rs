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
    let mut snapshot = super::super::snapshot_tests::snapshot(&session);
    let band = &mut snapshot.profile.profile.compact;
    band.evidence_envelope = super::super::profile_types::TokenBudget::fixed(120);
    band.files.enabled = false;
    band.modified_files.enabled = false;
    band.text_attachments.enabled = true;
    band.text_attachments.max_items = 2;
    band.text_attachments.tokens_per_item = 60;
    band.critical_references.enabled = true;
    band.critical_references.max_items = 4;
    band.critical_references.total_tokens = 60;

    let collected =
        super::super::orchestrator_sections::collect(&snapshot, &[], std::path::Path::new("/"))
            .await
            .unwrap();
    let names = collected
        .sections
        .iter()
        .map(|section| section.name.as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"text_attachments"));
    assert!(names.contains(&"critical_references"));
    assert!(collected.evidence_tokens <= 120);
}
