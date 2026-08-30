use super::*;
use crate::services::agent_local::{session_store, session_store_updates};
use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

#[tokio::test]
async fn project_cleanup_revalidates_the_selected_project_under_lock() {
    let mut session = session_store::create_full(
        "Project race",
        "llama3",
        "ollama",
        false,
        Some("deleted-project".to_string()),
    )
    .await
    .expect("create session");
    session.project_id = Some("deleted-project".to_string());
    session_store::save(&session).await.expect("seed project");
    let (selected_tx, selected_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let cleanup = tokio::spawn(async move {
        clear_project_id_with_after_list("deleted-project", move || async move {
            let _ = selected_tx.send(());
            let _ = release_rx.await;
        })
        .await
    });
    selected_rx.await.expect("cleanup selected stale metadata");

    session_store_updates::update_locked(&session.id, |current| {
        current.project_id = Some("replacement-project".to_string());
    })
    .await
    .expect("move session to replacement project");
    let _ = release_tx.send(());
    cleanup.await.expect("join cleanup").expect("cleanup");

    let saved = session_store::get(&session.id).await.expect("reload");
    assert_eq!(saved.project_id.as_deref(), Some("replacement-project"));
    session_store::delete_one(&session.id)
        .await
        .expect("cleanup session");
}

#[tokio::test]
async fn truncate_rejects_a_continuation_from_the_legacy_ipc_boundary() {
    let session = session_store::create_full(
        "IPC replacement guard",
        "fixture-model",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create session");
    let original = message("target-message", "visible original", None);
    session_store::add_messages(&session.id, vec![original], 0)
        .await
        .expect("seed target");
    let envelope = ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: RouteId::Ollama,
            model_id: "fixture-model".into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: "opaque fixture".into(),
        },
        Vec::new(),
    );
    let replacement = message("replacement-message", "replacement", Some(envelope));

    let result = truncate_and_replace(&session.id, "target-message", Some(replacement)).await;
    let restored = session_store::get(&session.id).await.expect("reload");

    assert!(result.is_err());
    assert_eq!(restored.messages.len(), 1);
    assert_eq!(restored.messages[0].id, "target-message");
    assert_eq!(restored.messages[0].content, "visible original");
    session_store::delete_one(&session.id)
        .await
        .expect("cleanup");
}

fn message(
    id: &str,
    content: &str,
    continuation: Option<ReasoningEnvelope>,
) -> crate::services::agent_local::types_session::AgentMessage {
    crate::services::agent_local::types_session::AgentMessage {
        id: id.into(),
        turn_id: "turn-fixture".into(),
        role: "assistant".into(),
        content: content.into(),
        message_kind: None,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation,
        replay_source: None,
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
