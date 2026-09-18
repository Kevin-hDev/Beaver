use super::agent_loop_test_provider;
use super::context_usage_buckets::ContextUsageSeed;
use super::extension_tool_set::ExtensionToolSet;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, OllamaThink, StreamResult};
use crate::services::reasoning_fixture_run::FixtureRunContext;
use serde_json::json;
use tokio_util::sync::CancellationToken;

const NATIVE_CHILD_MARKER: &str = "BEAVER_OLLAMA_TOOL_STACK_CHILD";
const NATIVE_TEST_NAME: &str = "services::agent_local::agent_loop_unbounded_tests::ollama_native_tool_dispatch_fits_the_production_worker_stack";

#[tokio::test]
async fn ollama_loop_continues_past_200_turns() {
    let mut fixture = FixtureRunContext::start().await.expect("fixture");
    let working_dir = fixture.root_for_test();
    let tools = ExtensionToolSet::passthrough(fixture.definitions().into());
    let session =
        super::session_store::create_full("Ollama unbounded", "fixture", "ollama", false, None)
            .await
            .expect("session");
    let session_id = session.id.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let script = agent_loop_test_provider::install(&request_id, tool_turns(201));
    let mut messages = vec![ChatMessage::user("continue".into())];
    let cancel = CancellationToken::new();

    let result = super::agent_loop::run_agent_loop(
        &AgentEventEmitter::test(session_id.clone()),
        &mut messages,
        "fixture",
        tools,
        OllamaThink::Bool(false),
        working_dir,
        session_id,
        request_id,
        None,
        cancel.clone(),
        1_000_000,
        "auto",
        false,
        ContextUsageSeed::default(),
        false,
        None,
        None,
        Some(&mut fixture),
        None,
    )
    .await;
    let error = match result {
        Ok(_) => panic!("script exhaustion must cancel the loop"),
        Err(error) => error,
    };
    super::session_store::delete_one(&session.id)
        .await
        .expect("delete session");

    assert_eq!(error, "Annulé");
    assert!(cancel.is_cancelled());
    assert_eq!(script.calls(), 201);
    assert_eq!(
        fixture
            .dispatch("fixture.read_note", &json!({}))
            .await
            .unwrap(),
        json!({ "value": "200" })
    );
}

fn tool_turns(count: usize) -> Vec<StreamResult> {
    (0..count)
        .map(|turn| StreamResult {
            tool_calls: vec![(
                "fixture.write_note".into(),
                json!({ "value": turn.to_string() }),
            )],
            tool_call_ids: vec![format!("call-{turn}")],
            ..Default::default()
        })
        .collect()
}

#[test]
fn ollama_native_tool_dispatch_fits_the_production_worker_stack() {
    if std::env::var_os(NATIVE_CHILD_MARKER).is_none() {
        let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", NATIVE_TEST_NAME, "--nocapture"])
            .env(NATIVE_CHILD_MARKER, "1")
            .status()
            .expect("spawn cold test process");
        assert!(status.success(), "cold child failed with {status}");
        return;
    }

    std::thread::Builder::new()
        .name("ollama-agent-stack-regression".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(run_native_tool)
        .expect("spawn bounded worker")
        .join()
        .expect("native tool dispatch must not overflow");
}

fn run_native_tool() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let exit = crate::app_exit::AppExitCoordinator::initialize().expect("exit coordinator");
    let work = crate::runtime_state::agent_work(&exit).shells();
    super::tool_dispatcher_shell_runtime::test_support::with(work, || {
        runtime.block_on(async {
            let root = tempfile::tempdir().expect("temporary project");
            let session = super::session_store::create_full(
                "Ollama native tool stack",
                "fixture",
                "ollama",
                false,
                None,
            )
            .await
            .expect("session");
            let request_id = uuid::Uuid::new_v4().to_string();
            let _script = agent_loop_test_provider::install(
                &request_id,
                vec![
                    StreamResult {
                        tool_calls: vec![(
                            "bash".into(),
                            json!({
                                "command": agent_loop_test_provider::NATIVE_SHELL_COMMAND,
                                "yield_time_ms": 30_000,
                            }),
                        )],
                        tool_call_ids: vec!["call-bash".into()],
                        ..Default::default()
                    },
                    StreamResult::default(),
                ],
            );
            let mut messages = vec![ChatMessage::user("run native tool".into())];

            super::agent_loop::run_agent_loop(
                &AgentEventEmitter::test(session.id.clone()),
                &mut messages,
                "fixture",
                ExtensionToolSet::passthrough(Vec::new()),
                OllamaThink::Bool(false),
                root.path().to_path_buf(),
                session.id.clone(),
                request_id,
                None,
                CancellationToken::new(),
                1_000_000,
                "auto",
                false,
                ContextUsageSeed::default(),
                false,
                None,
                None,
                None,
                None,
            )
            .await
            .expect("agent loop");

            let tool_outputs: Vec<_> = messages
                .iter()
                .filter(|message| message.role == "tool")
                .map(|message| message.content.as_str())
                .collect();
            assert_eq!(tool_outputs.len(), 1);
            assert_eq!(tool_outputs[0], "stack-proof-shell");
            super::session_store::delete_one(&session.id)
                .await
                .expect("delete session");
        })
    });
}
