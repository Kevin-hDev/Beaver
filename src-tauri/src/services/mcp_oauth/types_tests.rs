use super::TokenResponse;

#[test]
fn oauth_token_response_rejects_non_bearer_and_unbounded_fields() {
    let mut response = TokenResponse {
        access_token: "access".to_string(),
        refresh_token: None,
        expires_in: Some(3600),
        token_type: "DPoP".to_string(),
    };
    assert!(response.validate().is_err());
    response.token_type = "Bearer".to_string();
    response.access_token = "x".repeat(16_385);
    assert!(response.validate().is_err());
    response.access_token = "access".to_string();
    response.expires_in = Some(i64::MAX);
    assert!(response.validate().is_err());
}
