use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const STATE: &str = "0123456789abcdef0123456789abcdef";

fn request(code: &str, state: &str) -> String {
    format!("GET /auth/callback?code={code}&state={state} HTTP/1.1\r\nHost: localhost\r\n\r\n")
}

#[test]
fn parses_valid_callback_into_zeroizing_code() {
    let result = parse_callback_bytes(request("abc123", STATE).as_bytes(), STATE).unwrap();
    assert_eq!(result.code.as_str(), "abc123");
}

#[test]
fn rejects_wrong_or_non_fixed_state() {
    let wrong = "1123456789abcdef0123456789abcdef";
    assert!(parse_callback_bytes(request("abc", wrong).as_bytes(), STATE).is_err());
    assert!(parse_callback_bytes(request("abc", "short").as_bytes(), STATE).is_err());
}

#[test]
fn rejects_missing_code_and_excessive_request() {
    let missing = format!("GET /auth/callback?state={STATE} HTTP/1.1\r\n\r\n");
    assert!(parse_callback_bytes(missing.as_bytes(), STATE).is_err());
    assert!(parse_callback_bytes(&vec![b'a'; MAX_REQUEST_BYTES + 1], STATE).is_err());
}

#[tokio::test]
async fn invalid_callback_can_be_followed_by_valid_callback() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { accept_until_valid(&listener, STATE).await });

    send(address, &request("bad", "wrong")).await;
    send(address, &request("good", STATE)).await;

    let result = task.await.unwrap().unwrap();
    assert_eq!(result.code.as_str(), "good");
}

async fn send(address: std::net::SocketAddr, request: &str) {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
}
