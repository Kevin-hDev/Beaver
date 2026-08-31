use std::path::PathBuf;

use serde_json::json;

use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

use super::session_limits::MAX_SESSION_FILE_BYTES;
use super::types_session::{
    AgentMessage, ToolActivityRecord, ToolCallRequest, ToolCallRequestFunction,
};

const WRITER_COMMIT: &str = "2848a17e87fa641bff067dc4b5c9a2398bae6540";
const V1_FIXTURE: &[u8] = include_bytes!("../../../test-fixtures/agent-session-v1-synthetic.json");
const SYNTHETIC_TOOL_CHAIN: &[u8] =
    include_bytes!("../../../test-fixtures/agent-session-v1-synthetic-tool-chain.json");
const V2_COMPRESSION_FIXTURE: &[u8] =
    include_bytes!("../../../test-fixtures/agent-session-v2-compression.json");
const V3_COMPRESSION_FIXTURE: &[u8] = include_bytes!("fixtures/session-v3-compression.json");

#[tokio::test]
async fn v3_migrates_to_v4_with_an_empty_guard_and_exact_backup() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000030.json");
    crate::services::private_store::atomic_write(&path, V3_COMPRESSION_FIXTURE)
        .expect("seed v3 fixture");

    let loaded = super::session_migration::read(V3_COMPRESSION_FIXTURE, path.clone())
        .expect("migrate v3 fixture");

    assert_eq!(
        loaded.version(),
        super::session_migration::LoadedVersion::V3
    );
    assert_eq!(loaded.session().schema_version, 4);
    assert!(loaded.session().automatic_compression_guard.is_empty());
    assert_eq!(loaded.session().compression_count, 2);
    assert_eq!(loaded.session().messages.len(), 3);
    super::session_migration::commit_current(&loaded)
        .await
        .expect("publish v4");
    let backup = super::session_migration::v3_backup_path(&path).expect("v3 backup path");
    assert_eq!(std::fs::read(&backup).unwrap(), V3_COMPRESSION_FIXTURE);

    let current = std::fs::read(&path).unwrap();
    let reloaded = super::session_migration::read(&current, path).expect("reload v4");
    assert_eq!(
        reloaded.version(),
        super::session_migration::LoadedVersion::V4
    );
    assert!(reloaded.session().automatic_compression_guard.is_empty());
}

#[test]
fn invalid_automatic_compression_guard_does_not_make_the_session_unreadable() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000031.json");
    let migrated = super::session_migration::read(V3_COMPRESSION_FIXTURE, path.clone())
        .expect("migrate v3 fixture");
    let mut value = serde_json::to_value(migrated.session()).expect("session json");
    value["automatic_compression_guard"] = serde_json::json!({
        "last_attempt": null,
        "consecutive_failures": 255,
        "suspended": true
    });
    let bytes = serde_json::to_vec(&value).expect("session bytes");

    let loaded = super::session_migration::read(&bytes, path).expect("guard degrades safely");

    assert!(loaded.session().automatic_compression_guard.is_empty());
    assert_eq!(
        serde_json::to_vec(&loaded.session().messages).unwrap(),
        serde_json::to_vec(&migrated.session().messages).unwrap()
    );
}

#[tokio::test]
async fn v2_compression_markers_migrate_to_v3_with_an_exact_backup() {
    use super::types_message::AgentMessageKind;

    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000020.json");
    crate::services::private_store::atomic_write(&path, V2_COMPRESSION_FIXTURE)
        .expect("seed v2 fixture");
    let loaded = super::session_migration::read(V2_COMPRESSION_FIXTURE, path.clone())
        .expect("migrate v2 fixture");

    assert_eq!(
        loaded.version(),
        super::session_migration::LoadedVersion::V2
    );
    assert_eq!(loaded.session().schema_version, 4);
    assert_eq!(
        loaded.session().messages[0].message_kind,
        Some(AgentMessageKind::CompressionCheckpoint)
    );
    assert_eq!(
        loaded.session().messages[1].message_kind,
        Some(AgentMessageKind::CompressionBoundary)
    );
    assert!(loaded.session().messages[2].message_kind.is_none());
    assert!(loaded.session().compression_profile_selection.is_none());
    assert_eq!(loaded.session().compression_count, 0);

    super::session_migration::commit_current(&loaded)
        .await
        .expect("publish v4");
    let backup = super::session_migration::v2_backup_path(&path).expect("v2 backup path");
    assert_eq!(std::fs::read(&backup).unwrap(), V2_COMPRESSION_FIXTURE);
    let current = std::fs::read(&path).unwrap();
    let reloaded = super::session_migration::read(&current, path).expect("reload v4");
    assert_eq!(
        reloaded.version(),
        super::session_migration::LoadedVersion::V4
    );
    assert_eq!(reloaded.session().schema_version, 4);
}

#[test]
fn synthetic_v1_fixture_keeps_visible_thinking_without_promoting_continuation() {
    assert_eq!(WRITER_COMMIT.len(), 40);

    let loaded = super::session_migration::read(V1_FIXTURE, PathBuf::from("fixture.json"))
        .expect("load synthetic v1 fixture");

    assert_eq!(
        loaded.session().schema_version,
        super::session_limits::CURRENT_SESSION_SCHEMA_VERSION
    );
    assert_eq!(
        loaded.session().messages[1].thinking.as_deref(),
        Some("fixture-visible-thinking")
    );
    assert!(loaded.session().messages[1].continuation.is_none());
}

#[test]
fn v1_compression_markers_are_classified_before_the_session_becomes_v3() {
    use super::types_message::AgentMessageKind;

    for checkpoint in [
        "This session is being continued from a previous conversation\nsummary",
        "Recent file context preserved across compression:\nfiles",
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(V1_FIXTURE).unwrap();
        value["messages"][0]["content"] = json!(checkpoint);
        value["messages"][1]["content"] =
            json!("[Compression boundary — previous messages have been summarized]");
        let bytes = serde_json::to_vec(&value).unwrap();

        let loaded =
            super::session_migration::read(&bytes, PathBuf::from("v1-compression-marker.json"))
                .expect("migrate v1 compression markers");

        assert_eq!(
            loaded.session().messages[0].message_kind,
            Some(AgentMessageKind::CompressionCheckpoint)
        );
        assert_eq!(
            loaded.session().messages[1].message_kind,
            Some(AgentMessageKind::CompressionBoundary)
        );
    }
}

#[test]
fn synthetic_v1_tool_chain_migrates_without_promoting_codex_sidecars() {
    let loaded = super::session_migration::read(
        SYNTHETIC_TOOL_CHAIN,
        PathBuf::from("synthetic-tool-chain.json"),
    )
    .expect("load synthetic v1 fixture");
    let session = loaded.session();

    assert_eq!(
        session.schema_version,
        super::session_limits::CURRENT_SESSION_SCHEMA_VERSION
    );
    assert!(session
        .messages
        .iter()
        .any(|message| message.role == "tool"));
    assert!(session
        .messages
        .iter()
        .all(|message| message.continuation.is_none()));
    assert!(session
        .messages
        .iter()
        .all(|message| message.replay_source.is_none()));
}

#[test]
fn v1_migration_merges_consecutive_users_and_keeps_a_valid_history() {
    let mut value: serde_json::Value = serde_json::from_slice(V1_FIXTURE).unwrap();
    let messages = value["messages"].as_array_mut().unwrap();
    messages.insert(1, messages[0].clone());
    let bytes = serde_json::to_vec(&value).unwrap();

    let loaded = super::session_migration::read(&bytes, PathBuf::from("users-v1.json")).unwrap();

    super::conversation_history_validation::validate(&loaded.session().messages).unwrap();
    assert_ne!(
        loaded.session().messages[0].role,
        loaded.session().messages[1].role
    );
}

#[test]
fn v1_migration_discards_an_invalid_leading_assistant_but_keeps_valid_turns() {
    let mut value: serde_json::Value = serde_json::from_slice(V1_FIXTURE).unwrap();
    let messages = value["messages"].as_array_mut().unwrap();
    messages.insert(0, messages[1].clone());
    let bytes = serde_json::to_vec(&value).unwrap();

    let loaded = super::session_migration::read(&bytes, PathBuf::from("leading-v1.json")).unwrap();

    super::conversation_history_validation::validate(&loaded.session().messages).unwrap();
    assert_eq!(loaded.session().messages.first().unwrap().role, "user");
}

#[test]
fn v1_incomplete_tool_chain_keeps_history_and_closes_every_missing_result() {
    let bytes = v1_with_incomplete_tool_chain();
    let loaded =
        super::session_migration::read(&bytes, PathBuf::from("incomplete-tool-chain-v1.json"))
            .expect("repair interrupted v1 tool chain");
    let messages = &loaded.session().messages;

    assert!(!messages.is_empty());
    super::conversation_history_validation::validate(messages).unwrap();
    let call_message = messages
        .iter()
        .find(|message| message.tool_calls.is_some())
        .expect("preserved tool call");
    let calls = call_message.tool_calls.as_ref().unwrap();
    for call in calls {
        let result = messages
            .iter()
            .find(|message| message.tool_call_id.as_deref() == Some(call.id.as_str()))
            .expect("synthetic interrupted result");
        assert_eq!(result.role, "tool");
        assert_eq!(
            result.tool_name.as_deref(),
            Some(call.function.name.as_str())
        );
        assert_eq!(
            result.content,
            r#"{"status":"cancelled","error":"tool_interrupted"}"#
        );
    }
    assert_eq!(messages.last().unwrap().role, "assistant");
    assert!(messages.last().unwrap().tool_calls.is_none());
}

#[tokio::test]
async fn empty_v2_never_acknowledges_a_nonempty_v1_backup() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000010.json");
    let backup = super::session_migration::backup_path(&path).unwrap();
    let mut empty_v2 = base_session();
    empty_v2.messages.clear();
    let empty_bytes = serde_json::to_vec_pretty(&empty_v2).unwrap();
    let interrupted_v1 = v1_with_incomplete_tool_chain();
    crate::services::private_store::atomic_write(&path, &empty_bytes).unwrap();
    crate::services::private_store::atomic_write(&backup, &interrupted_v1).unwrap();

    let restored = super::session_store_document::read_from_path(path)
        .await
        .expect("empty v2 remains readable");

    assert!(restored.messages.is_empty());
    assert_eq!(std::fs::read(backup).unwrap(), interrupted_v1);
}

#[tokio::test]
async fn migrated_v1_history_builds_a_required_moonshot_payload_after_admission() {
    use crate::services::llm::fast_mode::FastModeRequest;
    use crate::services::reasoning_continuity::contract::{
        ContinuationTarget, ContinuationUse, ReplayTarget,
    };

    let loaded = super::session_migration::read(
        SYNTHETIC_TOOL_CHAIN,
        PathBuf::from("migration-payload-v1.json"),
    )
    .unwrap();
    let mut session = loaded.into_session();
    session.id = uuid::Uuid::new_v4().to_string();
    session.provider = "moonshot".into();
    session.model = "kimi-k2.7-code".into();
    session.reasoning_mode = Some("auto".into());
    session.thinking_enabled = true;
    super::session_store::save(&session).await.unwrap();
    let replay = ReplayTarget {
        route_id: RouteId::Moonshot,
        model_id: session.model.clone(),
        credential_scope: CredentialScope::authenticated("migration-fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    };
    let target = ContinuationTarget::Replay(replay);
    let admitted = super::conversation_admission::new_turn_for_continuation(
        &session.id,
        super::conversation_input::ResolvedTurnInput {
            user_content: "continue".into(),
            provider_content: "continue".into(),
            files: Vec::new(),
            images: Vec::new(),
            skills: Vec::new(),
        },
        target.clone(),
    )
    .await
    .unwrap();
    let messages = admitted
        .history
        .messages
        .into_iter()
        .map(crate::commands::agent_chat_task::convert_provider_message_for_test)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let config = crate::services::llm::RequestConfigForTest {
        provider_id: "moonshot",
        model: "kimi-k2.7-code",
        messages: &messages,
        tools: &[],
        think: true,
        reasoning_mode: Some("auto"),
        max_tokens: None,
        purpose: crate::services::llm::request_purpose::RequestPurpose::ManualChat,
        session_id: Some(&session.id),
        fast_mode: FastModeRequest::Unsupported,
        continuation_target: Some(&target),
    };

    let payload = crate::services::llm::build_chat_payload_for_test(
        &config,
        &crate::services::llm::route::resolve("moonshot").unwrap(),
        None,
    );
    super::session_store::delete_one(&session.id).await.unwrap();

    assert!(payload.is_ok());
}

#[tokio::test]
async fn v2_writer_rejects_invalid_private_skill_links_but_accepts_legacy_names() {
    let root = tempfile::tempdir().unwrap();
    let invalid = [
        (vec!["../forged"], vec!["Skill"]),
        (vec!["local:same", "local:same"], vec!["One", "Two"]),
        (vec!["local:one"], vec!["One", "Two"]),
    ];
    for (index, (ids, names)) in invalid.into_iter().enumerate() {
        let mut session = base_session();
        session.messages[0].skill_ids = Some(ids.into_iter().map(str::to_string).collect());
        session.messages[0].skill_names = Some(names.into_iter().map(str::to_string).collect());
        let path = root.path().join(format!("invalid-{index}.json"));
        assert!(super::session_store_document::write_to_path(path, &session)
            .await
            .is_err());
    }

    let mut legacy = base_session();
    legacy.messages[0].skill_names = Some(vec![String::new(), "../legacy-visible".into()]);
    legacy.messages[0].skill_ids = None;
    super::session_store_document::write_to_path(root.path().join("legacy.json"), &legacy)
        .await
        .expect("legacy names remain writable");
}

#[tokio::test]
async fn v2_writer_validates_private_turn_provenance_and_keeps_legacy_absence() {
    let root = tempfile::tempdir().unwrap();
    let valid_source = ReasoningSource {
        route_id: RouteId::Ollama,
        model_id: "fixture-model".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
    };
    let mut valid = base_session();
    valid.messages[0].replay_source = Some(valid_source.clone());
    let valid_path = root.path().join("valid-source.json");
    super::session_store_document::write_to_path(valid_path.clone(), &valid)
        .await
        .unwrap();
    let restored = super::session_store_document::read_from_path(valid_path)
        .await
        .unwrap();
    assert_eq!(restored.messages[0].replay_source, Some(valid_source));

    let mut wrong_role = base_session();
    wrong_role.messages[1].replay_source = Some(responses_envelope(Vec::new()).source);
    assert!(super::session_store_document::write_to_path(
        root.path().join("assistant-source.json"),
        &wrong_role,
    )
    .await
    .is_err());

    let mut wrong_scope = base_session();
    wrong_scope.messages[0].replay_source = Some(ReasoningSource {
        route_id: RouteId::Ollama,
        model_id: "fixture-model".into(),
        credential_scope: CredentialScope::authenticated("remote-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Auto,
    });
    assert!(super::session_store_document::write_to_path(
        root.path().join("wrong-scope.json"),
        &wrong_scope,
    )
    .await
    .is_err());

    let legacy = base_session();
    assert!(legacy
        .messages
        .iter()
        .all(|message| message.replay_source.is_none()));
    super::session_store_document::write_to_path(root.path().join("legacy-none.json"), &legacy)
        .await
        .unwrap();
}

#[tokio::test]
async fn legacy_tool_ids_are_local_linked_and_stable_after_commit() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000002.json");
    let bytes = v1_with_tool_chain();
    crate::services::private_store::atomic_write(&path, &bytes).expect("seed v1");
    let loaded = super::session_migration::read(&bytes, path.clone()).expect("load v1");
    let call_message = loaded
        .session()
        .messages
        .iter()
        .find(|message| message.tool_calls.is_some())
        .expect("migrated tool call");
    let call_id = call_message.tool_calls.as_ref().unwrap()[0].id.clone();
    assert!(super::session_migration::is_legacy_local_id(&call_id));
    assert_eq!(
        loaded
            .session()
            .messages
            .iter()
            .find(|message| message.role == "tool")
            .and_then(|message| message.tool_call_id.as_deref()),
        Some(call_id.as_str())
    );

    super::session_migration::commit_current(&loaded)
        .await
        .expect("commit v2");
    let backup = super::session_migration::backup_path(&path).expect("backup path");
    assert_eq!(std::fs::read(&backup).unwrap(), bytes);
    let persisted = super::session_store_document::read_from_path(path)
        .await
        .expect("reload v2");
    assert_eq!(
        persisted
            .messages
            .iter()
            .find(|message| message.tool_calls.is_some())
            .unwrap()
            .tool_calls
            .as_ref()
            .unwrap()[0]
            .id,
        call_id
    );
    assert!(!backup.exists());
}

#[tokio::test]
async fn legacy_messages_share_one_turn_until_the_next_user_message() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000008.json");
    let bytes = v1_with_two_turns();
    crate::services::private_store::atomic_write(&path, &bytes).expect("seed v1");
    let loaded = super::session_migration::read(&bytes, path.clone()).expect("load v1");
    let first_turn = loaded.session().messages[0].turn_id.clone();
    let second_start = loaded
        .session()
        .messages
        .iter()
        .position(|message| message.content == "fixture-second-user")
        .unwrap();
    let second_turn = loaded.session().messages[second_start].turn_id.clone();

    assert!(super::session_migration::is_legacy_local_id(&first_turn));
    assert!(super::session_migration::is_legacy_local_id(&second_turn));
    assert_ne!(first_turn, second_turn);
    assert!(loaded.session().messages[..second_start]
        .iter()
        .all(|message| message.turn_id == first_turn));
    assert!(loaded.session().messages[second_start..]
        .iter()
        .all(|message| message.turn_id == second_turn));

    super::session_migration::commit_current(&loaded)
        .await
        .expect("commit v2");
    let restored = super::session_store_document::read_from_path(path)
        .await
        .expect("reload v2");
    assert!(restored.messages[..second_start]
        .iter()
        .all(|message| message.turn_id == first_turn));
    assert!(restored.messages[second_start..]
        .iter()
        .all(|message| message.turn_id == second_turn));
}

#[tokio::test]
async fn v2_round_trip_keeps_order_tool_ids_and_opaque_envelope() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("session.json");
    let mut session = base_session();
    let envelope = responses_envelope(vec![json!({"type":"reasoning","index":1})]);
    let mut assistant = session.messages[1].clone();
    assistant.turn_id = "turn-provider-1".into();
    assistant.continuation = Some(envelope.clone());
    assistant.tool_calls = Some(vec![ToolCallRequest {
        id: "provider-call-1".into(),
        extra_content: Some(json!({"opaque":"fixture"})),
        function: ToolCallRequestFunction {
            name: "read_file".into(),
            arguments: json!({"path":"fixture.txt"}),
        },
    }]);
    session.messages = vec![session.messages[0].clone(), assistant];
    super::session_store_document::write_to_path(path.clone(), &session)
        .await
        .expect("write v2");

    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("future_root".into(), json!(true));
    value["messages"][1]
        .as_object_mut()
        .unwrap()
        .insert("future_message".into(), json!(42));
    crate::services::private_store::atomic_write(
        &path,
        &serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();

    let restored = super::session_store_document::read_from_path(path)
        .await
        .expect("read v2");
    assert_eq!(restored.messages[0].role, "user");
    assert_eq!(restored.messages[1].role, "assistant");
    assert_eq!(
        restored.messages[1].tool_calls.as_ref().unwrap()[0].id,
        "provider-call-1"
    );
    assert_eq!(restored.messages[1].continuation, Some(envelope));
}

#[tokio::test]
async fn writer_redacts_visible_text_without_mutating_opaque_state_or_provider_ids() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("session.json");
    let mut session = base_session();
    let opaque = responses_envelope(vec![json!({
        "type": "reasoning",
        "encrypted_content": "Bearer opaque-native-token-12345678",
        "provider_item_id": "sk-native-item-12345678"
    })]);
    let tool_extra = json!({
        "google": {"thought_signature": "Bearer opaque-tool-signature-12345678"},
        "codex": {"output_items": [{"id": "sk-output-item-12345678"}]}
    });
    let controlled_collisions = json!({
        "id": "sk-controlled-id-12345678",
        "continuation": "Bearer controlled-continuation-12345678",
        "extra_content": "aaaaaaaaaaaaaaaaaaaa.bbbbb.cccccccccccccccccccc",
        "provider_id": "sk-controlled-provider-id-12345678"
    });
    session.messages[1].content = "sk-visible-content-12345678".into();
    session.messages[1].id = "sk-message-id-12345678".into();
    session.messages[1].turn_id = "sk-turn-id-12345678".into();
    session.messages[1].tool_call_id = Some("sk-linked-call-id-12345678".into());
    session.messages[1].continuation = Some(opaque.clone());
    session.messages[1].tool_calls = Some(vec![ToolCallRequest {
        id: "sk-provider-call-12345678".into(),
        extra_content: Some(tool_extra.clone()),
        function: ToolCallRequestFunction {
            name: "read_file".into(),
            arguments: controlled_collisions.clone(),
        },
    }]);
    session.messages[1].tool_activities = Some(vec![ToolActivityRecord {
        name: "read_file".into(),
        summary: "fixture".into(),
        domain: None,
        resolved_path: None,
        args: Some(controlled_collisions.clone()),
        result: Some(controlled_collisions.to_string()),
        is_error: Some(false),
        result_meta: None,
        content: None,
        old_text: None,
        new_text: None,
        start_line: None,
        affected_paths: Vec::new(),
        file_changes: Vec::new(),
    }]);

    super::session_store_document::write_to_path(path.clone(), &session)
        .await
        .expect("write v2");
    let restored = super::session_store_document::read_from_path(path)
        .await
        .expect("read v2");

    assert_eq!(restored.messages[1].content, "[REDACTED]");
    assert_eq!(restored.messages[1].id, "sk-message-id-12345678");
    assert_eq!(restored.messages[1].turn_id, "sk-turn-id-12345678");
    assert_eq!(
        restored.messages[1].tool_call_id.as_deref(),
        Some("sk-linked-call-id-12345678")
    );
    assert_eq!(
        restored.messages[1].tool_calls.as_ref().unwrap()[0].id,
        "sk-provider-call-12345678"
    );
    assert_eq!(
        restored.messages[1].tool_calls.as_ref().unwrap()[0].extra_content,
        Some(serde_json::json!({ "google": tool_extra["google"].clone() }))
    );
    assert_eq!(restored.messages[1].continuation, Some(opaque));
    let arguments = &restored.messages[1].tool_calls.as_ref().unwrap()[0]
        .function
        .arguments;
    for key in ["id", "continuation", "extra_content", "provider_id"] {
        assert_eq!(arguments[key], "[REDACTED]");
    }
    let activity = &restored.messages[1].tool_activities.as_ref().unwrap()[0];
    let args = activity.args.as_ref().unwrap();
    for key in ["id", "continuation", "extra_content", "provider_id"] {
        assert_eq!(args[key], "[REDACTED]");
    }
    let result = activity.result.as_deref().unwrap();
    for secret in [
        "sk-controlled-id-12345678",
        "controlled-continuation-12345678",
        "aaaaaaaaaaaaaaaaaaaa.bbbbb.cccccccccccccccccccc",
        "sk-controlled-provider-id-12345678",
    ] {
        assert!(!result.contains(secret));
    }
    assert!(result.contains("[REDACTED]"));
}

#[tokio::test]
async fn future_session_is_visible_without_continuity_or_rewrite() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("future.json");
    let mut session = base_session();
    session.messages[1].continuation = Some(responses_envelope(vec![json!({"opaque":1})]));
    session.messages[0].replay_source = Some(ReasoningSource {
        route_id: RouteId::Ollama,
        model_id: "fixture-model".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
    });
    let mut value = serde_json::to_value(&session).unwrap();
    value["schema_version"] = json!(99);
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    crate::services::private_store::atomic_write(&path, &bytes).unwrap();

    let visible = super::session_store_document::read_from_path(path.clone())
        .await
        .expect("future visible");
    assert_eq!(visible.schema_version, 99);
    assert_eq!(visible.messages[1].content, "fixture-assistant-content");
    assert!(visible.messages[1].continuation.is_none());
    assert!(visible.messages[0].replay_source.is_none());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}

#[test]
fn invalid_envelope_is_dropped_without_hiding_visible_message() {
    let session = base_session();
    let mut value = serde_json::to_value(&session).unwrap();
    value["messages"][1]["continuation"] =
        serde_json::to_value(responses_envelope(vec![json!({"opaque":"fixture"})])).unwrap();
    value["messages"][1]["continuation"]["schema_version"] = json!(77);
    let bytes = serde_json::to_vec(&value).unwrap();

    let loaded = super::session_migration::read(&bytes, PathBuf::from("invalid.json"))
        .expect("visible v2 remains readable");
    assert_eq!(
        loaded.session().messages[1].content,
        "fixture-assistant-content"
    );
    assert!(loaded.session().messages[1].continuation.is_none());
}

#[test]
fn v2_rejects_missing_turn_and_tool_call_ids() {
    let session = base_session();
    let mut missing_turn = serde_json::to_value(&session).unwrap();
    missing_turn["messages"][0]
        .as_object_mut()
        .unwrap()
        .remove("turn_id");
    assert!(super::session_migration::read(
        &serde_json::to_vec(&missing_turn).unwrap(),
        PathBuf::from("missing-turn.json")
    )
    .is_err());

    let mut missing_call = serde_json::to_value(&session).unwrap();
    missing_call["messages"][1]["tool_calls"] = json!([{
        "function": {"name":"read_file","arguments":{"path":"fixture.txt"}}
    }]);
    assert!(super::session_migration::read(
        &serde_json::to_vec(&missing_call).unwrap(),
        PathBuf::from("missing-call.json")
    )
    .is_err());
}

#[tokio::test]
async fn injected_failure_before_rename_keeps_v1_and_exact_backup() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root
        .path()
        .join("00000000-0000-4000-8000-000000000002.json");
    crate::services::private_store::atomic_write(&path, V1_FIXTURE).expect("seed v1");
    let loaded = super::session_migration::read(V1_FIXTURE, path.clone()).expect("load v1");

    assert!(
        super::session_migration::commit_current_fail_before_rename(&loaded)
            .await
            .is_err()
    );
    let backup = super::session_migration::backup_path(&path).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), V1_FIXTURE);
    assert_eq!(std::fs::read(&backup).unwrap(), V1_FIXTURE);
    assert!(std::fs::read_dir(root.path()).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".tmp")));
}

#[cfg(unix)]
#[tokio::test]
async fn migrated_session_backup_and_directory_are_private() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().expect("tempdir");
    let private = root.path().join("private");
    let path = private.join("00000000-0000-4000-8000-000000000002.json");
    crate::services::private_store::atomic_write(&path, V1_FIXTURE).expect("seed v1");
    let loaded = super::session_migration::read(V1_FIXTURE, path.clone()).expect("load v1");
    super::session_migration::commit_current(&loaded)
        .await
        .unwrap();
    let backup = super::session_migration::backup_path(&path).unwrap();

    assert_eq!(
        std::fs::metadata(&private).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[tokio::test]
async fn writer_rejects_32_mib_plus_one_before_creating_a_temp() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("oversized.json");
    let mut session = base_session();
    session.name = "x".repeat(MAX_SESSION_FILE_BYTES as usize + 1);

    assert!(
        super::session_store_document::write_to_path(path.clone(), &session)
            .await
            .is_err()
    );
    assert!(!path.exists());
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
}

#[test]
fn continuity_accepts_exactly_16_mib_and_blocks_the_next_whole_turn() {
    let maximum = crate::services::reasoning_continuity::limits::MAX_ENVELOPE_BYTES;
    let envelope = envelope_with_serialized_len(maximum);
    let mut session = base_session();
    session.messages = vec![
        assistant_with_continuation("turn-a", envelope.clone()),
        assistant_with_continuation("turn-b", envelope),
    ];
    super::session_limits::validate_continuity(&session).expect("exact 16 MiB accepted");
    session.messages.push(assistant_with_continuation(
        "turn-c",
        responses_envelope(vec![json!({"extra":true})]),
    ));
    assert!(super::session_limits::validate_continuity(&session).is_err());
    assert!(session.messages[2].continuation.is_some());
}

fn base_session() -> super::types_session::AgentSession {
    super::session_migration::read(V1_FIXTURE, PathBuf::from("fixture.json"))
        .expect("fixture")
        .into_session()
}

fn responses_envelope(items: Vec<serde_json::Value>) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::CodexResponsesV1,
        ReasoningSource {
            route_id: RouteId::CodexOauth,
            model_id: "gpt-5.6-luna".into(),
            credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
            reasoning_mode: ReasoningModeId::Medium,
        },
        CompletionState::Complete,
        ContinuationState::ResponsesLocal { items },
        Vec::new(),
    )
}

fn assistant_with_continuation(turn_id: &str, continuation: ReasoningEnvelope) -> AgentMessage {
    let mut message = base_session().messages[1].clone();
    message.id = format!("message-{turn_id}");
    message.turn_id = turn_id.to_string();
    message.continuation = Some(continuation);
    message
}

fn envelope_with_serialized_len(target: usize) -> ReasoningEnvelope {
    let empty = ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: RouteId::Ollama,
            model_id: "fixture-model".into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: String::new(),
        },
        Vec::new(),
    );
    let overhead = serde_json::to_vec(&empty).unwrap().len();
    let envelope = ReasoningEnvelope::new(
        empty.contract_id,
        empty.source,
        empty.completion,
        ContinuationState::OllamaNative {
            thinking: "x".repeat(target - overhead),
        },
        Vec::new(),
    );
    assert_eq!(serde_json::to_vec(&envelope).unwrap().len(), target);
    envelope.validate().expect("envelope at limit");
    envelope
}

fn v1_with_tool_chain() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(V1_FIXTURE).unwrap();
    let messages = value["messages"].as_array_mut().unwrap();
    messages.push(json!({
        "id": "00000000-0000-4000-8000-000000000006",
        "role": "assistant",
        "content": "",
        "tool_calls": [{
            "function": {"name":"read_file","arguments":{"path":"fixture.txt"}}
        }],
        "files": [],
        "timestamp": "2026-08-25T10:00:00Z",
        "tokens": 0
    }));
    messages.push(json!({
        "id": "00000000-0000-4000-8000-000000000007",
        "role": "tool",
        "content": "fixture-tool-result",
        "tool_name": "read_file",
        "files": [],
        "timestamp": "2026-08-25T10:00:00Z",
        "tokens": 0
    }));
    serde_json::to_vec_pretty(&value).unwrap()
}

fn v1_with_incomplete_tool_chain() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(&v1_with_tool_chain()).unwrap();
    let messages = value["messages"].as_array_mut().unwrap();
    messages.pop();
    messages.last_mut().unwrap()["tool_calls"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "function": {"name":"read_file","arguments":{"path":"second-fixture.txt"}}
        }));
    serde_json::to_vec_pretty(&value).unwrap()
}

fn v1_with_two_turns() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(&v1_with_tool_chain()).unwrap();
    let messages = value["messages"].as_array_mut().unwrap();
    messages.push(json!({
        "id": "00000000-0000-4000-8000-000000000008",
        "role": "user",
        "content": "fixture-second-user",
        "files": [],
        "timestamp": "2026-08-25T10:01:00Z",
        "tokens": 0
    }));
    messages.push(json!({
        "id": "00000000-0000-4000-8000-000000000009",
        "role": "assistant",
        "content": "fixture-second-assistant",
        "files": [],
        "timestamp": "2026-08-25T10:01:01Z",
        "tokens": 0
    }));
    serde_json::to_vec_pretty(&value).unwrap()
}
