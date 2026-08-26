use super::*;

#[test]
fn api_and_oauth_routes_keep_distinct_scopes_without_exposing_them() {
    let api = build_with_scope(
        "grok-4.6",
        Some("high"),
        RouteId::Xai,
        CredentialScope::authenticated("api-scope").unwrap(),
    )
    .unwrap();
    let oauth = build_with_scope(
        "grok-4.6",
        Some("high"),
        RouteId::XaiOauth,
        CredentialScope::authenticated("oauth-scope").unwrap(),
    )
    .unwrap();

    assert_eq!(api.route_id, RouteId::Xai);
    assert_eq!(oauth.route_id, RouteId::XaiOauth);
    assert_ne!(api.credential_scope, oauth.credential_scope);
    assert_eq!(api.reasoning_mode, ReasoningModeId::High);
}

#[test]
fn unknown_routes_and_modes_fail_closed() {
    assert!(RouteId::from_provider_id("forged").is_none());
    assert!(reasoning_mode_id(Some("forged")).is_err());
}

#[tokio::test]
async fn forged_frontend_hints_cannot_change_persisted_reasoning_mode() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Canonical target",
        "qwen3.5:4b",
        "ollama",
        true,
        None,
    )
    .await
    .unwrap();
    session.thinking_enabled = true;
    session.reasoning_mode = Some("high".to_string());
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();

    let target = resolve_with_ollama_capabilities(
        &session.id,
        "ollama",
        "qwen3.5:4b",
        Some("off"),
        Some(false),
        &["completion".into(), "thinking".into()],
    )
    .await
    .unwrap();
    assert_eq!(target.continuation.reasoning_mode(), ReasoningModeId::Auto);
    assert_eq!(
        target.ollama_reasoning.as_ref().map(|value| &value.payload),
        Some(&crate::services::agent_local::types_ollama::OllamaThink::Bool(true))
    );
    assert!(resolve(
        &session.id,
        "xai-oauth",
        "qwen3.5:4b",
        Some("high"),
        Some(true),
    )
    .await
    .is_err());
    cleanup(&session.id).await;
}

#[tokio::test]
async fn groq_resolves_as_explicit_non_replay_without_a_credential_scope() {
    let session = crate::services::agent_local::session_store::create_full(
        "Groq no replay",
        "openai/gpt-oss-120b",
        "groq",
        false,
        None,
    )
    .await
    .unwrap();
    let target = resolve(
        &session.id,
        "groq",
        "openai/gpt-oss-120b",
        Some("high"),
        Some(true),
    )
    .await
    .unwrap();
    assert!(matches!(
        target.continuation,
        ContinuationTarget::Forbidden(_)
    ));
    let input = crate::services::agent_local::conversation_input::resolve_with_key(
        crate::models::agent_turn_contract::NewUserTurnInput {
            content: "continue".into(),
            files: Vec::new(),
            skills: Vec::new(),
        },
        &[],
    )
    .await
    .unwrap();
    let admitted = crate::services::agent_local::conversation_admission::new_turn_for_continuation(
        &session.id,
        input,
        target.continuation,
    )
    .await
    .unwrap();
    assert!(admitted
        .history
        .messages
        .iter()
        .all(|message| message.continuation.is_none()));
    let stored = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert!(stored.messages[0].replay_source.is_none());
    assert!(stored.messages[0].continuation.is_none());
    cleanup(&session.id).await;
}

#[tokio::test]
async fn legacy_thinking_enabled_without_mode_keeps_implicit_runtime_mode() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Legacy thinking",
        "qwen3.5:4b",
        "ollama",
        true,
        None,
    )
    .await
    .unwrap();
    session.thinking_enabled = true;
    session.reasoning_mode = None;
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();

    let target = resolve_with_ollama_capabilities(
        &session.id,
        "ollama",
        "qwen3.5:4b",
        None,
        None,
        &["thinking".into()],
    )
    .await
    .unwrap();
    assert!(target.think);
    assert_eq!(target.reasoning_mode.as_deref(), Some("auto"));
    assert_eq!(target.continuation.reasoning_mode(), ReasoningModeId::Auto);
    assert_eq!(
        target.ollama_reasoning.as_ref().map(|value| &value.payload),
        Some(&crate::services::agent_local::types_ollama::OllamaThink::Bool(true))
    );
    cleanup(&session.id).await;
}

#[tokio::test]
async fn legacy_gpt_oss_uses_medium_for_provenance_and_transport() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Legacy GPT OSS",
        "gpt-oss:20b",
        "ollama",
        true,
        None,
    )
    .await
    .unwrap();
    session.thinking_enabled = true;
    session.reasoning_mode = None;
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();

    let target = resolve_with_ollama_capabilities(
        &session.id,
        "ollama",
        "gpt-oss:20b",
        None,
        None,
        &["thinking".into()],
    )
    .await
    .unwrap();
    let effective = target.ollama_reasoning.as_ref().unwrap();
    assert_eq!(
        target.continuation.reasoning_mode(),
        ReasoningModeId::Medium
    );
    assert_eq!(effective.mode, ReasoningModeId::Medium);
    assert_eq!(
        effective.payload,
        crate::services::agent_local::types_ollama::OllamaThink::Level("medium".into())
    );
    cleanup(&session.id).await;
}

async fn cleanup(session_id: &str) {
    crate::services::agent_local::session_store::delete_one(session_id)
        .await
        .unwrap();
}
