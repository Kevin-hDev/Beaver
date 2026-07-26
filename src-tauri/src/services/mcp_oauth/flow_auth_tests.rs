use super::verify_state_constant_time;

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
