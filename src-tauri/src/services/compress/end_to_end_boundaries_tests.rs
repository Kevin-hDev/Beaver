use std::collections::HashMap;
use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::models::agent_turn_contract::NewUserTurnInput;
use crate::services::agent_local::parent_message_inbox::ParentMessageInbox;
use crate::ActiveStreams;

#[tokio::test]
async fn seventeenth_report_and_active_stream_send_fail_closed_without_session_mutation() {
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

    let before = serde_json::to_vec(
        &crate::services::agent_local::session_store::get(&session.id)
            .await
            .expect("before queue"),
    )
    .unwrap();
    let inbox = Arc::new(ParentMessageInbox::new());
    let app = tauri::test::mock_builder()
        .manage(ActiveStreams(Mutex::new(HashMap::from([(
            session.id.clone(),
            (CancellationToken::new(), 7, "request".into(), inbox.clone()),
        )]))))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    let result = crate::commands::agent_chat_queue::queue_agent_message(
        session.id.clone(),
        7,
        NewUserTurnInput {
            content: "keep this draft".into(),
            files: Vec::new(),
            skills: Vec::new(),
        },
        app.state::<ActiveStreams>(),
    )
    .await;
    assert_eq!(result, Ok(false));
    assert!(!inbox.is_closed());
    let after = serde_json::to_vec(
        &crate::services::agent_local::session_store::get(&session.id)
            .await
            .expect("after queue"),
    )
    .unwrap();
    assert_eq!(after, before);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .expect("cleanup");
}
