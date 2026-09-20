use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wiremock::{matchers::any, Mock, MockServer, Request, ResponseTemplate};

pub(super) async fn legacy_session_server(call_status: u16) -> MockServer {
    legacy_server(call_status, None, None).await
}

pub(super) async fn legacy_slow_discover_server() -> MockServer {
    legacy_server(200, Some(std::time::Duration::from_secs(11)), None).await
}

pub(super) async fn legacy_slow_list_server() -> MockServer {
    legacy_server(200, None, Some(std::time::Duration::from_secs(11))).await
}

async fn legacy_server(
    call_status: u16,
    discover_delay: Option<std::time::Duration>,
    list_delay: Option<std::time::Duration>,
) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(move |request: &Request| {
            match request.method.as_str() {
                "GET" => return ResponseTemplate::new(405),
                "DELETE" => return ResponseTemplate::new(204),
                "POST" => {}
                _ => return ResponseTemplate::new(405),
            }
            let payload: Value = match serde_json::from_slice(&request.body) {
                Ok(payload) => payload,
                Err(_) => return ResponseTemplate::new(400),
            };
            let id = payload.get("id").cloned().unwrap_or(Value::Null);
            match payload.get("method").and_then(Value::as_str) {
                Some("server/discover") => {
                    let response = ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc":"2.0","id":id,
                        "error":{"code":-32601,"message":"method not found"}
                    }));
                    match discover_delay {
                        Some(delay) => response.set_delay(delay),
                        None => response,
                    }
                }
                Some("initialize") => ResponseTemplate::new(200)
                    .set_body_json(json!({"jsonrpc":"2.0","id":id,"result":{
                        "protocolVersion":"2025-03-26", "capabilities":{},
                        "serverInfo":{"name":"test-server","version":"1.0.0"}
                    }}))
                    .insert_header("Mcp-Session-Id", "fixture-session"),
                Some("notifications/initialized") => ResponseTemplate::new(202),
                Some("tools/list") => {
                    let response = ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc":"2.0","id":id,"result":{"tools":[{
                            "name":"echo", "inputSchema":{"type":"object","properties":{}}
                        }]}
                    }));
                    match list_delay {
                        Some(delay) => response.set_delay(delay),
                        None => response,
                    }
                }
                Some("tools/call") if call_status == 200 => ResponseTemplate::new(200)
                    .set_body_json(json!({"jsonrpc":"2.0","id":id,"result":{
                        "content":[{"type":"text","text":"ok"}]
                    }})),
                Some("tools/call") => ResponseTemplate::new(call_status),
                _ => ResponseTemplate::new(400).set_body_json(json!({
                    "jsonrpc":"2.0","id":id,
                    "error":{"code":-32601,"message":"method not found"}
                })),
            }
        })
        .mount(&server)
        .await;
    server
}

pub(super) async fn modern_catalog_server(
    ttl_ms: Option<Value>,
    scope: Option<&str>,
) -> MockServer {
    let server = MockServer::start().await;
    let scope = scope.map(str::to_owned);
    Mock::given(any())
        .respond_with(move |request: &Request| {
            if request.method.as_str() == "DELETE" {
                return ResponseTemplate::new(204);
            }
            if request.method.as_str() == "GET" {
                return ResponseTemplate::new(405);
            }
            let payload: Value = match serde_json::from_slice(&request.body) {
                Ok(payload) => payload,
                Err(_) => return ResponseTemplate::new(400),
            };
            let id = payload.get("id").cloned().unwrap_or(Value::Null);
            match payload.get("method").and_then(Value::as_str) {
                Some("server/discover") => ResponseTemplate::new(200).set_body_json(json!({
                    "jsonrpc":"2.0","id":id,"result":{
                        "resultType":"complete", "supportedVersions":["2026-07-28"],
                        "capabilities":{}, "ttlMs":0,"cacheScope":"private"
                    }
                })),
                Some("tools/list") => {
                    let mut result = json!({"resultType":"complete","tools":[{
                        "name":"echo", "inputSchema":{"type":"object","properties":{}}
                    }]});
                    if let Some(ttl) = &ttl_ms {
                        result["ttlMs"] = ttl.clone();
                    }
                    if let Some(scope) = &scope {
                        result["cacheScope"] = Value::String(scope.clone());
                    }
                    ResponseTemplate::new(200)
                        .set_body_json(json!({"jsonrpc":"2.0","id":id,"result":result}))
                }
                _ => ResponseTemplate::new(400).set_body_json(json!({
                    "jsonrpc":"2.0","id":id,
                    "error":{"code":-32601,"message":"method not found"}
                })),
            }
        })
        .mount(&server)
        .await;
    server
}

pub(super) async fn cut_after_call_server(
) -> (String, tokio::task::JoinHandle<(usize, Vec<String>)>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/mcp", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let mut calls = 0;
        let mut methods = Vec::new();
        for _ in 0..32 {
            let Ok(Ok((mut socket, _))) =
                tokio::time::timeout(std::time::Duration::from_secs(2), listener.accept()).await
            else {
                break;
            };
            let Ok(Some((method, body))) =
                tokio::time::timeout(std::time::Duration::from_secs(2), read_request(&mut socket))
                    .await
            else {
                break;
            };
            let payload: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            methods.push(
                payload
                    .get("method")
                    .and_then(Value::as_str)
                    .unwrap_or(&method)
                    .to_owned(),
            );
            let id = payload.get("id").cloned().unwrap_or(Value::Null);
            let response = match (
                method.as_str(),
                payload.get("method").and_then(Value::as_str),
            ) {
                ("POST", Some("server/discover")) => (
                    200,
                    Some(json!({
                        "jsonrpc":"2.0","id":id,
                        "error":{"code":-32601,"message":"method not found"}
                    })),
                ),
                ("POST", Some("initialize")) => (
                    200,
                    Some(json!({
                        "jsonrpc":"2.0","id":id,"result":{
                            "protocolVersion":"2025-03-26","capabilities":{},
                            "serverInfo":{"name":"test-server","version":"1.0.0"}
                        }
                    })),
                ),
                ("POST", Some("notifications/initialized")) => (202, None),
                ("POST", Some("tools/call")) => {
                    calls += 1;
                    continue;
                }
                ("DELETE", _) => (204, None),
                ("GET", _) => (405, None),
                _ => (400, None),
            };
            let body = response
                .1
                .map(|value| value.to_string())
                .unwrap_or_default();
            let response_headers = match (response.0, payload.get("method").and_then(Value::as_str))
            {
                (200, Some("server/discover")) => "Content-Type: application/json\r\n",
                (200, _) => "Content-Type: application/json\r\nMcp-Session-Id: fixture-session\r\n",
                _ => "",
            };
            let header = format!(
                "HTTP/1.1 {} OK\r\n{response_headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
                response.0, body.len(),
            );
            if socket.write_all(header.as_bytes()).await.is_err() {
                break;
            }
            if socket.write_all(body.as_bytes()).await.is_err() {
                break;
            }
        }
        (calls, methods)
    });
    (endpoint, task)
}

pub(super) async fn read_request(socket: &mut tokio::net::TcpStream) -> Option<(String, Vec<u8>)> {
    let mut bytes = Vec::new();
    let header_end = loop {
        if bytes.len() > 16 * 1024 {
            return None;
        }
        let mut chunk = [0u8; 2048];
        let count = socket.read(&mut chunk).await.ok()?;
        if count == 0 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let header = std::str::from_utf8(&bytes[..header_end]).ok()?;
    let method = header.split_whitespace().next()?.to_owned();
    let length = header
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .and_then(|value| value.trim().parse::<usize>().ok())
        })
        .unwrap_or(0);
    if length > 16 * 1024 {
        return None;
    }
    while bytes.len() - header_end < length {
        let mut chunk = [0u8; 2048];
        let count = socket.read(&mut chunk).await.ok()?;
        if count == 0 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Some((method, bytes[header_end..header_end + length].to_vec()))
}
