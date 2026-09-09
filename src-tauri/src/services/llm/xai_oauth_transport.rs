use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamOutcome;
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm_oauth::{XaiBackend, XaiCatalogModel};
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
use tokio_util::sync::CancellationToken;

pub(super) use super::xai_oauth_chat::prepare as prepare_chat_request;
pub(super) use super::xai_oauth_transport_status::classify_status;

pub(super) struct StreamContext<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub request: super::stream_http::RequestConfig<'a>,
    pub cancel: CancellationToken,
    pub buffer_content: bool,
    pub realtime_budget: Option<RealtimeBudget>,
    pub reasoning_capture: Option<super::reasoning_wire::ReasoningCapture>,
    pub request_id: &'a str,
}

pub(super) async fn stream_chat(
    context: StreamContext<'_>,
    catalog_model: &XaiCatalogModel,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let StreamContext {
        on_event,
        request,
        cancel,
        buffer_content,
        realtime_budget,
        reasoning_capture,
        request_id,
    } = context;
    validate_backend(catalog_model.backend, &request)?;
    match catalog_model.backend {
        XaiBackend::ChatCompletions => {
            let request = prepare_chat_request(request, catalog_model);
            let response =
                super::xai_oauth_chat::post(&request, measurement.as_deref_mut(), Some(request_id))
                    .await;
            let response = response.map_err(|error| error.to_string())?;
            super::stream_consume::consume_stream(
                on_event,
                response,
                cancel,
                buffer_content,
                realtime_budget,
                request.tools,
                crate::services::provider_usage::UsageContext::chat("xai", request.model),
                super::route_profile::FragmentMode::DifferentialFragments,
                super::route_profile::ErrorPolicy::XaiOauth,
                reasoning_capture,
                measurement,
            )
            .await
        }
        XaiBackend::Responses => {
            let prepared = prepare_responses_request(catalog_model, &request)?;
            crate::services::llm::reasoning_wire::replay::record_evidence(
                request.session_id,
                Some(request_id),
                &prepared.replayed,
            )
            .await;
            let response = post_responses(
                catalog_model,
                &prepared.payload,
                request.purpose,
                Some(request_id),
                request.tools.len(),
            )
            .await?;
            crate::services::codex_client::stream::consume_external_responses_sse(
                on_event,
                response,
                cancel,
                buffer_content,
                realtime_budget,
                "xai",
                request.model,
                request.tools,
                reasoning_capture,
                measurement,
            )
            .await
        }
    }
}

pub(super) fn prepare_responses_request(
    catalog_model: &XaiCatalogModel,
    request: &super::stream_http::RequestConfig<'_>,
) -> Result<super::xai_oauth_payload::PreparedResponsesPayload, String> {
    // xAI OAuth is explicitly text-only: preview bytes never cross this
    // builder boundary until its own wire contract is proven.
    let prepared = super::xai_oauth_payload::build_with_evidence(
        catalog_model,
        request.messages,
        request.tools,
        request.reasoning_mode,
        request.session_id,
        request.continuation_target,
    )?;
    #[cfg(debug_assertions)]
    let prepared = {
        let mut prepared = prepared;
        if crate::services::reasoning_fixture_budget::is_active() {
            let output_limit =
                crate::services::reasoning_fixture_budget::output_limit(request.max_tokens)
                    .ok_or_else(|| "fixture limits invalid".to_string())?;
            prepared.payload["max_output_tokens"] = output_limit.into();
        }
        prepared
    };
    Ok(prepared)
}

pub(super) fn validate_backend(
    backend: XaiBackend,
    request: &super::stream_http::RequestConfig<'_>,
) -> Result<(), String> {
    if backend != XaiBackend::Responses
        && super::xai_oauth_transport_status::requires_responses_backend(request)
    {
        return Err("reasoning_continuity_invalid".to_string());
    }
    Ok(())
}

pub(super) fn catalog_reasoning_mode<'a>(
    model: &'a XaiCatalogModel,
    requested_mode: Option<&'a str>,
) -> Option<&'a str> {
    requested_mode
        .filter(|mode| {
            model
                .reasoning_modes
                .iter()
                .any(|candidate| candidate == mode)
        })
        .or_else(|| {
            model.default_reasoning_mode.as_deref().filter(|mode| {
                model
                    .reasoning_modes
                    .iter()
                    .any(|candidate| candidate == mode)
            })
        })
}

async fn post_responses(
    model: &XaiCatalogModel,
    payload: &serde_json::Value,
    purpose: super::request_purpose::RequestPurpose,
    request_id: Option<&str>,
    tool_count: usize,
) -> Result<reqwest::Response, String> {
    let route = super::route::resolve("xai-oauth")
        .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    let client = AuthenticatedClient::new_streaming(
        super::timeouts::connect_timeout(),
        super::timeouts::idle_timeout_for("xai-oauth"),
    )
    .map_err(|_| "provider_configuration_invalid".to_string())?;
    let headers = crate::services::llm_oauth::xai_model_header(&model.id)
        .map_err(|_| "provider_configuration_invalid".to_string())?;
    let url = format!("{}{}", route.base_url, backend_path(model.backend));
    let response = route
        .send_generation_authenticated(&client, purpose, payload, |token, auth_headers| {
            let mut combined = auth_headers;
            combined.extend(headers.clone());
            client
                .post(&url)
                .headers(combined)
                .bearer_auth(token)
                .header("Accept", "text/event-stream")
                .json(payload)
        })
        .await
        .map_err(|error| match error {
            super::route::RouteError::Unauthorized => "oauth_reauthentication_required".to_owned(),
            super::route::RouteError::Forbidden => "provider_access_unavailable".to_owned(),
            super::route::RouteError::Network => "provider_connection_failed".to_owned(),
            #[cfg(debug_assertions)]
            super::route::RouteError::FixtureBudget(message) => message,
        })?;
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status().as_u16();
    let has_retry_after = response.headers().contains_key("retry-after");
    let diagnostic_context =
        super::provider_diagnostics::ProviderDiagnosticContext::from_payload(request_id, payload)
            .with_retry_after(response.headers());
    let request_bytes = serde_json::to_vec(payload)
        .map(zeroize::Zeroizing::new)
        .map_or(0, |bytes| bytes.len());
    let body = read_bounded(response, PROVIDER_ERROR_LIMIT)
        .await
        .map(|bytes| zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()))
        .unwrap_or_default();
    super::provider_diagnostics::record_http_failure(
        "xai-oauth",
        &model.id,
        status,
        super::provider_error::safe_details(&body),
        request_bytes,
        tool_count,
        diagnostic_context,
    );
    Err(classify_status(route.error_policy, status, &body, has_retry_after).to_string())
}

pub(super) const fn backend_path(backend: XaiBackend) -> &'static str {
    match backend {
        XaiBackend::ChatCompletions => "/chat/completions",
        XaiBackend::Responses => "/responses",
    }
}
