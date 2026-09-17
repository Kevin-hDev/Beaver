#[tokio::test]
async fn seventeenth_report_is_refused_without_session_mutation() {
    let session = crate::services::agent_local::session_store::create_full(
        "bounded e2e",
        "fixture",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create session");
    for index in 0..16 {
        let report = crate::services::agent_local::subagent_hidden_reports::build_report(
            format!("child-{index}"),
            "worker".into(),
            "explorer".into(),
            "completed".into(),
            "bounded report".into(),
        );
        crate::services::agent_local::subagent_hidden_reports::append(&session.id, report)
            .await
            .expect("fill reports");
    }
    let overflow = crate::services::agent_local::subagent_hidden_reports::build_report(
        "child-overflow".into(),
        "worker".into(),
        "explorer".into(),
        "completed".into(),
        "must be refused".into(),
    );
    assert!(
        crate::services::agent_local::subagent_hidden_reports::append(&session.id, overflow)
            .await
            .is_err()
    );

    let reports =
        crate::services::agent_local::subagent_hidden_reports::peek_reports(&session.id).await;
    assert_eq!(reports.len(), 16);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("cleanup");
}
