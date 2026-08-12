use super::*;

#[tokio::test]
async fn cancelled_wait_releases_the_owned_listener_before_returning() {
    let server = CallbackServer::bind().await.expect("callback server");
    let port = server.port();
    let cancel = CancellationToken::new();
    cancel.cancel();

    assert!(server.wait(&cancel).await.is_err());

    let rebound = TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("listener released");
    drop(rebound);
}
