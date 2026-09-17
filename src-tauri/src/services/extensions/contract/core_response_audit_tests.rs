use super::access_log::AccessResult;
use super::core_bridge::{CoreResponse, ExtensionBridgeError};
use super::core_response_audit::record_outcome;
use serde_json::json;
use zeroize::Zeroizing;

#[test]
fn audit_failure_withholds_a_secret() {
    let outcome = Ok(CoreResponse::Secret(Zeroizing::new(
        "AUDIT-FAKE-KEY".to_string(),
    )));

    let result = record_outcome(outcome, |_| Err("audit unavailable".to_string()));

    assert!(matches!(result, Err(ExtensionBridgeError::Failed)));
}

#[test]
fn successful_audit_returns_the_original_response() {
    let result = record_outcome(Ok(CoreResponse::Json(json!({"ok": true}))), |access| {
        assert_eq!(access, AccessResult::Granted);
        Ok(())
    });

    assert!(matches!(result, Ok(CoreResponse::Json(value)) if value == json!({"ok": true})));
}

#[test]
fn successful_audit_preserves_every_error_classification() {
    for (error, access_result) in [
        (ExtensionBridgeError::Denied, AccessResult::Denied),
        (ExtensionBridgeError::Failed, AccessResult::Failed),
        (ExtensionBridgeError::Revoked, AccessResult::Revoked),
        (ExtensionBridgeError::Timeout, AccessResult::Timeout),
    ] {
        let result = record_outcome(Err(error), |access| {
            assert_eq!(access, access_result);
            Ok(())
        });

        assert_eq!(result.err(), Some(error));
    }
}

#[test]
fn an_executed_non_secret_operation_is_not_retried_after_audit_failure() {
    let mut audit_attempts = 0;
    let result = record_outcome(Ok(CoreResponse::Json(json!({"done": true}))), |_| {
        audit_attempts += 1;
        Err("audit unavailable".to_string())
    });

    assert!(matches!(result, Err(ExtensionBridgeError::Failed)));
    assert_eq!(audit_attempts, 1);
}
