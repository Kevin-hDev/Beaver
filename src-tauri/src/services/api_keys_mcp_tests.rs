//! Tests de VALIDATION D'ENTRÉE pour set_mcp_token / get_mcp_token /
//! delete_mcp_token / has_mcp_token.
//!
//! Les fonctions set_mcp_token / get_mcp_token / delete_mcp_token valident
//! l'id du connecteur AVANT de locker le STATE global. Donc :
//!   - id invalide  -> Err("identifiant connecteur invalide")  (avant STATE)
//!   - id valide    -> Err("coffre indisponible")              (validation OK)
//!
//! Cette distinction nous permet de tester la validation purement, sans
//! initialiser le vault ni toucher au keyring OS.

use super::{delete_mcp_token, get_mcp_token, has_mcp_token, set_mcp_token};

/// Un id valide doit passer la validation puis échouer sur le coffre absent.
/// On l'utilise pour distinguer "validation OK" des autres.
const VAULT_UNINIT: &str = "coffre indisponible";

/// Helper : prouve que l'id passe la validation (échec attendu = STATE absent).
fn passes_validation(connector_id: &str) -> bool {
    // set_mcp_token valide d'abord l'id, puis tente un lock STATE. Si l'id est
    // valide, on atteint l'erreur générique du coffre absent.
    match set_mcp_token(connector_id, "{\"token\":\"x\"}") {
        Err(msg) if msg.contains(VAULT_UNINIT) => true,
        Ok(_) => true,
        Err(_) => false,
    }
}

#[test]
fn rejects_slash_in_connector_id() {
    let err = set_mcp_token("conn/evil", "{\"token\":\"x\"}").unwrap_err();
    assert!(
        !err.contains(VAULT_UNINIT),
        "un id avec slash doit échouer à la validation, pas au STATE"
    );
}

#[test]
fn rejects_space_in_connector_id() {
    let err = set_mcp_token("conn evil", "{\"token\":\"x\"}").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn rejects_dot_in_connector_id() {
    let err = set_mcp_token("conn.evil", "{\"token\":\"x\"}").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn rejects_path_traversal_in_connector_id() {
    // '../' ou '..' ne doit jamais pouvoir cibler une autre entrée vault.
    let err = set_mcp_token("../escape", "{\"token\":\"x\"}").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn rejects_empty_connector_id() {
    let err = set_mcp_token("", "{\"token\":\"x\"}").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn rejects_overlong_connector_id() {
    let long_id = "a".repeat(65); // MAX_CONNECTOR_ID = 64
    let err = set_mcp_token(&long_id, "{\"token\":\"x\"}").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn rejects_empty_token() {
    // id valide + token vide -> erreur "token vide" (avant STATE).
    let err = set_mcp_token("valid-id", "").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn accepts_alphanumeric_connector_id() {
    assert!(passes_validation("conn123"));
}

#[test]
fn accepts_dash_and_underscore() {
    assert!(passes_validation("my-connector_id"));
}

#[test]
fn accepts_id_at_max_length() {
    let exact = "a".repeat(64); // MAX_CONNECTOR_ID = 64
    assert!(passes_validation(&exact));
}

#[test]
fn get_mcp_token_rejects_invalid_id_before_state() {
    let err = get_mcp_token("bad id").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn delete_mcp_token_rejects_invalid_id_before_state() {
    let err = delete_mcp_token("bad/id").unwrap_err();
    assert!(!err.contains(VAULT_UNINIT));
}

#[test]
fn has_mcp_token_returns_false_for_invalid_id() {
    // has_mcp_token appelle validate_mcp_connector_id et retourne false si
    // la validation échoue (jamais d'erreur).
    assert!(!has_mcp_token("bad id"));
    assert!(!has_mcp_token("../evil"));
}
