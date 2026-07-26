//! Tests de VALIDATION PURE pour api_keys.
//!
//! On ne teste QUE les fonctions qui valident l'entrée SANS toucher au STATE
//! global (LazyLock non réinitialisable) ni au keyring OS. Les fonctions qui
//! dépendent du STATE (set_key, get_key, set_mcp_token...) sont couvertes par
//! des tests d'intégration séparés.
//!
//! Stratégie pour set_mcp_token/set_key_raw : leur validation d'entrée échoue
//! AVANT de locker le STATE, donc un id invalide ne pollue pas l'état global.

use super::validate::{validate_key_input, validate_provider};

// --- validate_provider -------------------------------------------------------

#[test]
fn validate_provider_accepts_known_llm_provider() {
    // "groq" est dans le catalog LLM.
    assert!(validate_provider("groq").is_ok());
}

#[test]
fn validate_provider_accepts_known_search_provider() {
    // "brave" est dans le catalog SEARCH_PROVIDERS.
    assert!(validate_provider("brave").is_ok());
}

#[test]
fn validate_provider_rejects_unknown_id() {
    let res = validate_provider("unknown-provider-xyz");
    assert!(res.is_err(), "un provider inconnu doit être rejeté");
}

#[test]
fn validate_provider_rejects_empty_string() {
    assert!(validate_provider("").is_err());
}

#[test]
fn validate_provider_rejects_suspicious_path_like_id() {
    // Pas d'injection possible : un id avec séparateurs n'est pas un provider.
    assert!(validate_provider("../etc/passwd").is_err());
    assert!(validate_provider("groq; rm -rf /").is_err());
}

// --- validate_key_input -----------------------------------------------------

#[test]
fn validate_key_input_accepts_normal_key() {
    assert!(validate_key_input("groq", "sk-normal-api-key-12345").is_ok());
}

#[test]
fn validate_key_input_rejects_empty_key() {
    assert!(validate_key_input("groq", "").is_err());
}

#[test]
fn validate_key_input_rejects_key_over_max_length() {
    let too_long = "x".repeat(257); // MAX_KEY_LEN = 256
    assert!(validate_key_input("groq", &too_long).is_err());
}

#[test]
fn validate_key_input_accepts_key_at_exact_max_length() {
    let exact = "x".repeat(256);
    assert!(validate_key_input("groq", &exact).is_ok());
}

#[test]
fn validate_key_input_rejects_null_byte() {
    // Caractère de contrôle 0x00 (< 0x20, != '\n') → rejeté.
    assert!(validate_key_input("groq", "abc\x00def").is_err());
}

#[test]
fn validate_key_input_rejects_other_control_chars() {
    // 0x01, 0x1F (avant 0x20) doivent être rejetés.
    assert!(validate_key_input("groq", "key\x01bad").is_err());
    assert!(validate_key_input("groq", "key\x1Fbad").is_err());
}

#[test]
fn validate_key_input_allows_newline() {
    // '\n' (0x0A) est explicitement autorisé (exceptions dans le code).
    assert!(validate_key_input("groq", "line1\nline2").is_ok());
}

#[test]
fn validate_key_input_rejects_key_for_unknown_provider() {
    // La validation du provider est faite en premier (fail fast).
    assert!(validate_key_input("unknown", "sk-12345").is_err());
}
