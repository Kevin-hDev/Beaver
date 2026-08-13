use super::process_manager;
use super::stdio::StdioTransport;
use super::transport::McpTransport;

#[tokio::test]
async fn stdio_transport_handshakes_and_calls_a_real_child_process() {
    let transport = StdioTransport::for_test_fixture();

    let tools = transport.list_tools().await.expect("tools/list response");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");

    let result = transport
        .call_tool("echo", serde_json::json!({ "value": "hello" }))
        .await
        .expect("tools/call response");
    assert_eq!(result.content, "hello");

    let pid = process_manager::process_id_for_test("__beaver_mcp_fixture")
        .expect("fixture process remains owned");
    process_manager::shutdown_one("__beaver_mcp_fixture").await;

    let mut processes = sysinfo::System::new();
    processes.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    assert!(processes.process(sysinfo::Pid::from_u32(pid)).is_none());
}

#[tokio::test]
async fn slow_ready_connector_uses_protocol_signal_instead_of_fixed_warmup() {
    let transport = StdioTransport::for_slow_test_fixture(650);
    let started = std::time::Instant::now();

    let tools = tokio::time::timeout(
        std::time::Duration::from_millis(900),
        transport.list_tools(),
    )
    .await
    .expect("the protocol response, not an added warmup, owns readiness")
    .expect("slow fixture tools");

    assert_eq!(tools[0].name, "echo");
    assert!(started.elapsed() >= std::time::Duration::from_millis(600));
    process_manager::shutdown_one("__beaver_mcp_slow_fixture").await;
}
