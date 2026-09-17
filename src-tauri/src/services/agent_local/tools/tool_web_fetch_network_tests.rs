use crate::services::agent_local::tool_web_fetch::fetch_url_allow_private;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn fetches_plain_text() {
    let url = serve(vec![response("200 OK", "text/plain", "bonjour")]).await;
    let result = fetch_url_allow_private(&url).await.unwrap();
    assert_eq!(result, "bonjour");
}

#[tokio::test]
async fn fetches_json_as_text() {
    let url = serve(vec![response(
        "200 OK",
        "application/json",
        "{\"ok\":true}",
    )])
    .await;
    let result = fetch_url_allow_private(&url).await.unwrap();
    assert_eq!(result, "{\"ok\":true}");
}

#[tokio::test]
async fn converts_html_to_readable_text() {
    let body = "<html><body><main><h1>Titre</h1><p>Texte principal.</p></main></body></html>";
    let url = serve(vec![response("200 OK", "text/html", body)]).await;
    let result = fetch_url_allow_private(&url).await.unwrap();
    assert!(result.contains("Titre"));
}

#[tokio::test]
async fn follows_valid_redirect() {
    let url = serve(vec![
        raw_response("302 Found", &[("Location", "/final")], ""),
        response("200 OK", "text/plain", "arrivé"),
    ])
    .await;
    let result = fetch_url_allow_private(&url).await.unwrap();
    assert_eq!(result, "arrivé");
}

#[tokio::test]
async fn rejects_oversized_body() {
    let big = "a".repeat((5 * 1024 * 1024) + 1);
    let url = serve(vec![response("200 OK", "text/plain", &big)]).await;
    let result = fetch_url_allow_private(&url).await.unwrap_err();
    assert!(result.contains("trop volumineuse"));
}

#[tokio::test]
async fn rejects_unsupported_content_type() {
    let url = serve(vec![response("200 OK", "image/png", "not-png")]).await;
    let result = fetch_url_allow_private(&url).await.unwrap_err();
    assert!(result.contains("non supporté"));
}

async fn serve(responses: Vec<String>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        for response in responses {
            let Ok((mut socket, _)) = listener.accept().await else {
                return;
            };
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let _ = socket.write_all(response.as_bytes()).await;
        }
    });
    format!("http://{addr}/")
}

fn response(status: &str, content_type: &str, body: &str) -> String {
    let content_length = body.len().to_string();
    raw_response(
        status,
        &[
            ("Content-Type", content_type),
            ("Content-Length", &content_length),
        ],
        body,
    )
}

fn raw_response(status: &str, headers: &[(&str, &str)], body: &str) -> String {
    let mut response = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    response.push_str(body);
    response
}
