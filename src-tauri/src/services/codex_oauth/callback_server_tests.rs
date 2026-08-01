use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::*;

const STATE: &str = "0123456789abcdef0123456789abcdef";

fn request(code: &str, state: &str) -> String {
    format!("GET /auth/callback?code={code}&state={state} HTTP/1.1\r\nHost: localhost\r\n\r\n")
}

#[tokio::test]
async fn invalid_callback_can_be_followed_by_valid_callback() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { accept_until_valid(&listener, STATE).await });

    send(address, &request("bad", "wrong")).await;
    send(address, &request("good", STATE)).await;

    let result = task.await.unwrap().unwrap();
    assert!(
        crate::services::codex_oauth::token::constant_time_secret_eq(
            result.code.as_bytes(),
            b"good"
        )
    );
}

#[tokio::test]
async fn binding_falls_back_when_the_first_port_is_busy() {
    let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let occupied_port = occupied.local_addr().unwrap().port();

    let listener = bind_first_available(&[occupied_port, 0]).await.unwrap();

    assert_ne!(listener.local_addr().unwrap().port(), occupied_port);
}

#[tokio::test]
async fn authenticated_refusal_ends_the_callback_wait() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { accept_until_valid(&listener, STATE).await });
    let request = format!("GET /auth/callback?error=access_denied&state={STATE} HTTP/1.1\r\n\r\n");

    send(address, &request).await;

    assert_eq!(task.await.unwrap().unwrap_err(), "callback OAuth refusé");
}

#[tokio::test]
async fn silent_connection_does_not_delay_the_valid_callback() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { accept_until_valid(&listener, STATE).await });
    let _silent = TcpStream::connect(address).await.unwrap();

    let started = std::time::Instant::now();
    send(address, &request("good", STATE)).await;
    let result = tokio::time::timeout(Duration::from_secs(1), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(
        crate::services::codex_oauth::token::constant_time_secret_eq(
            result.code.as_bytes(),
            b"good"
        )
    );
}

async fn send(address: std::net::SocketAddr, request: &str) {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
}
