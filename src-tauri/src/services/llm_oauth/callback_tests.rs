use super::*;

const STATE: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGH123456789";

#[test]
fn accepts_valid_callback() {
    let request =
        format!("GET /callback?code=abc&state={STATE} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert_eq!(
        parse_request(request.as_bytes(), STATE).unwrap().as_str(),
        "abc"
    );
}

#[test]
fn rejects_wrong_path_and_state() {
    let wrong_path = format!("GET /other?code=abc&state={STATE} HTTP/1.1\r\n\r\n");
    let wrong_state = "abcdefghijklmnopqrstuvwxyzABCDEFGH123456788";
    let wrong = format!("GET /callback?code=abc&state={wrong_state} HTTP/1.1\r\n\r\n");
    assert!(parse_request(wrong_path.as_bytes(), STATE).is_err());
    assert!(parse_request(wrong.as_bytes(), STATE).is_err());
}

#[test]
fn validates_state_in_constant_time_helper() {
    assert!(verify_state(STATE, STATE).is_ok());
    assert!(verify_state("short", STATE).is_err());
}

#[tokio::test]
async fn fixed_callback_port_reports_unavailable_for_device_fallback() {
    let listener = tokio::net::TcpListener::bind(BIND_ADDR).await.unwrap();
    let result = start(Zeroizing::new(STATE.to_string()), CancellationToken::new()).await;
    assert!(matches!(result, Err(OAuthFailure::Generic)));
    drop(listener);
}
