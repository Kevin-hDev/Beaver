use super::{constant_time_token_eq, valid_health_token};
use crate::updater_worker::WorkerError;

#[test]
fn health_tokens_are_exact_lowercase_hex() {
    let valid = "ab".repeat(32);
    assert!(valid_health_token(&valid));
    assert!(!valid_health_token(&"ab".repeat(31)));
    assert!(!valid_health_token(&"AB".repeat(32)));
    assert!(!valid_health_token(&"zz".repeat(32)));
}

#[test]
fn health_token_comparison_checks_exact_values() {
    let token = "01".repeat(32);
    assert!(constant_time_token_eq(&token, &token));
    assert!(!constant_time_token_eq(&token, &"02".repeat(32)));
    assert!(!constant_time_token_eq(&token, "short"));
}

#[test]
fn worker_errors_never_reveal_internal_details() {
    assert_eq!(WorkerError.to_string(), "update failed");
    assert!(!WorkerError.to_string().contains('/'));
    assert!(!WorkerError.to_string().contains('\\'));
}
