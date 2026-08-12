use super::*;

async fn send_callback(port: u16, state: &str) {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("callback connection");
    let request = format!(
        "GET /callback?code=code-{state}&state={state} HTTP/1.1\r\nHost: localhost\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
}

#[tokio::test]
async fn cancelled_wait_releases_the_owned_listener_before_returning() {
    let server = CallbackServer::bind().await.expect("callback server");
    let port = server.port();
    let cancel = CancellationToken::new();
    cancel.cancel();

    assert!(server.wait(&"e".repeat(43), &cancel).await.is_err());

    let rebound = TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("listener released");
    drop(rebound);
}

#[tokio::test]
async fn wrong_state_does_not_consume_the_mcp_callback_server() {
    let server = CallbackServer::bind().await.expect("callback server");
    let port = server.port();
    let cancel = CancellationToken::new();
    let expected = "e".repeat(43);
    let wrong = "w".repeat(43);
    let waiter_state = expected.clone();
    let waiter = tokio::spawn(async move { server.wait(&waiter_state, &cancel).await });

    send_callback(port, &wrong).await;
    assert!(!waiter.is_finished());
    send_callback(port, &expected).await;

    let result = waiter.await.unwrap().unwrap();
    assert_eq!(result.code.as_str(), format!("code-{expected}"));
}
