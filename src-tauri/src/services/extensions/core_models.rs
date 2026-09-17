use serde_json::Value;

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

pub(super) async fn call(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    match method {
        "models.list" => super::core_model_catalog::list(params).await,
        "models.generate" => super::core_model_generation::generate(context, params).await,
        _ => Err(ExtensionBridgeError::MethodUnavailable),
    }
}
