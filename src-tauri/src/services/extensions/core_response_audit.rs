use super::access_log::AccessResult;
use super::core_bridge::{CoreResponse, ExtensionBridgeError};

pub(super) fn record_outcome(
    outcome: Result<CoreResponse, ExtensionBridgeError>,
    writer: impl FnOnce(AccessResult) -> Result<(), String>,
) -> Result<CoreResponse, ExtensionBridgeError> {
    writer(classify(&outcome)).map_err(|_| ExtensionBridgeError::Failed)?;
    outcome
}

fn classify(outcome: &Result<CoreResponse, ExtensionBridgeError>) -> AccessResult {
    match outcome {
        Ok(_) => AccessResult::Granted,
        Err(ExtensionBridgeError::Denied) => AccessResult::Denied,
        Err(ExtensionBridgeError::Failed) => AccessResult::Failed,
        Err(ExtensionBridgeError::Revoked) => AccessResult::Revoked,
        Err(ExtensionBridgeError::Timeout) => AccessResult::Timeout,
    }
}
