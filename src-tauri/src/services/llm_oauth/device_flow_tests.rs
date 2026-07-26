use super::*;

fn authorization(expires_in_seconds: u64) -> DeviceAuthorization {
    DeviceAuthorization {
        device_code: Zeroizing::new(uuid::Uuid::new_v4().to_string()),
        user_code: "CODE".to_string(),
        verification_uri: "https://auth.kimi.com/activate".to_string(),
        verification_uri_complete: None,
        interval_seconds: 1,
        expires_in_seconds,
    }
}

fn config() -> DeviceFlowConfig {
    DeviceFlowConfig {
        provider: LlmOAuthProvider::Kimi,
        client_id: "public-client",
        device_url: "https://auth.kimi.com/device",
        token_url: "https://auth.kimi.com/token",
        scope: None,
    }
}

#[test]
fn rejects_untrusted_verification_urls() {
    assert!(!trusted_verification_url(
        LlmOAuthProvider::Kimi,
        "https://evil.test/login"
    ));
    assert!(trusted_verification_url(
        LlmOAuthProvider::Kimi,
        "https://auth.kimi.com/activate"
    ));
}

#[test]
fn accepts_kimi_response_with_only_complete_verification_url() {
    let wire = DeviceWire {
        device_code: uuid::Uuid::new_v4().to_string(),
        user_code: "CODE".to_string(),
        verification_uri: String::new(),
        verification_uri_complete: Some("https://auth.kimi.com/activate?code=fixture".to_string()),
        expires_in: None,
        interval: Some(5),
    };
    assert!(validate_wire(LlmOAuthProvider::Kimi, &wire).is_ok());
}

#[test]
fn progress_url_never_exposes_the_device_query() {
    let authorization = DeviceAuthorization {
        device_code: Zeroizing::new(uuid::Uuid::new_v4().to_string()),
        user_code: "CODE".to_string(),
        verification_uri: String::new(),
        verification_uri_complete: Some(
            "https://auth.kimi.com/activate?code=sensitive".to_string(),
        ),
        interval_seconds: 5,
        expires_in_seconds: 300,
    };
    let public = public_verification_url(&authorization).unwrap();
    assert_eq!(public, "https://auth.kimi.com/activate");
    assert!(!public.contains("code="));
}

#[test]
fn handles_all_rfc8628_polling_states() {
    assert_eq!(
        poll_error_action(Some("authorization_pending")),
        Ok(PollAction::Continue)
    );
    assert_eq!(
        poll_error_action(Some("slow_down")),
        Ok(PollAction::SlowDown)
    );
    assert_eq!(
        poll_error_action(Some("access_denied")),
        Err(OAuthFailure::Denied)
    );
    assert_eq!(
        poll_error_action(Some("authorization_denied")),
        Err(OAuthFailure::Denied)
    );
    assert_eq!(
        poll_error_action(Some("expired_token")),
        Err(OAuthFailure::Expired)
    );
    assert_eq!(
        poll_error_action(Some("unknown")),
        Err(OAuthFailure::Generic)
    );
}

#[tokio::test]
async fn cancelled_device_flow_stops_before_network_access() {
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(matches!(
        poll(&config(), &authorization(60), &cancel).await,
        Err(OAuthFailure::Cancelled)
    ));
}

#[tokio::test]
async fn expired_device_flow_stops_before_network_access() {
    let cancel = CancellationToken::new();
    assert!(matches!(
        poll(&config(), &authorization(0), &cancel).await,
        Err(OAuthFailure::Expired)
    ));
}
