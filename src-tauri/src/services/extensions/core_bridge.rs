use super::types::{MAX_PROJECT_RESULTS, MAX_SESSION_RESULTS};
use serde::Serialize;
use serde_json::{json, Value};
use std::time::Duration;
use zeroize::Zeroizing;

pub enum CoreResponse {
    Json(Value),
    Secret(Zeroizing<String>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionBridgeError {
    Backend(&'static str),
    Denied,
    Failed,
    MethodUnavailable,
    Context(&'static str),
    Revoked,
    Timeout,
}

impl ExtensionBridgeError {
    pub(super) fn reason(self) -> &'static str {
        match self {
            Self::Backend(reason) => reason,
            Self::Denied => "core_permission_denied",
            Self::Failed => "core_request_failed",
            Self::MethodUnavailable => "core_method_unavailable",
            Self::Context(reason) => reason,
            Self::Revoked => "core_context_revoked",
            Self::Timeout => "core_request_timeout",
        }
    }
}

pub async fn call(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: Option<&Value>,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let outcome = execute(context, method, params).await;
    finalize_response(
        outcome,
        || super::registry_access::mark_sensitive_access(context.identity()),
        |result| super::access_log::write_core(context, method, result),
    )
}

fn finalize_response(
    mut outcome: Result<CoreResponse, ExtensionBridgeError>,
    mark_sensitive: impl FnOnce() -> Result<(), String>,
    write_audit: impl FnOnce(super::access_log::AccessResult) -> Result<(), String>,
) -> Result<CoreResponse, ExtensionBridgeError> {
    if matches!(outcome, Ok(CoreResponse::Secret(_))) && mark_sensitive().is_err() {
        outcome = Err(ExtensionBridgeError::Failed);
    }
    super::core_response_audit::record_outcome(outcome, write_audit)
}

async fn execute(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: Option<&Value>,
) -> Result<CoreResponse, ExtensionBridgeError> {
    if !super::registry_access::authorize_call(context).unwrap_or(false) {
        return Err(ExtensionBridgeError::Denied);
    }
    let params = params.unwrap_or(&Value::Null);
    let policy = super::core_api_dispatch::policy(context, method)?;
    validate_request_params(params)?;
    if context.revoked().is_cancelled() {
        return Err(ExtensionBridgeError::Revoked);
    }
    super::core_api_permissions::authorize(context, method, policy.effect).await?;
    let budget = context.core_scope().map_or(policy.budget, |scope| {
        policy.budget.min(
            scope
                .deadline
                .saturating_duration_since(std::time::Instant::now()),
        )
    });
    if budget.is_zero() {
        return Err(ExtensionBridgeError::Timeout);
    }
    await_unrevoked(context, budget, dispatch(context, method, params)).await
}

async fn await_unrevoked<F>(
    context: &super::call_context::ExtensionCallContext,
    budget: Duration,
    operation: F,
) -> Result<CoreResponse, ExtensionBridgeError>
where
    F: std::future::Future<Output = Result<CoreResponse, ExtensionBridgeError>>,
{
    tokio::select! {
        biased;
        _ = context.revoked().cancelled() => Err(ExtensionBridgeError::Revoked),
        result = tokio::time::timeout(budget, operation) => match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(ExtensionBridgeError::Timeout),
        },
    }
}

async fn dispatch(
    context: &super::call_context::ExtensionCallContext,
    method: &str,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    if method.starts_with("models.") {
        return super::core_models::call(context, method, params).await;
    }
    dispatch_legacy(method, params)
        .await
        .map_err(|()| ExtensionBridgeError::Failed)
}

async fn dispatch_legacy(method: &str, params: &Value) -> Result<CoreResponse, ()> {
    match method {
        "app.info" => Ok(CoreResponse::Json(json!({
            "apiVersion": super::types::BEAVER_API_VERSION,
            "dataDir": crate::services::paths::data_dir().to_string_lossy(),
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
        }))),
        "sessions.list" => {
            let sessions = crate::services::agent_local::session_store::list()
                .await
                .map_err(|_| ())?
                .into_iter()
                .take(MAX_SESSION_RESULTS)
                .collect::<Vec<_>>();
            json_response(sessions)
        }
        "sessions.get" => {
            let id = string_param(params, "sessionId")?;
            let session = crate::services::agent_local::session_store::get(id)
                .await
                .map_err(|_| ())?;
            json_response(session)
        }
        "projects.list" => {
            let projects = crate::services::agent_local::project_store::list()
                .await
                .map_err(|_| ())?
                .into_iter()
                .take(MAX_PROJECT_RESULTS)
                .collect::<Vec<_>>();
            json_response(projects)
        }
        "mcp.connectors.list" => {
            let connectors = crate::services::mcp_bridge::config::load().map_err(|_| ())?;
            json_response(connectors)
        }
        "mcp.tool.call" => call_mcp_tool(params).await,
        "channels.config.get" => {
            let config = crate::services::config::read_config().map_err(|_| ())?;
            json_response(config.gateway)
        }
        "secrets.provider.get" => super::core_secrets::provider(params),
        "secrets.mcp.oauth.get" => super::core_secrets::mcp_oauth(params).await,
        "secrets.mcp.env.get" => super::core_secrets::mcp_env(params),
        "secrets.channel.get" => super::core_secrets::channel(params),
        _ => Err(()),
    }
}

fn validate_request_params(params: &Value) -> Result<(), ExtensionBridgeError> {
    if params.get("extensionId").is_some() || super::validation::message(params).is_err() {
        return Err(ExtensionBridgeError::Denied);
    }
    Ok(())
}

async fn call_mcp_tool(params: &Value) -> Result<CoreResponse, ()> {
    let connector_id = string_param(params, "connectorId")?;
    let tool_name = string_param(params, "toolName")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    super::validation::message(&arguments).map_err(|_| ())?;
    let (connector, tool) =
        crate::services::mcp_bridge::registry::resolve_enabled_tool(connector_id, tool_name)
            .await
            .map_err(|_| ())?;
    crate::services::mcp_bridge::arguments::validate(&arguments, tool.input_schema.as_ref())
        .map_err(|_| ())?;
    let result = connector
        .transport
        .call_tool(&tool.name, arguments)
        .await
        .map_err(|_| ())?;
    if result.is_error {
        return Err(());
    }
    Ok(CoreResponse::Json(Value::String(result.content)))
}

pub(super) fn string_param<'a>(params: &'a Value, key: &str) -> Result<&'a str, ()> {
    let value = params.get(key).and_then(Value::as_str).ok_or(())?;
    super::validation::source_input(value).map_err(|_| ())?;
    Ok(value)
}

fn json_response(value: impl Serialize) -> Result<CoreResponse, ()> {
    serde_json::to_value(value)
        .map(CoreResponse::Json)
        .map_err(|_| ())
}

#[cfg(test)]
#[path = "core_bridge_tests.rs"]
mod tests;
