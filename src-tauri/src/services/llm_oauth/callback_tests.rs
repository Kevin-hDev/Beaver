use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const STATE: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGH123456789";

#[test]
fn accepts_valid_callback() {
    let request =
        format!("GET /callback?code=abc&state={STATE} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(
        parse_request(request.as_bytes(), STATE).unwrap().as_str(),
        "abc"
    );
}

#[test]
fn rejects_wrong_path_and_state() {
    let wrong_path = format!("GET /other?code=abc&state={STATE} HTTP/1.1\r\n\r\n");
    let wrong_state = "abcdefghijklmnopqrstuvwxyzABCDEFGH123456788";
    let wrong = format!("GET /callback?code=abc&state={wrong_state} HTTP/1.1\r\n\r\n");
    assert!(parse_request(wrong_path.as_bytes(), STATE).is_err());
    assert!(parse_request(wrong.as_bytes(), STATE).is_err());
}

#[test]
fn validates_state_in_constant_time_helper() {
    assert!(verify_state(STATE, STATE).is_ok());
    assert!(verify_state("short", STATE).is_err());
}

#[tokio::test]
async fn occupied_callback_port_reports_unavailable_for_device_fallback() {
    // An OS-assigned port avoids Windows ranges where even the first bind is forbidden.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let occupied = listener.local_addr().unwrap().to_string();
    let result = CallbackServer::bind_at(Zeroizing::new(STATE.to_string()), &occupied).await;
    assert!(matches!(result, Err(OAuthFailure::Generic)));
    drop(listener);
}

#[tokio::test]
async fn cancelled_wait_releases_the_owned_listener_before_returning() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = CallbackServer {
        listener,
        expected_state: Zeroizing::new(STATE.to_string()),
    };
    let cancel = CancellationToken::new();
    let waiter_cancel = cancel.clone();
    let wait = tokio::spawn(async move { server.wait(&waiter_cancel).await });

    let mut incomplete = tokio::net::TcpStream::connect(address).await.unwrap();
    incomplete.write_all(b"GET /callback?").await.unwrap();
    tokio::task::yield_now().await;
    cancel.cancel();

    assert_eq!(wait.await.unwrap(), Err(OAuthFailure::Cancelled));
    let mut response = Vec::new();
    tokio::time::timeout(
        Duration::from_secs(1),
        incomplete.read_to_end(&mut response),
    )
    .await
    .expect("cancellation must close incomplete connections")
    .unwrap();
    assert!(response.is_empty());
    let rebound = tokio::net::TcpListener::bind(address).await.unwrap();
    drop(rebound);
}

#[tokio::test]
async fn incomplete_connection_does_not_block_a_valid_callback() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = CallbackServer {
        listener,
        expected_state: Zeroizing::new(STATE.to_string()),
    };
    let cancel = CancellationToken::new();
    let wait = tokio::spawn(async move { server.wait(&cancel).await });

    let mut incomplete = tokio::net::TcpStream::connect(address).await.unwrap();
    incomplete.write_all(b"GET /callback?").await.unwrap();

    let mut valid = tokio::net::TcpStream::connect(address).await.unwrap();
    valid
        .write_all(
            format!("GET /callback?code=abc&state={STATE} HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .as_bytes(),
        )
        .await
        .unwrap();
    let code = tokio::time::timeout(Duration::from_secs(1), wait)
        .await
        .expect("a valid callback must not wait for the incomplete connection")
        .unwrap()
        .unwrap();
    let mut response = Vec::new();
    valid.read_to_end(&mut response).await.unwrap();
    assert_eq!(code.as_str(), "abc");
    assert!(response.starts_with(b"HTTP/1.1 200 OK"));
}

#[tokio::test]
async fn invalid_callback_attempts_are_bounded() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let wait = tokio::spawn(async move { accept_until_valid(&listener, STATE).await });

    for _ in 0..MAX_CALLBACK_ATTEMPTS {
        let mut invalid = tokio::net::TcpStream::connect(address).await.unwrap();
        invalid
            .write_all(b"GET /callback?code=abc&state=wrong HTTP/1.1\r\n\r\n")
            .await
            .unwrap();
        let mut response = Vec::new();
        invalid.read_to_end(&mut response).await.unwrap();
        assert!(response.starts_with(b"HTTP/1.1 400 Bad Request"));
    }

    assert!(matches!(
        tokio::time::timeout(Duration::from_secs(1), wait)
            .await
            .expect("the callback attempt limit must stop the server")
            .unwrap(),
        Err(OAuthFailure::Generic)
    ));
}
