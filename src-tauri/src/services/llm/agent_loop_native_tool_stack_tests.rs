use super::fast_mode::FastModeRequest;
use crate::services::agent_local::agent_loop_test_provider;
use crate::services::agent_local::context_usage_buckets::ContextUsageSeed;
use crate::services::agent_local::extension_tool_set::ExtensionToolSet;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[test]
fn native_tool_dispatch_fits_the_production_worker_stack() {
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
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            let root = tempfile::tempdir().expect("temporary project");
            std::fs::write(root.path().join("needle.txt"), "stack-proof\n").expect("fixture");
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
                    tool_calls: vec![
                        (
                            "grep".into(),
                            json!({ "pattern": "stack-proof", "path": root.path() }),
                        ),
                        ("bash".into(), json!({ "command": "printf stack-proof" })),
                    ],
                    tool_call_ids: vec!["call-grep".into(), "call-bash".into()],
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
            assert_eq!(tool_outputs.len(), 2);
            assert!(
                tool_outputs[0].contains("stack-proof")
                    && tool_outputs[1].contains("shell_dispatch_failed"),
                "{tool_outputs:?}",
            );
            crate::services::agent_local::session_store::delete_one(&session.id)
                .await
                .expect("delete session");
        });
}
