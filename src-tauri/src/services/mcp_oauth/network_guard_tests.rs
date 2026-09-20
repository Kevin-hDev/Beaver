use super::{prepare_with, DestinationRole};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};

const MCP: &str = "https://mcp.notion.com/mcp";

#[tokio::test]
async fn oauth_discovery_rejects_target_before_send() {
    let resolutions = AtomicUsize::new(0);
    let result = prepare_with(
        "notion",
        MCP,
        DestinationRole::OAuth,
        "https://evil.example/.well-known/oauth-authorization-server",
        |_, _| {
            resolutions.fetch_add(1, Ordering::SeqCst);
            async { Ok(vec![SocketAddr::from(([1, 1, 1, 1], 443))]) }
        },
    )
    .await;
    assert!(result.is_err());
    assert_eq!(resolutions.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn oauth_destination_pins_checked_address() {
    let result = prepare_with(
        "notion",
        MCP,
        DestinationRole::ResourceMetadata,
        "https://mcp.notion.com/.well-known/oauth-protected-resource",
        |_, _| async { Ok(vec![SocketAddr::from(([1, 1, 1, 1], 443))]) },
    )
    .await
    .unwrap();
    assert_eq!(result.address.ip(), IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
}

#[tokio::test]
async fn oauth_destination_refuses_mixed_and_oversized_dns_answers() {
    for addresses in [
        vec![
            SocketAddr::from(([1, 1, 1, 1], 443)),
            SocketAddr::from(([127, 0, 0, 1], 443)),
        ],
        vec![SocketAddr::from((Ipv6Addr::LOCALHOST, 443))],
        vec![SocketAddr::from(([169, 254, 169, 254], 443))],
        vec![SocketAddr::from(([100, 100, 100, 200], 443))],
        vec![SocketAddr::from(([1, 1, 1, 1], 8443))],
        vec![SocketAddr::from(([1, 1, 1, 1], 443)); 33],
        Vec::new(),
    ] {
        assert!(prepare_with(
            "notion",
            MCP,
            DestinationRole::OAuth,
            "https://notion.com/oauth/token",
            |_, _| async { Ok(addresses) },
        )
        .await
        .is_err());
    }
}

#[tokio::test]
async fn metadata_path_on_exact_mcp_origin_is_allowed_but_subdomain_is_not() {
    let target = "https://mcp.notion.com/.well-known/oauth-protected-resource";
    assert!(prepare_with(
        "notion",
        MCP,
        DestinationRole::ResourceMetadata,
        target,
        |_, _| async { Ok(vec![SocketAddr::from(([1, 1, 1, 1], 443))]) }
    )
    .await
    .is_ok());
    assert!(prepare_with(
        "notion",
        MCP,
        DestinationRole::ResourceMetadata,
        "https://other.mcp.notion.com/.well-known/oauth-protected-resource",
        |_, _| async { Ok(vec![SocketAddr::from(([1, 1, 1, 1], 443))]) },
    )
    .await
    .is_err());
}

#[tokio::test]
async fn resource_metadata_cannot_inherit_an_untrusted_mcp_origin() {
    for target in [
        "https://evil.example/.well-known/oauth-protected-resource",
        "https://notion.com/.well-known/oauth-protected-resource",
    ] {
        assert!(prepare_with(
            "notion",
            "https://evil.example/mcp",
            DestinationRole::ResourceMetadata,
            target,
            |_, _| async { Ok(vec![SocketAddr::from(([1, 1, 1, 1], 443))]) },
        )
        .await
        .is_err());
    }
}

#[tokio::test]
async fn oauth_target_rejects_port_userinfo_and_fragment_before_dns() {
    let resolutions = AtomicUsize::new(0);
    for target in [
        "https://notion.com:8443/oauth/token",
        "https://user@notion.com/oauth/token",
        "https://notion.com/oauth/token#fragment",
    ] {
        let result = prepare_with("notion", MCP, DestinationRole::OAuth, target, |_, _| {
            resolutions.fetch_add(1, Ordering::SeqCst);
            async { Ok(vec![SocketAddr::from(([1, 1, 1, 1], 443))]) }
        })
        .await;
        assert!(result.is_err());
    }
    assert_eq!(resolutions.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn loopback_test_exception_is_exact_and_local_only() {
    let allowed = "http://127.0.0.1:50001/token";
    assert!(super::destination_loopback_for_test(
        "test-refresh",
        allowed,
        "http://127.0.0.1:50002/token",
    )
    .await
    .is_err());
    assert!(super::destination_loopback_for_test(
        "test-refresh",
        allowed,
        "http://169.254.169.254:50001/token",
    )
    .await
    .is_err());
}
