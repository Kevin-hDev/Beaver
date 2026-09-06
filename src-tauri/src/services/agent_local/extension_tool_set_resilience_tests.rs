use super::{native_only_for_session, ExtensionToolSet, PrepareContext};
use crate::services::extensions::{error_codes, CatalogSnapshot};
use serde_json::json;

fn context(session_id: &str) -> PrepareContext<'_> {
    PrepareContext {
        session_id,
        provider: "deepseek",
        model: "deepseek-chat",
        context_window: 128_000,
        preserve_dynamic_tools: false,
    }
}

#[tokio::test]
async fn refused_registry_preserves_native_policy_and_canonical_schemas() {
    for code in [
        error_codes::REGISTRY_UNAVAILABLE,
        error_codes::REGISTRY_VERSION_UNSUPPORTED,
        error_codes::REGISTRY_MIGRATION_FAILED,
    ] {
        let id = uuid::Uuid::new_v4().to_string();
        let mut tools = ExtensionToolSet::prepare_with_registry(
            vec![
                json!({"function":{"name":"read_file","parameters":{"extensionOverride":true}}}),
                json!({"function":{"name":"unknown_dynamic"}}),
                json!({"function":{"name":"list_extensions"}}),
                json!({"function":{"name":"inspect_extensions"}}),
                json!({"function":{"name":"load_extension_resource"}}),
            ],
            context(&id),
            Err(code),
        )
        .await
        .unwrap();
        let canonical = crate::services::agent_local::tool_definitions::native_tool_definitions()
            .into_iter()
            .find(|tool| tool["function"]["name"] == "read_file")
            .unwrap();
        assert_eq!(tools.active(), &[canonical]);
        assert_eq!(tools.degradation, Some(code));
        assert!(native_only_for_session(&id));
        assert!(!state_path(&id).exists());
        let before = tools.active().to_vec();
        tools.refresh_from_session(&id).await.unwrap();
        assert_eq!(tools.active(), before);
        assert!(
            !crate::services::agent_local::extension_session_plugins::is_tool_active(
                &id,
                "read_file"
            )
            .await
            .unwrap()
        );
        let resource =
            crate::services::agent_local::tool_extension_resource::execute(&json!({}), &id).await;
        assert!(resource.is_error);
        drop(tools);
        assert!(!native_only_for_session(&id));
    }
}

fn state_path(id: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("extension-session-state")
        .join(format!("{id}.json"))
}

#[tokio::test]
async fn session_store_write_failure_keeps_native_tools_and_closes_extensions() {
    let id = uuid::Uuid::new_v4().to_string();
    let path = state_path(&id);
    std::fs::create_dir_all(&path).unwrap();
    let catalog = CatalogSnapshot {
        version: "a".repeat(64),
        ..Default::default()
    };
    let result = ExtensionToolSet::prepare_with_registry(
        vec![json!({"function":{"name":"read_file"}})],
        context(&id),
        Ok(catalog),
    )
    .await;
    std::fs::remove_dir(&path).unwrap();
    let tools = result.unwrap();
    assert_eq!(tools.degradation, Some(error_codes::STATE_UNAVAILABLE));
    assert_eq!(tools.active().len(), 1);
    assert!(native_only_for_session(&id));
    drop(tools);
    assert!(!native_only_for_session(&id));
}

#[tokio::test]
async fn invalid_admission_is_not_hidden_by_extension_fallback() {
    assert!(ExtensionToolSet::prepare_with_registry(
        vec![],
        context("../invalid"),
        Err(error_codes::REGISTRY_UNAVAILABLE)
    )
    .await
    .is_err());
    let id = uuid::Uuid::new_v4().to_string();
    let mut context = context(&id);
    context.provider = "unknown-provider";
    assert!(ExtensionToolSet::prepare_with_registry(
        vec![],
        context,
        Err(error_codes::REGISTRY_UNAVAILABLE)
    )
    .await
    .is_err());
    assert!(!native_only_for_session(&id));
}

#[test]
fn native_fallback_respects_provider_capacity_and_empty_admission() {
    let id = uuid::Uuid::new_v4().to_string();
    let native = crate::services::agent_local::tool_definitions::native_tool_definitions();
    for limit in [0, 1, 3] {
        let tools =
            ExtensionToolSet::degraded(native.clone(), limit, error_codes::STATE_UNAVAILABLE, &id)
                .unwrap();
        assert_eq!(tools.active().len(), limit);
    }
    let tools =
        ExtensionToolSet::degraded(vec![], 100, error_codes::STATE_UNAVAILABLE, &id).unwrap();
    assert!(tools.active().is_empty());
}

#[tokio::test]
async fn degraded_turn_reaches_transport_returns_text_and_persists_its_cause() {
    use crate::services::agent_local::{
        session_store, stream_diagnostics, stream_events::AgentEventEmitter,
        types_ollama::ChatMessage,
    };
    use crate::services::llm::{
        fast_mode::FastModeRequest, request_purpose::RequestPurpose,
        stream_test_transport::StreamScenario,
    };
    let session = session_store::create_full(
        "extension resilience fixture",
        "gpt-5.6-luna",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let request_id = stream_diagnostics::start_request(&session.id, 1).await;
    let tools = ExtensionToolSet::prepare_with_registry(
        vec![
            json!({"function":{"name":"read_file"}}),
            json!({"function":{"name":"unknown_dynamic"}}),
        ],
        PrepareContext {
            provider: "openai",
            model: "gpt-5.6-luna",
            ..context(&session.id)
        },
        Err(error_codes::REGISTRY_VERSION_UNSUPPORTED),
    )
    .await
    .unwrap();
    let emitter = AgentEventEmitter::test(session.id.clone());
    tools
        .report_prepared(&emitter, &session.id, &request_id)
        .await
        .unwrap();
    let scenario = StreamScenario::start_with_fragments(&session.id, vec![
        json!({"type":"response.output_text.delta","delta":"Bonjour"}).to_string(),
        json!({"type":"response.completed","response":{"usage":{"input_tokens":1,"output_tokens":1}}}).to_string(),
    ]).await.unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        crate::services::llm::stream::stream_chat_no_done(
            &emitter,
            &session.id,
            &request_id,
            1,
            1,
            "openai",
            FastModeRequest::Standard,
            RequestPurpose::ManualChat,
            "gpt-5.6-luna",
            &[ChatMessage::user("salut".into())],
            tools.active(),
            false,
            None,
            &Default::default(),
            tokio_util::sync::CancellationToken::new(),
            true,
            None,
            None,
            None,
        ),
    )
    .await;
    let persisted = session_store::get(&session.id).await.unwrap();
    let payloads = scenario.payloads();
    session_store::delete_one(&session.id).await.unwrap();
    assert_eq!(result.unwrap().unwrap().into_result().content, "Bonjour");
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["tools"].as_array().unwrap().len(), 1);
    assert_eq!(payloads[0]["tools"][0]["name"], "read_file");
    assert!(persisted
        .diagnostic_runs
        .iter()
        .any(|run| run.request_id == request_id
            && run.events.iter().any(|event| event.error_type.as_deref()
                == Some(error_codes::REGISTRY_VERSION_UNSUPPORTED))));
}

#[tokio::test]
async fn ollama_preparation_also_degrades_without_extensions() {
    let id = uuid::Uuid::new_v4().to_string();
    let tools = ExtensionToolSet::prepare_with_registry(
        vec![json!({"function":{"name":"read_file"}})],
        PrepareContext {
            provider: "ollama",
            model: "qwen3:8b",
            ..context(&id)
        },
        Err(error_codes::REGISTRY_UNAVAILABLE),
    )
    .await
    .unwrap();
    assert_eq!(tools.active().len(), 1);
    assert_eq!(tools.degradation, Some(error_codes::REGISTRY_UNAVAILABLE));
    assert!(matches!(
        crate::services::agent_local::extension_skill_loader::load_skill_for_session(
            "extension:example:skill",
            &id
        )
        .await,
        Err(crate::services::agent_local::tool_skill_loader::SkillLoadError::Unavailable)
    ));
}

#[tokio::test]
async fn degraded_turn_stops_if_the_conversation_itself_cannot_be_saved() {
    use crate::services::agent_local::{
        session_store, stream_diagnostics, stream_events::AgentEventEmitter,
    };
    let session = session_store::create_full(
        "journal failure fixture",
        "gpt-5.6-luna",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    let request = stream_diagnostics::start_request(&session.id, 1).await;
    let tools =
        ExtensionToolSet::degraded(vec![], 0, error_codes::STATE_UNAVAILABLE, &session.id).unwrap();
    let target = crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{}.json", session.id));
    std::fs::remove_file(&target).unwrap();
    std::fs::create_dir(&target).unwrap();
    let result = tools
        .report_prepared(
            &AgentEventEmitter::test(session.id.clone()),
            &session.id,
            &request,
        )
        .await;
    let guard_held = native_only_for_session(&session.id);
    std::fs::remove_dir(&target).unwrap();
    session_store::save(&session).await.unwrap();
    session_store::delete_one(&session.id).await.unwrap();
    assert!(result.is_err());
    assert!(guard_held);
}
