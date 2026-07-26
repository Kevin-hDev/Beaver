use std::ffi::OsString;
use std::fs;
use std::time::Duration;

use super::{token_in_arguments, HealthToken};

#[test]
fn generates_valid_unique_tokens_and_consumes_ack() {
    let root = tempfile::tempdir().unwrap();
    let first = HealthToken::generate(root.path().to_path_buf()).unwrap();
    let second = HealthToken::generate(root.path().to_path_buf()).unwrap();
    assert_eq!(first.value().len(), 64);
    assert_ne!(first.value(), second.value());

    let path = first.ack_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"ok").unwrap();
    first.wait_for(Duration::from_millis(10)).unwrap();
    assert!(!path.exists());
}

#[test]
fn rejects_invalid_ack_and_matches_token_argument_exactly() {
    let root = tempfile::tempdir().unwrap();
    let token = HealthToken::generate(root.path().to_path_buf()).unwrap();
    let path = token.ack_path();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not-ok").unwrap();
    assert!(token.wait_for(Duration::from_millis(10)).is_err());

    let arguments = vec![
        OsString::from("--clgo-update-health"),
        OsString::from(token.value()),
    ];
    assert!(token_in_arguments(&arguments, token.value()));
    assert!(!token_in_arguments(&arguments, &"00".repeat(32)));
}
