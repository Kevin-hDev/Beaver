use super::http_catalog::list;
use super::http_client::BeaverHttpClient;
use super::http_lifecycle::start_with_client;
use futures_util::StreamExt;
use rmcp::transport::streamable_http_client::StreamableHttpClient;
use rmcp::transport::streamable_http_client::StreamableHttpPostResponse;
use serde_json::json;
use std::sync::Arc;
use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

impl BeaverHttpClient {
    fn new_loopback(endpoint: &str, token: &str) -> Self {
        let endpoint = reqwest::Url::parse(endpoint).expect("test endpoint");
        assert_eq!(endpoint.scheme(), "http");
        assert!(endpoint.host().is_some_and(|host| match host {
            url::Host::Ipv4(address) => address.is_loopback(),
            url::Host::Ipv6(address) => address.is_loopback(),
            _ => false,
        }));
        Self {
            client: crate::services::secure_http::AuthenticatedClient::new_loopback_streaming(
                std::time::Duration::from_secs(2),
                std::time::Duration::from_secs(2),
            )
            .expect("test client"),
            endpoint,
            token: Arc::new(zeroize::Zeroizing::new(token.to_owned())),
            bytes_left: Arc::new(std::sync::atomic::AtomicUsize::new(
                crate::services::secure_http::MCP_BODY_LIMIT,
            )),
            cancellation: tokio_util::sync::CancellationToken::new(),
        }
    }
}

#[test]
fn http_sdk_binds_token_to_exact_endpoint() {
    let client = BeaverHttpClient::new(
        "github",
        "https://api.githubcopilot.com/mcp",
        "fixture-token",
    )
    .expect("trusted endpoint");
    assert!(client.accepts("https://api.githubcopilot.com/mcp"));
    for target in [
        "https://mcp.notion.com/mcp",
        "https://api.githubcopilot.com/other",
        "https://api.githubcopilot.com:444/mcp",
        "https://api.githubcopilot.com/mcp?x=1",
        "https://api.githubcopilot.com/mcp#fragment",
        "https://user@api.githubcopilot.com/mcp",
    ] {
        assert!(!client.accepts(target), "unexpected accepted destination");
    }
    assert!(BeaverHttpClient::new(
        "github",
        "https://user@api.githubcopilot.com/mcp",
        "fixture-token",
    )
    .is_err());
}

#[tokio::test]
async fn http_sdk_refuses_mismatched_destination_before_all_methods_send() {
    let server = MockServer::start().await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    let message: rmcp::model::ClientJsonRpcMessage = serde_json::from_value(json!({
        "jsonrpc": "2.0", "id": 1, "method": "ping"
    }))
    .expect("valid SDK request");

    for destination in [
        format!("{}/other", server.uri()),
        format!("{endpoint}?x=1"),
        format!("{endpoint}#fragment"),
        endpoint.replacen("http://", "http://user@", 1),
        "https://mcp.notion.com/mcp".to_string(),
    ] {
        let other = Arc::<str>::from(destination);
        assert!(client
            .post_message(
                other.clone(),
                message.clone(),
                None,
                None,
                Default::default()
            )
            .await
            .is_err());
        assert!(client
            .post_message_with_max_sse_event_size(
                other.clone(),
                message.clone(),
                None,
                None,
                Default::default(),
                1024,
            )
            .await
            .is_err());
        assert!(client
            .get_stream(other.clone(), None, None, None, Default::default())
            .await
            .is_err());
        assert!(client
            .get_stream_with_max_sse_event_size(
                other.clone(),
                None,
                None,
                None,
                Default::default(),
                1024,
            )
            .await
            .is_err());
        assert!(client
            .delete_session(other, Arc::from("session"), None, Default::default())
            .await
            .is_err());
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn http_sdk_rejects_redirect_without_forwarding_token() {
    let origin = MockServer::start().await;
    let destination = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/sink", destination.uri())),
        )
        .mount(&origin)
        .await;
    let client = BeaverHttpClient::new_loopback(&format!("{}/mcp", origin.uri()), "fixture-token");

    assert!(client
        .get_stream(
            Arc::from(format!("{}/mcp", origin.uri())),
            None,
            None,
            None,
            Default::default(),
        )
        .await
        .is_err());
    assert_eq!(origin.received_requests().await.unwrap().len(), 1);
    assert!(destination.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn http_sdk_rejects_oversized_sse() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(format!("data: {}", "x".repeat(100)), "text/event-stream"),
        )
        .mount(&server)
        .await;
    let endpoint = Arc::<str>::from(format!("{}/mcp", server.uri()));
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");

    let mut get = client
        .get_stream_with_max_sse_event_size(
            endpoint.clone(),
            None,
            None,
            None,
            Default::default(),
            32,
        )
        .await
        .unwrap();
    assert!(get.next().await.unwrap().is_err());

    let message: rmcp::model::ClientJsonRpcMessage = serde_json::from_value(json!({
        "jsonrpc": "2.0", "id": 1, "method": "ping"
    }))
    .unwrap();
    let post = client
        .post_message_with_max_sse_event_size(endpoint, message, None, None, Default::default(), 32)
        .await
        .unwrap();
    let StreamableHttpPostResponse::Sse(mut stream, _) = post else {
        panic!("expected SSE response");
    };
    assert!(stream.next().await.unwrap().is_err());
}

#[tokio::test]
async fn http_sdk_rejects_wrong_response_id() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0", "id": 999,
            "result": {"resultType": "complete", "supportedVersions": ["2026-07-28"],
                "capabilities": {}, "ttlMs": 0, "cacheScope": "private"}
        })))
        .mount(&server)
        .await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");

    assert!(start_with_client(client, &endpoint).await.is_err());
    assert!(!server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn http_sdk_legacy_catalog_has_bounded_fallback_ttl() {
    let server = super::http_test_server::legacy_session_server(200).await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    let mut service = start_with_client(client, &endpoint)
        .await
        .expect("legacy initialization");
    let catalog = list(&service).await.expect("catalogue");
    assert_eq!(catalog.tools.len(), 1);
    assert_eq!(catalog.cache_ttl, Some(super::registry_cache::FALLBACK_TTL));
    service.close().await.expect("worker closed");
    let requests = server.received_requests().await.unwrap();
    assert!(requests.len() <= 32);
    let methods: Vec<_> = requests
        .iter()
        .filter_map(|request| {
            serde_json::from_slice::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|body| body["method"].as_str().map(str::to_owned))
        })
        .collect();
    assert_eq!(
        methods
            .iter()
            .filter(|method| *method == "server/discover")
            .count(),
        1
    );
    assert_eq!(
        methods
            .iter()
            .filter(|method| *method == "initialize")
            .count(),
        1
    );
}

#[tokio::test]
async fn http_sdk_never_replays_tool_call() {
    let server = super::http_test_server::legacy_session_server(404).await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    let mut service = start_with_client(client, &endpoint)
        .await
        .expect("legacy initialization");
    let result = service
        .call_tool_once(rmcp::model::CallToolRequestParams::new("echo"))
        .await;
    assert!(result.is_err());
    service.close().await.expect("worker closed");
    let requests = server.received_requests().await.unwrap();
    assert!(requests.len() <= 32);
    let calls = requests
        .iter()
        .filter(|request| {
            serde_json::from_slice::<serde_json::Value>(&request.body)
                .ok()
                .and_then(|body| body["method"].as_str().map(str::to_owned))
                .as_deref()
                == Some("tools/call")
        })
        .count();
    assert_eq!(calls, 1);
}

#[tokio::test]
async fn http_sdk_modern_catalog_uses_server_ttl_only_when_valid() {
    for (ttl, scope, expected) in [
        (None, None, None),
        (Some(json!(0)), Some("private"), None),
        (
            Some(json!(1000)),
            Some("private"),
            Some(std::time::Duration::from_secs(1)),
        ),
        (
            Some(json!(600_000)),
            Some("public"),
            Some(super::registry_cache::FALLBACK_TTL),
        ),
        (Some(json!(1000)), Some("future-scope"), None),
    ] {
        let server = super::http_test_server::modern_catalog_server(ttl, scope).await;
        let endpoint = format!("{}/mcp", server.uri());
        let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
        let mut service = start_with_client(client, &endpoint)
            .await
            .expect("modern discovery");
        let catalog = list(&service).await.expect("modern catalogue");
        assert_eq!(catalog.cache_ttl, expected);
        service.close().await.expect("worker closed");
        assert!(server.received_requests().await.unwrap().len() <= 32);
    }
}

#[tokio::test]
async fn http_sdk_refuses_negative_or_string_ttl_before_sdk_normalizes_it() {
    for ttl in [json!(-1), json!("1000")] {
        let server =
            super::http_test_server::modern_catalog_server(Some(ttl), Some("private")).await;
        let endpoint = format!("{}/mcp", server.uri());
        let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
        let mut service = start_with_client(client, &endpoint)
            .await
            .expect("modern discovery");
        assert!(list(&service).await.is_err());
        service.close().await.expect("worker closed");
    }
}

#[tokio::test]
async fn http_sdk_auth_and_busy_servers_never_fall_back_to_legacy() {
    for status in [401, 429, 503] {
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(status))
            .mount(&server)
            .await;
        let endpoint = format!("{}/mcp", server.uri());
        let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
        assert!(start_with_client(client, &endpoint).await.is_err());
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
    }
}

#[tokio::test]
async fn http_sdk_http_not_found_is_not_explicit_method_not_found() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");

    assert!(start_with_client(client, &endpoint).await.is_err());
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1, "HTTP 404 must not trigger initialize");
}

#[tokio::test]
async fn http_sdk_empty_discovery_ack_never_falls_back() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(202))
        .mount(&server)
        .await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        start_with_client(client, &endpoint),
    )
    .await;
    assert!(
        matches!(result, Ok(Err(_))),
        "discovery needs an explicit answer"
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn http_sdk_invalid_discovery_error_never_falls_back() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(|request: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
            ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc":"2.0", "id":body["id"],
                "error":{"code":-32600,"message":"invalid request"}
            }))
        })
        .mount(&server)
        .await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");

    assert!(start_with_client(client, &endpoint).await.is_err());
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn http_sdk_incompatible_version_never_falls_back_to_legacy() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(|request: &wiremock::Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
            ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc":"2.0", "id":body["id"], "result":{
                    "resultType":"complete", "supportedVersions":["2099-01-01"],
                    "capabilities":{},"ttlMs":0,"cacheScope":"private"
                }
            }))
        })
        .mount(&server)
        .await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");

    assert!(start_with_client(client, &endpoint).await.is_err());
    let requests = server.received_requests().await.unwrap();
    assert!(requests.len() <= 2);
    assert!(requests.iter().all(|request| {
        serde_json::from_slice::<serde_json::Value>(&request.body)
            .ok()
            .and_then(|body| {
                body["method"]
                    .as_str()
                    .map(|method| method == "server/discover")
            })
            .unwrap_or(false)
    }));
}

#[tokio::test]
async fn http_sdk_reads_fragmented_sse_without_buffering_the_whole_response() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/mcp", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0u8; 2048];
        socket.read(&mut request).await.unwrap();
        let chunks = [
            b"data: {\"ok".as_slice(),
            b"\":true}\n".as_slice(),
            b"\n".as_slice(),
        ];
        let len: usize = chunks.iter().map(|chunk| chunk.len()).sum();
        socket.write_all(format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n"
        ).as_bytes()).await.unwrap();
        for chunk in chunks {
            socket.write_all(chunk).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    });
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    let mut stream = client
        .get_stream(Arc::from(endpoint), None, None, None, Default::default())
        .await
        .expect("stream open");
    assert_eq!(
        stream.next().await.unwrap().unwrap().data.as_deref(),
        Some("{\"ok\":true}")
    );
    assert!(stream.next().await.is_none());
    server.await.unwrap();
}

#[tokio::test]
async fn http_sdk_slow_discovery_fails_closed_without_legacy_fallback() {
    let server = super::http_test_server::legacy_slow_discover_server().await;
    let endpoint = format!("{}/mcp", server.uri());
    let mut client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    client.client = crate::services::secure_http::AuthenticatedClient::new_loopback_streaming(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(15),
    )
    .unwrap();
    let started = std::time::Instant::now();
    let started_service = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        start_with_client(client, &endpoint),
    )
    .await
    .expect("bounded discovery");
    assert!(started_service.is_err(), "slow discovery must fail closed");
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
    let requests = server.received_requests().await.unwrap();
    assert!(requests.len() <= 32);
    assert!(!requests.iter().any(|request| {
        serde_json::from_slice::<serde_json::Value>(&request.body)
            .ok()
            .and_then(|body| body["method"].as_str().map(|method| method == "initialize"))
            .unwrap_or(false)
    }));
}

#[tokio::test]
async fn http_sdk_abandoned_startup_releases_transport() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/mcp", listener.local_addr().unwrap());
    let mut client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    client.client = crate::services::secure_http::AuthenticatedClient::new_loopback_streaming(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(15),
    )
    .unwrap();
    let token_owner = Arc::downgrade(&client.token);
    let request_endpoint = endpoint.clone();
    let startup = tokio::spawn(async move { start_with_client(client, &request_endpoint).await });

    let (mut socket, _) = listener.accept().await.unwrap();
    let (method, body) = super::http_test_server::read_request(&mut socket)
        .await
        .expect("complete discovery request");
    assert_eq!(method, "POST");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["method"],
        "server/discover"
    );

    startup.abort();
    assert!(startup.await.unwrap_err().is_cancelled());
    assert_transport_released(token_owner).await;
}

#[tokio::test]
async fn http_sdk_abandoned_operation_releases_transport() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/mcp", listener.local_addr().unwrap());
    let mut client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    client.client = crate::services::secure_http::AuthenticatedClient::new_loopback_streaming(
        std::time::Duration::from_secs(2),
        std::time::Duration::from_secs(15),
    )
    .unwrap();
    let token_owner = Arc::downgrade(&client.token);
    let operation = tokio::spawn(async move {
        super::http_lifecycle::list_tools_with_client(client, endpoint).await
    });

    let (mut socket, _) = listener.accept().await.unwrap();
    let (_, body) = super::http_test_server::read_request(&mut socket)
        .await
        .expect("complete discovery request");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["method"],
        "server/discover"
    );

    operation.abort();
    assert!(matches!(operation.await, Err(error) if error.is_cancelled()));
    assert_transport_released(token_owner).await;
}

#[tokio::test]
async fn http_sdk_abandoned_list_releases_running_service() {
    let server = super::http_test_server::legacy_slow_list_server().await;
    let endpoint = format!("{}/mcp", server.uri());
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    let token_owner = Arc::downgrade(&client.token);
    let operation = tokio::spawn(async move {
        super::http_lifecycle::list_tools_with_client(client, endpoint).await
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let requests = server.received_requests().await.unwrap();
            if requests.iter().any(|request| {
                serde_json::from_slice::<serde_json::Value>(&request.body)
                    .ok()
                    .and_then(|body| body["method"].as_str().map(|method| method == "tools/list"))
                    .unwrap_or(false)
            }) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("tools/list request sent");

    operation.abort();
    assert!(matches!(operation.await, Err(error) if error.is_cancelled()));
    assert_transport_released(token_owner).await;
}

async fn assert_transport_released(token_owner: std::sync::Weak<zeroize::Zeroizing<String>>) {
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while token_owner.strong_count() != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "transport must release its token owner (owners: {})",
            token_owner.strong_count()
        )
    });
}

#[tokio::test]
async fn http_sdk_cut_after_recorded_call_does_not_replay_it() {
    let (endpoint, server) = super::http_test_server::cut_after_call_server().await;
    let client = BeaverHttpClient::new_loopback(&endpoint, "fixture-token");
    let started = start_with_client(client, &endpoint).await;
    if started.is_err() {
        let (_, methods) = server.await.unwrap();
        panic!("legacy initialization failed after methods: {methods:?}");
    }
    let mut service = started.expect("legacy initialization");
    let result = service
        .call_tool_once(rmcp::model::CallToolRequestParams::new("echo"))
        .await;
    assert!(result.is_err());
    service.close().await.expect("worker closed");
    assert_eq!(server.await.unwrap().0, 1);
}
