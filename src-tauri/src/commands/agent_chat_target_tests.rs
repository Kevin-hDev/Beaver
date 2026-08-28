use super::*;

fn build_with_scope(
    model: &str,
    persisted_mode: Option<&str>,
    route_id: RouteId,
    credential_scope: CredentialScope,
) -> Result<ReplayTarget, String> {
    let reasoning_mode = ReasoningModeId::from_name(persisted_mode).ok_or_else(generic_error)?;
    build_with_mode(model, route_id, credential_scope, reasoning_mode)
}

fn reasoning_mode_id(mode: Option<&str>) -> Result<ReasoningModeId, String> {
    ReasoningModeId::from_name(mode).ok_or_else(generic_error)
}

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
        target.reasoning.ollama_payload.as_ref(),
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
async fn disabled_required_route_resolves_as_forbidden_until_live_validated() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Disabled required route",
        "gemini-3.7-flash",
        "google",
        true,
        None,
    )
    .await
    .unwrap();
    session.reasoning_mode = Some("medium".into());
    session.thinking_enabled = true;
    let session_id = session.id.clone();
    let target = resolve_session(
        session,
        RouteId::Google,
        None,
        Some(true),
        Some(CredentialScope::authenticated("test-scope").unwrap()),
    )
    .unwrap();

    assert!(matches!(
        target.continuation,
        ContinuationTarget::Forbidden(_)
    ));
    cleanup(&session_id).await;
}

#[tokio::test]
async fn api_legacy_thinking_never_resolves_off_before_admission() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "API legacy thinking",
        "deepseek-v4-flash",
        "deepseek",
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

    let target = resolve_with_api_capability(&session.id, "deepseek", "deepseek-v4-flash", true)
        .await
        .unwrap();

    assert!(target.reasoning.active);
    assert_ne!(target.continuation.reasoning_mode(), ReasoningModeId::Off);
    assert_eq!(
        target.reasoning.mode_name.as_deref(),
        Some("high"),
        "la cible et le runtime doivent partager le défaut canonique"
    );
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
    assert!(target.reasoning.active);
    assert_eq!(target.reasoning.mode_name.as_deref(), Some("auto"));
    assert_eq!(target.continuation.reasoning_mode(), ReasoningModeId::Auto);
    assert_eq!(
        target.reasoning.ollama_payload.as_ref(),
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
    let effective = &target.reasoning;
    assert_eq!(
        target.continuation.reasoning_mode(),
        ReasoningModeId::Medium
    );
    assert_eq!(effective.mode, ReasoningModeId::Medium);
    assert_eq!(
        effective.ollama_payload,
        Some(crate::services::agent_local::types_ollama::OllamaThink::Level("medium".into()))
    );
    cleanup(&session.id).await;
}

#[tokio::test]
async fn ollama_mode_is_identical_from_resolution_through_durable_admission() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Ollama canonical flow",
        "qwen3.5:4b",
        "ollama",
        true,
        None,
    )
    .await
    .unwrap();
    session.reasoning_mode = Some("high".into());
    session.thinking_enabled = true;
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let target = resolve_with_ollama_capabilities(
        &session.id,
        "ollama",
        "qwen3.5:4b",
        None,
        None,
        &["completion".into(), "thinking".into()],
    )
    .await
    .unwrap();

    crate::commands::agent_chat_task::validate_target_profile(
        "ollama",
        "qwen3.5:4b",
        &target.continuation,
        &target.reasoning,
        target.reasoning.active,
        target.reasoning.mode_name.as_deref(),
    )
    .unwrap();
    admit_resolved(&session.id, &target).await;

    let stored = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(stored.reasoning_mode.as_deref(), Some("auto"));
    assert!(stored.messages[0].replay_source.is_none());
    assert_eq!(
        target.reasoning.ollama_payload,
        Some(crate::services::agent_local::types_ollama::OllamaThink::Bool(true))
    );
    cleanup(&session.id).await;
}

#[tokio::test]
async fn api_mode_is_identical_from_resolution_through_payload_and_provenance() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "API canonical flow",
        "deepseek-v4-flash",
        "deepseek",
        true,
        None,
    )
    .await
    .unwrap();
    session.reasoning_mode = None;
    session.thinking_enabled = true;
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let target = resolve_with_api_capability(&session.id, "deepseek", "deepseek-v4-flash", true)
        .await
        .unwrap();

    crate::commands::agent_chat_task::validate_target_profile(
        "deepseek",
        "deepseek-v4-flash",
        &target.continuation,
        &target.reasoning,
        target.reasoning.active,
        target.reasoning.mode_name.as_deref(),
    )
    .unwrap();
    admit_resolved(&session.id, &target).await;
    let mut payload = serde_json::json!({});
    crate::services::llm::stream_reasoning::apply(
        &mut payload,
        crate::services::llm::route_profile::payload_policy("deepseek", "deepseek-v4-flash")
            .unwrap()
            .parameters,
        "deepseek-v4-flash",
        target.reasoning.active,
        target.reasoning.mode_name.as_deref(),
    );

    let stored = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(stored.reasoning_mode.as_deref(), Some("high"));
    assert!(matches!(target.continuation, ContinuationTarget::Replay(_)));
    assert!(stored.messages[0].replay_source.is_none());
    assert_eq!(payload["thinking"]["type"], "enabled");
    assert_eq!(payload["reasoning_effort"], "high");
    cleanup(&session.id).await;
}

#[tokio::test]
async fn disabled_deepseek_resume_does_not_store_replay_provenance() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Resume provenance",
        "deepseek-v4-flash",
        "deepseek",
        true,
        None,
    )
    .await
    .unwrap();
    session.reasoning_mode = Some("low".into());
    session.thinking_enabled = true;
    let session_id = session.id.clone();
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let old = resolve_session(
        session,
        RouteId::DeepSeek,
        None,
        Some(true),
        Some(CredentialScope::authenticated("old-scope").unwrap()),
    )
    .unwrap();
    let admitted = admit_resolved(&session_id, &old).await;

    crate::services::agent_local::session_ops::edit_user_message(
        &session_id,
        crate::models::agent_session_contract::EditUserMessageInput {
            message_id: admitted.user_message_id.clone(),
            new_content: "edited retry".into(),
        },
    )
    .await
    .unwrap();
    let mut changed = crate::services::agent_local::session_store::get(&session_id)
        .await
        .unwrap();
    changed.reasoning_mode = Some("high".into());
    crate::services::agent_local::session_store::save(&changed)
        .await
        .unwrap();
    let current = resolve_session(
        changed,
        RouteId::DeepSeek,
        None,
        Some(true),
        Some(CredentialScope::authenticated("new-scope").unwrap()),
    )
    .unwrap();
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(&session_id).await;
    crate::services::agent_local::conversation_resume::resume_with_lease_and_reasoning(
        &lease,
        crate::models::agent_turn_contract::ResumeTurnInput {
            message_id: admitted.user_message_id.clone(),
        },
        current.continuation,
        &current.session_reasoning,
    )
    .await
    .unwrap();

    let stored = crate::services::agent_local::session_store::get(&session_id)
        .await
        .unwrap();
    assert!(stored.messages.last().unwrap().replay_source.is_none());
    assert_eq!(stored.messages.last().unwrap().content, "edited retry");
    cleanup(&session_id).await;
}

#[tokio::test]
async fn forbidden_resume_erases_stale_replay_source() {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Forbidden resume",
        "deepseek-v4-flash",
        "deepseek",
        true,
        None,
    )
    .await
    .unwrap();
    session.reasoning_mode = Some("high".into());
    session.thinking_enabled = true;
    let session_id = session.id.clone();
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    let replay = resolve_session(
        session,
        RouteId::DeepSeek,
        None,
        Some(true),
        Some(CredentialScope::authenticated("old-scope").unwrap()),
    )
    .unwrap();
    let admitted = admit_resolved(&session_id, &replay).await;
    let mut changed = crate::services::agent_local::session_store::get(&session_id)
        .await
        .unwrap();
    changed.provider = "google".into();
    changed.model = "gemini-3.7-flash".into();
    changed.reasoning_mode = Some("medium".into());
    crate::services::agent_local::session_store::save(&changed)
        .await
        .unwrap();
    let forbidden = resolve_session(
        changed,
        RouteId::Google,
        None,
        Some(true),
        Some(CredentialScope::authenticated("new-scope").unwrap()),
    )
    .unwrap();
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(&session_id).await;
    crate::services::agent_local::conversation_resume::resume_with_lease_and_reasoning(
        &lease,
        crate::models::agent_turn_contract::ResumeTurnInput {
            message_id: admitted.user_message_id,
        },
        forbidden.continuation,
        &forbidden.session_reasoning,
    )
    .await
    .unwrap();

    let stored = crate::services::agent_local::session_store::get(&session_id)
        .await
        .unwrap();
    assert!(stored.messages.last().unwrap().replay_source.is_none());
    cleanup(&session_id).await;
}

async fn admit_resolved(
    session_id: &str,
    target: &ResolvedChatTarget,
) -> crate::services::agent_local::conversation_admission::AdmittedTurn {
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
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(session_id).await;
    crate::services::agent_local::conversation_admission::new_turn_with_lease_and_reasoning(
        &lease,
        input,
        target.continuation.clone(),
        &target.session_reasoning,
    )
    .await
    .unwrap()
}

async fn cleanup(session_id: &str) {
    crate::services::agent_local::session_store::delete_one(session_id)
        .await
        .unwrap();
}
