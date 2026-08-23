use super::{session_index, session_store, session_store_updates};
use serde_json::json;

fn legacy_session_json() -> serde_json::Value {
    json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Legacy",
        "created_at": "2026-08-22T12:00:00Z",
        "model": "llama3",
        "provider": "ollama",
        "accumulated_tokens": 0,
        "messages": []
    })
}

async fn delete_sessions(ids: &[&str]) {
    for id in ids {
        session_store::delete_one(id)
            .await
            .expect("delete test session");
    }
}

#[test]
fn session_fast_mode_legacy_default_and_explicit_serialization() {
    let legacy = serde_json::from_value::<super::types_session::AgentSession>(
        legacy_session_json(),
    )
    .expect("deserialize legacy session");
    assert!(!legacy.fast_mode_enabled);

    let mut enabled = legacy;
    enabled.fast_mode_enabled = true;
    let json = serde_json::to_value(&enabled).expect("serialize enabled session");
    assert_eq!(json["fast_mode_enabled"], true);

    enabled.fast_mode_enabled = false;
    let json = serde_json::to_value(&enabled).expect("serialize disabled session");
    assert_eq!(json["fast_mode_enabled"], false);
}

#[tokio::test]
async fn session_fast_mode_creation_paths_choose_their_own_default() {
    let standard = session_store::create_full("Standard", "llama3", "ollama", false, None)
        .await
        .expect("create standard session");
    let heartbeat = session_store::create_full("Heartbeat", "llama3", "ollama", true, None)
        .await
        .expect("create heartbeat session");
    let gateway = session_store::create_gateway(
        "Gateway",
        "llama3",
        "ollama",
        "telegram:test".to_string(),
    )
    .await
    .expect("create gateway session");
    let interactive = session_store::create_with_project_and_fast_mode(
        "Interactive",
        "gpt-5.6",
        "openai",
        None,
        true,
    )
    .await
    .expect("create interactive session");

    assert!(!standard.fast_mode_enabled);
    assert!(!heartbeat.fast_mode_enabled);
    assert!(!gateway.fast_mode_enabled);
    assert!(interactive.fast_mode_enabled);
    delete_sessions(&[
        &standard.id,
        &heartbeat.id,
        &gateway.id,
        &interactive.id,
    ])
    .await;
}

#[tokio::test]
async fn session_fast_mode_is_independent_and_survives_model_changes() {
    let first = session_store::create_full("A", "gpt-5.6", "openai", false, None)
        .await
        .expect("create first session");
    let second = session_store::create_full("B", "gpt-5.6", "openai", false, None)
        .await
        .expect("create second session");

    assert!(session_store_updates::update_fast_mode(&first.id, true)
        .await
        .expect("enable first session"));
    session_store_updates::update_model(&first.id, "llama3", "ollama", None, Some(false))
        .await
        .expect("change model");

    let reloaded_first = session_store::get(&first.id).await.expect("reload first");
    let reloaded_second = session_store::get(&second.id).await.expect("reload second");
    assert!(reloaded_first.fast_mode_enabled);
    assert!(!reloaded_second.fast_mode_enabled);
    delete_sessions(&[&first.id, &second.id]).await;
}

#[tokio::test]
async fn session_fast_mode_command_persists_and_lists_confirmed_metadata() {
    let first = session_store::create_full("IPC A", "gpt-5.6", "openai", false, None)
        .await
        .expect("create first session");
    let second = session_store::create_full("IPC B", "gpt-5.6", "openai", false, None)
        .await
        .expect("create second session");

    let confirmed = crate::commands::set_session_fast_mode(first.id.clone(), true)
        .await
        .expect("set fast mode through command");
    let listed = crate::commands::list_agent_sessions()
        .await
        .expect("list sessions through command");

    assert!(confirmed);
    assert!(listed
        .iter()
        .find(|meta| meta.id == first.id)
        .expect("first metadata")
        .fast_mode_enabled);
    assert!(!listed
        .iter()
        .find(|meta| meta.id == second.id)
        .expect("second metadata")
        .fast_mode_enabled);
    delete_sessions(&[&first.id, &second.id]).await;
}

#[tokio::test]
async fn session_fast_mode_failed_writer_keeps_previous_file_value() {
    let session = session_store::create_full("Failure", "gpt-5.6", "openai", false, None)
        .await
        .expect("create session");

    let result = session_store_updates::update_fast_mode_with_writer(
        &session.id,
        true,
        |_| async { Err("injected write failure".to_string()) },
    )
    .await;

    assert!(result.is_err());
    assert!(!session_store::get(&session.id)
        .await
        .expect("reload previous file")
        .fast_mode_enabled);
    delete_sessions(&[&session.id]).await;
}

#[tokio::test]
async fn failed_index_upsert_is_reconciled_from_the_fast_mode_document() {
    let session = session_store::create_full("Index failure", "gpt-5.6", "openai", false, None)
        .await
        .expect("create session");
    session_index::fail_next_upsert_for_session(&session.id).await;

    assert!(crate::commands::set_session_fast_mode(session.id.clone(), true)
        .await
        .expect("document save remains authoritative"));
    let reloaded = crate::commands::get_agent_session(session.id.clone())
        .await
        .expect("reload document");
    let listed = crate::commands::list_agent_sessions()
        .await
        .expect("reconcile derived index");

    assert!(reloaded.fast_mode_enabled);
    assert!(listed
        .iter()
        .find(|meta| meta.id == session.id)
        .expect("reconciled metadata")
        .fast_mode_enabled);
    delete_sessions(&[&session.id]).await;
}

#[tokio::test]
async fn rebuilt_index_copies_fast_mode_from_the_session_document() {
    let root = tempfile::tempdir().expect("tempdir");
    let mut session = serde_json::from_value::<super::types_session::AgentSession>(
        legacy_session_json(),
    )
    .expect("deserialize session");
    session.fast_mode_enabled = true;
    session_store::write_to_dir(root.path(), &session)
        .await
        .expect("persist session document");

    let rebuilt = session_index::rebuild_index_from(root.path())
        .await
        .expect("rebuild index");

    assert_eq!(rebuilt.len(), 1);
    assert!(rebuilt[0].fast_mode_enabled);
}
