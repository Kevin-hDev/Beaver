use zeroize::Zeroizing;

use super::diagnostics::{self, ReasonCode, ReasoningDecision};
use super::fingerprint::{opaque_hmac_with_key, FingerprintContext};

#[test]
fn opaque_fingerprints_are_scoped_to_the_session_and_turn() {
    let key = Zeroizing::new(vec![7; 32]);
    let opaque = b"sentinel-encrypted-content";
    let first = opaque_hmac_with_key(
        &key,
        FingerprintContext {
            session_id: "session-a",
            turn_id: "turn-a",
            contract_id: "ollama-native-v1",
        },
        opaque,
    );
    let same = opaque_hmac_with_key(
        &key,
        FingerprintContext {
            session_id: "session-a",
            turn_id: "turn-a",
            contract_id: "ollama-native-v1",
        },
        opaque,
    );
    let other_session = opaque_hmac_with_key(
        &key,
        FingerprintContext {
            session_id: "session-b",
            turn_id: "turn-a",
            contract_id: "ollama-native-v1",
        },
        opaque,
    );
    let other_turn = opaque_hmac_with_key(
        &key,
        FingerprintContext {
            session_id: "session-a",
            turn_id: "turn-b",
            contract_id: "ollama-native-v1",
        },
        opaque,
    );
    assert_eq!(first, same);
    assert_ne!(first, other_session);
    assert_ne!(first, other_turn);
}

#[test]
fn unavailable_fingerprint_keeps_only_non_sensitive_counters() {
    let diagnostic = diagnostics::record(
        ReasoningDecision::Captured,
        ReasonCode::Captured,
        2,
        42,
        Err(super::fingerprint::FingerprintError::Unavailable),
    );
    let rendered = serde_json::to_string(&diagnostic).unwrap();
    assert_eq!(diagnostic.code, ReasonCode::Captured);
    assert!(!rendered.contains("sentinel"));
    assert!(!rendered.contains("encrypted-content"));
}
