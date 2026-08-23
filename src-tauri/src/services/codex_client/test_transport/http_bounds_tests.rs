use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncWriteExt;

use super::super::sensitive_buffer::SensitiveBuffer;

const REQUEST_LIMIT: usize = 64 * 1024;
const FINAL_FRAGMENT_SLACK: usize = 512;

#[tokio::test]
async fn request_at_the_exact_limit_is_accepted_and_erased() {
    let (request, request_zeroized) = request_fixture(REQUEST_LIMIT);
    let read_zeroized = Arc::new(AtomicBool::new(false));

    let result = read_fragmented(&request, Arc::clone(&read_zeroized)).await;

    let bytes = result.expect("an exact-limit request is accepted");
    assert_eq!(bytes.len(), REQUEST_LIMIT);
    drop(bytes);
    drop(request);
    assert!(read_zeroized.load(Ordering::SeqCst));
    assert!(request_zeroized.load(Ordering::SeqCst));
}

#[tokio::test]
async fn request_one_byte_over_the_limit_is_rejected_and_erased() {
    let (request, request_zeroized) = request_fixture(REQUEST_LIMIT + 1);
    let read_zeroized = Arc::new(AtomicBool::new(false));

    let result = read_fragmented(&request, Arc::clone(&read_zeroized)).await;

    match result {
        Ok(_) => panic!("limit + 1 must fail"),
        Err(error) => assert_eq!(error, super::invalid()),
    }
    drop(request);
    assert!(read_zeroized.load(Ordering::SeqCst));
    assert!(request_zeroized.load(Ordering::SeqCst));
}

async fn read_fragmented(
    request: &[u8],
    zeroized: Arc<AtomicBool>,
) -> Result<SensitiveBuffer, String> {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind bounded HTTP fixture");
    let address = listener.local_addr().expect("read loopback address");
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("accept HTTP fixture");
        super::read_request(&mut socket, zeroized).await
    });
    let mut client = tokio::net::TcpStream::connect(address)
        .await
        .expect("connect HTTP fixture");
    let split = REQUEST_LIMIT - FINAL_FRAGMENT_SLACK;
    client
        .write_all(&request[..split])
        .await
        .expect("write first bounded fragment");
    tokio::time::sleep(Duration::from_millis(25)).await;
    client
        .write_all(&request[split..])
        .await
        .expect("write final bounded fragment");
    client.shutdown().await.expect("close HTTP fixture");

    tokio::time::timeout(Duration::from_secs(2), server)
        .await
        .expect("bounded HTTP fixture timeout")
        .expect("HTTP fixture task completes")
}

fn request_fixture(total_bytes: usize) -> (SensitiveBuffer, Arc<AtomicBool>) {
    let mut body_bytes = total_bytes;
    let (header, body_bytes) = loop {
        let header = format!("POST /responses HTTP/1.1\r\nContent-Length: {body_bytes}\r\n\r\n");
        match header.len().checked_add(body_bytes) {
            Some(total) if total == total_bytes => break (header, body_bytes),
            Some(total) if total > total_bytes => body_bytes -= total - total_bytes,
            _ => panic!("fixture has an exact bounded size"),
        }
    };
    let zeroized = Arc::new(AtomicBool::new(false));
    let mut request = SensitiveBuffer::with_capacity(total_bytes, Arc::clone(&zeroized));
    request.extend_from_slice(header.as_bytes());
    request.resize(header.len() + body_bytes, b'x');
    assert_eq!(request.len(), total_bytes);
    (request, zeroized)
}
