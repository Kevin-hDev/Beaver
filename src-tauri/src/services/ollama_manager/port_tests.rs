use std::net::TcpListener;

use super::port::{DefaultOllamaPortAllocator, OllamaPortAllocator};

#[test]
fn allocator_returns_loopback_endpoint_and_honors_exclusions() {
    let allocator = DefaultOllamaPortAllocator::new();
    let first = allocator.allocate_loopback(&[]).expect("first port");
    let second = allocator
        .allocate_loopback(&[first.port()])
        .expect("second port");
    assert_ne!(first.port(), second.port());
    assert!(first.as_http_url().starts_with("http://127.0.0.1:"));
}

#[tokio::test]
async fn absent_external_daemon_is_not_a_validation_result() {
    let allocator = DefaultOllamaPortAllocator::with_external_port(0);
    assert_eq!(allocator.detect_external().await.expect("probe"), None);
}

#[cfg(windows)]
#[tokio::test]
async fn a_slow_closed_windows_port_does_not_block_owned_startup() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("temporary listener");
    let port = listener.local_addr().expect("address").port();
    drop(listener);

    let allocator = DefaultOllamaPortAllocator::with_external_port(port);
    assert_eq!(allocator.detect_external().await.expect("probe"), None);
}

#[tokio::test]
async fn listening_external_daemon_is_only_observed() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("listener");
    let port = listener.local_addr().expect("address").port();
    let allocator = DefaultOllamaPortAllocator::with_external_port(port);
    let endpoint = allocator
        .detect_external()
        .await
        .expect("external probe")
        .expect("external endpoint");
    assert_eq!(endpoint.port(), port);
    drop(listener);
}
