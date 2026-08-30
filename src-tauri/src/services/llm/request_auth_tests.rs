use super::{request_auth, route_profile::ApiKeyHeader};

#[test]
fn anthropic_api_key_header_is_marked_sensitive_before_the_request_is_built() {
    let client = reqwest::Client::new();
    let request = request_auth::apply(
        client.get("https://example.com"),
        ApiKeyHeader::XApiKey,
        "fixture-secret",
    )
    .build()
    .expect("valid fixture request");

    let header = request
        .headers()
        .get("x-api-key")
        .expect("Anthropic API key header");
    assert_eq!(header, "fixture-secret");
    assert!(header.is_sensitive());
}
