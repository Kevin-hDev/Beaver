use super::{Manifest, MANIFEST_ENV};

pub(super) fn run() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let exit = crate::app_exit::AppExitCoordinator::initialize().expect("exit coordinator");
    let work = crate::runtime_state::agent_work(&exit).shells();
    crate::services::agent_local::tool_dispatcher_shell_runtime::test_support::with(work, || {
        runtime.block_on(emit_stream())
    });
}

async fn emit_stream() {
    use crate::services::agent_local::agent_loop_test_provider;
    use crate::services::agent_local::context_usage_buckets::ContextUsageSeed;
    use crate::services::agent_local::extension_tool_set::ExtensionToolSet;
    use crate::services::agent_local::stream_events::AgentEventEmitter;
    use crate::services::agent_local::types_ollama::StreamResult;
    use tokio_util::sync::CancellationToken;

    let root = tempfile::tempdir().expect("project");
    std::fs::write(root.path().join("proof.txt"), "completed read proof").expect("proof file");
    let session = super::super::session_store::create_full(
        "Killed stream",
        "qwen3.5:4b",
        "ollama",
        false,
        None,
    )
    .await
    .expect("session");
    let admitted = super::super::conversation_admission::new_turn(
        &session.id,
        crate::services::agent_local::conversation_input::ResolvedTurnInput {
            user_content: "recover this run".into(),
            provider_content: "recover this run".into(),
            files: Vec::new(),
            images: Vec::new(),
            skills: Vec::new(),
        },
        target(),
    )
    .await
    .expect("admit durable user");
    let request_id = uuid::Uuid::new_v4().to_string();
    let header = header(&session.id, &request_id, &admitted);
    let manifest = Manifest {
        data_dir: crate::services::paths::data_dir(),
        session_id: session.id.clone(),
    };
    std::fs::write(
        std::env::var_os(MANIFEST_ENV).expect("manifest path"),
        serde_json::to_vec(&manifest).expect("manifest json"),
    )
    .expect("write manifest");

    let cancel = CancellationToken::new();
    let (log, lease) = super::super::stream_recovery_log::StreamRecoveryLog::create(
        header.clone(),
        cancel.clone(),
    )
    .await
    .expect("recovery log");
    let emitter = AgentEventEmitter::test(session.id.clone()).with_recovery_log(log.clone());
    let mut journal = super::super::conversation_journal::ConversationJournal::new(
        session.id.clone(),
        header.turn_id,
        header.user_message_id,
        header.assistant_message_id,
        request_id.clone(),
    )
    .expect("conversation journal");
    journal.attach_recovery(log, lease);
    let responses = vec![
        StreamResult {
            content: "visible work".into(),
            thinking: "visible thinking".into(),
            tool_calls: vec![("read_file".into(), serde_json::json!({"path": "proof.txt"}))],
            tool_call_ids: vec!["call-first".into()],
            ..Default::default()
        },
        StreamResult {
            tool_calls: vec![("bash".into(), serde_json::json!({"command": "sleep 600"}))],
            tool_call_ids: vec!["call-second".into()],
            ..Default::default()
        },
    ];
    let _script = agent_loop_test_provider::install(&request_id, responses);
    let mut messages = admitted
        .history
        .messages
        .into_iter()
        .map(crate::commands::agent_chat_task::convert_provider_message_for_test)
        .collect::<Result<Vec<_>, _>>()
        .expect("provider history");
    crate::services::llm::agent_loop::run_agent_loop(
        &emitter,
        "fixture",
        crate::services::llm::fast_mode::FastModeRequest::Unsupported,
        "fixture",
        &mut messages,
        ExtensionToolSet::passthrough(Vec::new()),
        false,
        None,
        root.path().to_path_buf(),
        session.id,
        request_id,
        None,
        cancel,
        1_000_000,
        "manual",
        false,
        ContextUsageSeed::default(),
        None,
        None,
        Some(&mut journal),
    )
    .await
    .expect("parent must kill this process while permission waits");
}

fn target() -> crate::services::reasoning_continuity::contract::ReplayTarget {
    use crate::services::reasoning_continuity::contract::*;
    ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "qwen3.5:4b".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Off,
        continuation_use: ContinuationUse::UserContinuation,
    }
}

fn header(
    session_id: &str,
    request_id: &str,
    admitted: &super::super::conversation_admission::AdmittedTurn,
) -> super::super::stream_recovery_record::StreamRecoveryHeader {
    super::super::stream_recovery_record::StreamRecoveryHeader {
        version: super::super::stream_recovery_record::STREAM_RECOVERY_VERSION,
        process_instance_id: super::super::stream_recovery_record::process_instance_id().into(),
        session_id: session_id.into(),
        request_id: request_id.into(),
        turn_id: admitted.turn_id.clone(),
        user_message_id: admitted.user_message_id.clone(),
        assistant_message_id: admitted.assistant_message_id.clone(),
        subagent_owner: None,
        created_at: chrono::Utc::now(),
    }
}
