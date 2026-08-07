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

    process_manager::shutdown_one("__beaver_mcp_fixture");
}
