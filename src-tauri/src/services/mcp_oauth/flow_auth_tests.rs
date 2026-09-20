use super::verify_state_constant_time;
use zeroize::Zeroizing;

const STATE: &str = "0123456789abcdef0123456789abcdef0123456789A";

#[test]
fn fixed_oauth_state_matches_itself() {
    assert!(verify_state_constant_time(STATE, STATE).is_ok());
}

#[test]
fn oauth_state_rejects_different_and_variable_lengths() {
    let different = "1123456789abcdef0123456789abcdef0123456789A";
    assert!(verify_state_constant_time(STATE, different).is_err());
    assert!(verify_state_constant_time(STATE, "short").is_err());
    assert!(verify_state_constant_time("short", "short").is_err());
}

#[tokio::test]
async fn dcr_and_code_exchange_refuse_foreign_destinations_before_send() {
    let resource = "https://mcp.notion.com/mcp";
    assert!(super::register_client(
        "notion",
        resource,
        "https://evil.example/register",
        "http://127.0.0.1:1234/callback",
    )
    .await
    .is_err());
    let verifier = Zeroizing::new("fixture-verifier".to_string());
    assert!(super::exchange_code(
        "notion",
        "https://mcp.notion.com",
        "https://evil.example/token",
        "fixture-code",
        "fixture-client",
        None,
        &verifier,
        "http://127.0.0.1:1234/callback",
        resource,
    )
    .await
    .is_err());
}
