use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncWriteExt;

use super::super::sensitive_buffer::SensitiveBuffer;

const HANDSHAKE_LIMIT: usize = 16 * 1024;
const FINAL_FRAGMENT_SLACK: usize = 512;

#[tokio::test]
async fn handshake_at_the_exact_limit_is_accepted_and_erased() {
    let (request, request_zeroized) = handshake_fixture(HANDSHAKE_LIMIT);

    let result = accept_fragmented(&request).await;

    assert!(result.is_ok(), "an exact-limit handshake is accepted");
    drop(request);
    assert!(request_zeroized.load(Ordering::SeqCst));
}

#[tokio::test]
async fn handshake_one_byte_over_the_limit_is_rejected_and_erased() {
    let (request, request_zeroized) = handshake_fixture(HANDSHAKE_LIMIT + 1);

    let result = accept_fragmented(&request).await;

    assert_eq!(result.expect_err("limit + 1 must fail"), super::invalid());
    drop(request);
    assert!(request_zeroized.load(Ordering::SeqCst));
}

async fn accept_fragmented(request: &[u8]) -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind bounded WebSocket fixture");
    let address = listener.local_addr().expect("read loopback address");
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("accept WS fixture");
        super::accept(socket, "bounded-session").await.map(drop)
    });
    let mut client = tokio::net::TcpStream::connect(address)
        .await
        .expect("connect WS fixture");
    let split = HANDSHAKE_LIMIT - FINAL_FRAGMENT_SLACK;
    client
        .write_all(&request[..split])
        .await
        .expect("write first bounded fragment");
    tokio::time::sleep(Duration::from_millis(25)).await;
    client
        .write_all(&request[split..])
        .await
        .expect("write final bounded fragment");
    client.shutdown().await.expect("close WS fixture");

    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("bounded WS fixture timeout")
        .expect("WS fixture task completes")
}

fn handshake_fixture(total_bytes: usize) -> (SensitiveBuffer, Arc<AtomicBool>) {
    const PREFIX: &str = concat!(
        "GET / HTTP/1.1\r\n",
        "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n",
        "X-Padding: "
    );
    const SUFFIX: &str = "\r\n\r\n";
    let padding = total_bytes
        .checked_sub(PREFIX.len() + SUFFIX.len())
        .expect("fixture limit holds a valid handshake");
    let zeroized = Arc::new(AtomicBool::new(false));
    let mut request = SensitiveBuffer::with_capacity(total_bytes, Arc::clone(&zeroized));
    request.extend_from_slice(PREFIX.as_bytes());
    request.resize(PREFIX.len() + padding, b'x');
    request.extend_from_slice(SUFFIX.as_bytes());
    assert_eq!(request.len(), total_bytes);
    (request, zeroized)
}
