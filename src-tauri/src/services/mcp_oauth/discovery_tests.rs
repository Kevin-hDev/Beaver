#[tokio::test]
async fn untrusted_mcp_endpoint_is_rejected_before_probe() {
    assert!(
        super::discover_auth_server("notion", "https://evil.example/mcp")
            .await
            .is_err()
    );
}
