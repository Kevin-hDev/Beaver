use super::*;

#[test]
fn codex_transport_errors_use_stable_codes() {
    assert_eq!(
        secure_http_error(SecureHttpError::Configuration),
        "provider_configuration_invalid"
    );
    assert_eq!(
        secure_http_error(SecureHttpError::Status),
        "provider_request_rejected"
    );
    assert_eq!(
        secure_http_error(SecureHttpError::Request),
        "provider_connection_failed"
    );
}
