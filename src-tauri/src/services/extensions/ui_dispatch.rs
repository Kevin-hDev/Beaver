use super::ui_types::UiActionRequest;
use serde_json::{json, Value};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

pub(super) async fn invoke(request: UiActionRequest) -> Result<Value, String> {
    validate_request(&request)?;
    let runtime = Arc::clone(
        super::runtime::global()
            .map_err(|_| failed(&request.extension_id, UiActionFailure::Unavailable))?,
    );
    let route = runtime
        .ui_catalog
        .route(
            &request.extension_id,
            &request.contribution_id,
            &request.action_id,
        )
        .map_err(|_| failed(&request.extension_id, UiActionFailure::Denied))?;
    let (identity, generation, process) = runtime
        .process_for_extension(
            &request.extension_id,
            super::runtime_lifecycle::new_stop_deadline(),
        )
        .await
        .map_err(|_| failed(&request.extension_id, UiActionFailure::Unavailable))?;
    if identity != route.identity || generation != route.generation {
        return Err(failed(&request.extension_id, UiActionFailure::Stale));
    }
    let context = runtime
        .call_context(&identity, generation)
        .await
        .map_err(|_| failed(&request.extension_id, UiActionFailure::Stale))?;
    let params = json!({
        "extensionId": request.extension_id,
        "contributionId": request.contribution_id,
        "actionId": request.action_id,
        "payload": request.payload,
        "context": {"locale": request.locale},
    });
    let response = await_action(
        process.request("ui.action", params),
        context.revoked().clone(),
        Duration::from_millis(super::types::UI_ACTION_TIMEOUT_MS as u64),
    )
    .await
    .map_err(|failure| failed(&request.extension_id, failure))?;
    let validated = super::ui_action_result::validate(&request.extension_id, response)
        .map_err(|_| failed(&request.extension_id, UiActionFailure::ResultInvalid))?;
    runtime
        .ui_catalog
        .refresh_actions(
            &request.extension_id,
            &request.contribution_id,
            generation,
            route.catalog_revision,
            validated.action_ids,
        )
        .map_err(|_| failed(&request.extension_id, UiActionFailure::Stale))?;
    Ok(validated.value)
}

pub(super) async fn await_action<F>(
    future: F,
    cancellation: CancellationToken,
    timeout: Duration,
) -> Result<Value, UiActionFailure>
where
    F: Future<Output = Result<Value, String>>,
{
    tokio::select! {
        _ = cancellation.cancelled() => Err(UiActionFailure::Cancelled),
        result = tokio::time::timeout(timeout, future) => match result {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(_)) => Err(UiActionFailure::RequestFailed),
            Err(_) => Err(UiActionFailure::Timeout),
        },
    }
}

fn validate_request(request: &UiActionRequest) -> Result<(), String> {
    for identifier in [
        request.extension_id.as_str(),
        request.contribution_id.as_str(),
        request.action_id.as_str(),
    ] {
        super::validation::identifier(identifier)
            .map_err(|_| failed(&request.extension_id, UiActionFailure::Invalid))?;
    }
    if !super::ui_contract::UI_LOCALES.contains(&request.locale.as_str()) {
        return Err(failed(&request.extension_id, UiActionFailure::Invalid));
    }
    request
        .payload
        .validate()
        .map_err(|_| failed(&request.extension_id, UiActionFailure::Invalid))
}

fn failed(extension_id: &str, failure: UiActionFailure) -> String {
    super::operation_log::write_ui_action(extension_id, failure.reason());
    super::error_codes::OPERATION_FAILED.to_string()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiActionFailure {
    Invalid,
    Denied,
    Stale,
    Unavailable,
    Cancelled,
    Timeout,
    RequestFailed,
    ResultInvalid,
}

impl UiActionFailure {
    fn reason(self) -> &'static str {
        match self {
            Self::Invalid => "ui_action_invalid",
            Self::Denied => "ui_action_denied",
            Self::Stale => "ui_action_stale",
            Self::Unavailable => "ui_action_unavailable",
            Self::Cancelled => "ui_action_cancelled",
            Self::Timeout => "ui_action_timeout",
            Self::RequestFailed => "ui_action_request_failed",
            Self::ResultInvalid => "ui_action_result_invalid",
        }
    }
}

pub(super) fn is_safe_failure_reason(reason: &str) -> bool {
    [
        UiActionFailure::Invalid,
        UiActionFailure::Denied,
        UiActionFailure::Stale,
        UiActionFailure::Unavailable,
        UiActionFailure::Cancelled,
        UiActionFailure::Timeout,
        UiActionFailure::RequestFailed,
        UiActionFailure::ResultInvalid,
    ]
    .into_iter()
    .any(|failure| failure.reason() == reason)
}
