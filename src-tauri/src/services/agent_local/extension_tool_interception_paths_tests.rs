use super::agent_loop_thinking_retry::EagerHandle;
use super::agent_loop_tool_batch;
use super::circuit_breaker::CircuitBreaker;
use super::types_tools::ToolResult;
use std::collections::HashMap;

#[tokio::test]
async fn denied_interception_prevents_the_real_write_executor_effect() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("must-not-exist.txt");
    let result = super::tool_executor_write::execute_write(
        &super::stream_events::AgentEventEmitter::test("interception-test".into()),
        "write_file",
        &serde_json::json!({"path":"must-not-exist.txt", "content":"forbidden"}),
        root.path(),
        "auto",
        &mut super::write_guard::WriteGuard::new(),
        "interception-test",
        "request",
        tokio_util::sync::CancellationToken::new(),
        false,
        None,
        None,
        &crate::services::extensions::InterceptionSnapshot::test_denied(),
    )
    .await;

    assert!(result.is_error);
    assert!(!path.exists());
}

#[tokio::test]
async fn eager_result_is_never_replayed_when_interception_is_active() {
    let eager: EagerHandle = tokio::spawn(async {
        HashMap::from([(0, ToolResult::ok("effect already produced"))])
    });
    let prepared = agent_loop_tool_batch::prepare(
        eager,
        false,
        &[("write_file".into(), serde_json::json!({"path":"blocked"}))],
        "session",
        &mut CircuitBreaker::new(),
        &crate::services::extensions::InterceptionSnapshot::test_non_empty(),
    )
    .await
    .unwrap();

    let result = prepared.eager_results.get(&0).unwrap();
    assert!(result.is_error);
    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some("extensions_eager_interception_conflict")
    );
    assert!(!result.content.contains("effect already produced"));
}
