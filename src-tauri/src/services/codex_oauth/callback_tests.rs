use super::*;
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

#[test]
fn accepts_only_a_state_authenticated_refusal() {
    let refused = format!("GET /auth/callback?error=access_denied&state={STATE} HTTP/1.1\r\n\r\n");
    let wrong_state = "GET /auth/callback?error=access_denied&state=wrong HTTP/1.1\r\n\r\n";

    assert_eq!(
        parse_callback_bytes(refused.as_bytes(), STATE).unwrap_err(),
        CallbackError::Refused
    );
    assert_eq!(
        parse_callback_bytes(wrong_state.as_bytes(), STATE).unwrap_err(),
        CallbackError::Invalid
    );
}
