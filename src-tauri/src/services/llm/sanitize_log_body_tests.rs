//! Tests de sanitize_log_body : troncature à 200 caractères (UTF-8 safe),
//! rédaction des secrets (key/token/secret/password/authorization/api_key),
//! remplacement des caractères de contrôle.
//!
//! Objectif de sécurité : aucun secret ne doit atterrir dans un log, même
//! si le body HTTP brut contient un token en clair.

use super::sanitize_log_body;

// --- Troncature -------------------------------------------------------------

#[test]
fn short_body_unchanged() {
    assert_eq!(sanitize_log_body("hello world"), "hello world");
}

#[test]
fn truncates_body_over_200_chars() {
    let body = "x".repeat(500);
    let result = sanitize_log_body(&body);
    assert!(
        result.chars().count() <= 200,
        "le body doit être tronqué à 200 caractères max"
    );
}

#[test]
fn truncation_is_utf8_safe() {
    // Body avec des multi-bytes : la troncature ne doit pas couper au milieu
    // d'un caractère (char_indices).
    let prefix = "é".repeat(199); // 199 'é' (multi-bytes)
    let body = format!("{prefix}AB"); // 201 chars
    let result = sanitize_log_body(&body);
    // Tous les caractères du résultat doivent être valides (pas de moitié de
    // codepoint). On vérifie juste que ça ne panique pas et reste lisible.
    assert!(result.chars().count() <= 200);
}

#[test]
fn body_at_exact_200_chars_not_truncated() {
    let body = "a".repeat(200);
    let result = sanitize_log_body(&body);
    assert_eq!(result.chars().count(), 200);
}

// --- Rédaction des secrets --------------------------------------------------

#[test]
fn redacts_authorization_bearer() {
    let body = r#"{"authorization":"Bearer sk-secret123"}"#;
    let result = sanitize_log_body(body);
    assert!(
        !result.contains("sk-secret123"),
        "le token ne doit pas apparaître dans le log"
    );
    assert!(
        result.contains("[REDACTED]"),
        "le token doit être remplacé par [REDACTED]"
    );
}

#[test]
fn redacts_api_key_json() {
    let body = r#"{"api_key":"sk-abc123def456"}"#;
    let result = sanitize_log_body(body);
    assert!(!result.contains("sk-abc123def456"));
}

#[test]
fn redacts_password() {
    let body = r#"{"password":"supersecret"}"#;
    let result = sanitize_log_body(body);
    assert!(!result.contains("supersecret"));
}

#[test]
fn redacts_query_param_style() {
    // Format URL-encoded : key=value&...
    let body = "token=abc123&user=bob";
    let result = sanitize_log_body(body);
    assert!(!result.contains("abc123"));
}

#[test]
fn redacts_secret_with_equals() {
    let body = "secret=my-top-secret-value";
    let result = sanitize_log_body(body);
    assert!(!result.contains("my-top-secret-value"));
}

#[test]
fn key_word_without_delimiter_not_redacted_value() {
    // "key" sans ':' ni '=' après → rien à rédactionner (pas de valeur
    // identifiable). Le test documente ce comportement sans crash.
    let body = "the keyword is interesting";
    let result = sanitize_log_body(body);
    assert!(result.contains("keyword"));
}

// --- Caractères de contrôle -------------------------------------------------

#[test]
fn replaces_control_chars_with_spaces() {
    let body = "line1\nline2\ttabbed";
    let result = sanitize_log_body(body);
    // \n et \t sont des control chars → remplacés par espaces.
    assert!(!result.contains('\n'));
    assert!(!result.contains('\t'));
}

#[test]
fn redacts_secret_containing_control_chars() {
    // Le nettoyage des control chars se fait AVANT la rédaction, donc un
    // secret avec des retours chariot doit quand même être masqué.
    let body = "{\"token\":\"abc\ndef\"}";
    let result = sanitize_log_body(body);
    assert!(!result.contains("abc"));
}
