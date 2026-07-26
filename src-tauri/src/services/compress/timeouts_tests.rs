use super::timeouts::{compression_idle_timeout, compression_request_timeout};
use crate::services::secure_http::{AuthenticatedClient, MAX_AUTHENTICATED_TIMEOUT};

#[test]
fn compression_timeouts_are_ten_minutes() {
    assert_eq!(compression_request_timeout().as_secs(), 600);
    assert_eq!(compression_idle_timeout().as_secs(), 600);
}

#[test]
fn authenticated_client_accepts_compression_timeout() {
    assert_eq!(compression_request_timeout(), MAX_AUTHENTICATED_TIMEOUT);
    assert!(AuthenticatedClient::new(compression_request_timeout()).is_ok());
}
