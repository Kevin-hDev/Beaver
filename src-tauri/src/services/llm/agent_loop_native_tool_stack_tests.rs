use super::fast_mode::FastModeRequest;
use crate::services::agent_local::agent_loop_test_provider;
use crate::services::agent_local::context_usage_buckets::ContextUsageSeed;
use crate::services::agent_local::extension_tool_set::ExtensionToolSet;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use serde_json::json;
use tokio_util::sync::CancellationToken;

const CHILD_MARKER: &str = "BEAVER_NATIVE_TOOL_STACK_CHILD";
const TEST_NAME: &str = "services::llm::agent_loop_native_tool_stack_tests::native_tool_dispatch_fits_the_production_worker_stack";

#[test]
fn native_tool_dispatch_fits_the_production_worker_stack() {
    if std::env::var_os(CHILD_MARKER).is_none() {
        let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", TEST_NAME, "--nocapture"])
            .env(CHILD_MARKER, "1")
            .status()
            .expect("spawn cold test process");
        assert!(status.success(), "cold child failed with {status}");
        return;
    }

    let worker = std::thread::Builder::new()
        .name("agent-stack-regression".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(run_native_tools)
        .expect("spawn bounded worker");

    worker
        .join()
        .expect("native tool dispatch must not overflow");
}

fn run_native_tools() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let exit = crate::app_exit::AppExitCoordinator::initialize().expect("exit coordinator");
    let work = crate::runtime_state::agent_work(&exit).shells();
    crate::services::agent_local::tool_dispatcher_shell_runtime::test_support::with(work, || {
        runtime.block_on(async {
            let root = tempfile::tempdir().expect("temporary project");
            let session = crate::services::agent_local::session_store::create_full(
                "Native tool stack",
                "fixture",
                "fixture",
                false,
                None,
            )
            .await
            .expect("session");
            let request_id = uuid::Uuid::new_v4().to_string();
            let responses = vec![
                StreamResult {
                    tool_calls: vec![(
                        "bash".into(),
                        json!({
                            "command": "printf stack-proof-shell",
                            "yield_time_ms": 30_000,
                        }),
                    )],
                    tool_call_ids: vec!["call-bash".into()],
                    ..Default::default()
                },
                StreamResult::default(),
            ];
            let _script = agent_loop_test_provider::install(&request_id, responses);
            let mut messages = vec![ChatMessage::user("run native tools".into())];

            super::agent_loop::run_agent_loop(
                &AgentEventEmitter::test(session.id.clone()),
                "fixture",
                FastModeRequest::Unsupported,
                "fixture",
                &mut messages,
                ExtensionToolSet::passthrough(Vec::new()),
                false,
                None,
                root.path().to_path_buf(),
                session.id.clone(),
                request_id,
                None,
                CancellationToken::new(),
                1_000_000,
                "auto",
                false,
                ContextUsageSeed::default(),
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
            crate::services::agent_local::session_store::delete_one(&session.id)
                .await
                .expect("delete session");
        })
    });
}
