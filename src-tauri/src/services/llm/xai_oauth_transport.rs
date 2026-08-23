use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome};
use crate::services::compress::realtime_budget::RealtimeBudget;
use crate::services::llm_oauth::{XaiBackend, XaiCatalogModel};
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
use tokio_util::sync::CancellationToken;

pub(super) struct StreamContext<'a> {
    pub on_event: &'a AgentEventEmitter,
    pub request: super::stream_http::RequestConfig<'a>,
    pub cancel: CancellationToken,
    pub buffer_content: bool,
    pub realtime_budget: Option<RealtimeBudget>,
}

pub(super) async fn stream_chat(
    context: StreamContext<'_>,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamOutcome, String> {
    let StreamContext {
        on_event,
        request,
        cancel,
        buffer_content,
        realtime_budget,
    } = context;
    let catalog_model = crate::services::llm_oauth::xai_catalog_model(request.model).await?;
    match catalog_model.backend {
        XaiBackend::ChatCompletions => {
            let request = prepare_chat_request(request, &catalog_model);
            let response = post_catalog_chat_request(&request, measurement.as_deref_mut()).await;
            let response = response.map_err(|error| error.to_string())?;
            super::stream_consume::consume_stream(
                on_event,
                response,
                cancel,
                buffer_content,
                realtime_budget,
                request.tools,
                crate::services::provider_usage::UsageContext::chat("xai", request.model),
                measurement,
            )
            .await
        }
        XaiBackend::Responses => {
            let payload = build_responses_payload(
                &catalog_model,
                request.messages,
                request.tools,
                request.reasoning_mode,
                request.session_id,
            );
            let response = post_responses(&catalog_model, &payload, request.purpose).await?;
            crate::services::codex_client::stream::consume_external_responses_sse(
                on_event,
                response,
                cancel,
                buffer_content,
                realtime_budget,
                "xai",
                request.model,
                request.tools,
                measurement,
            )
            .await
        }
    }
}

pub(super) struct CatalogChatRequest<'a>(super::stream_http::RequestConfig<'a>);

impl<'a> std::ops::Deref for CatalogChatRequest<'a> {
    type Target = super::stream_http::RequestConfig<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

async fn post_catalog_chat_request(
    request: &CatalogChatRequest<'_>,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<reqwest::Response, super::stream_http::RequestError> {
    // Ce type fermé empêche le chemin OAuth d'envoyer une requête chat non restreinte.
    super::stream_http::post_chat_request_measured(request, measurement).await
}

pub(super) fn prepare_chat_request<'request, 'catalog>(
    request: super::stream_http::RequestConfig<'request>,
    model: &'catalog XaiCatalogModel,
) -> CatalogChatRequest<'catalog>
where
    'request: 'catalog,
{
    CatalogChatRequest(super::stream_http::RequestConfig {
        provider_id: request.provider_id,
        model: request.model,
        messages: request.messages,
        tools: request.tools,
        think: request.think,
        reasoning_mode: catalog_reasoning_mode(model, request.reasoning_mode),
        max_tokens: request.max_tokens,
        purpose: request.purpose,
        session_id: request.session_id,
        fast_mode: request.fast_mode,
    })
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

pub(super) fn build_responses_payload(
    model: &XaiCatalogModel,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    requested_mode: Option<&str>,
    session_id: Option<&str>,
) -> serde_json::Value {
    let (instructions, input) =
        crate::services::codex_client::convert::convert_messages_with_tools(messages, tools);
    let effort = catalog_reasoning_mode(model, requested_mode);
    let mut payload = serde_json::json!({
        "model": model.id,
        "instructions": instructions,
        "input": input,
        "stream": true,
        "store": false,
        "tools": crate::services::codex_client::convert::convert_tools_to_responses_api(tools),
        "tool_choice": "auto",
        "parallel_tool_calls": false,
        "prompt_cache_key": super::prompt_cache_policy::routing_key(
            "xai-oauth", &model.id, session_id,
        ),
        "include": ["reasoning.encrypted_content"],
    });
    if let Some(effort) = effort {
        payload["reasoning"] = serde_json::json!({"effort": effort});
    }
    payload
}

async fn post_responses(
    model: &XaiCatalogModel,
    payload: &serde_json::Value,
    purpose: super::request_purpose::RequestPurpose,
) -> Result<reqwest::Response, String> {
    let route = super::route::resolve("xai-oauth")
        .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    let client = AuthenticatedClient::new_streaming(
        std::time::Duration::from_secs(10),
        super::timeouts::idle_timeout_for("xai-oauth"),
    )
    .map_err(|_| "provider_configuration_invalid".to_string())?;
    let headers = crate::services::llm_oauth::xai_model_header(&model.id)
        .map_err(|_| "provider_configuration_invalid".to_string())?;
    let url = format!("{}{}", route.base_url, backend_path(model.backend));
    let response = route
        .send_authenticated(&client, purpose, |token, auth_headers| {
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
            super::route::RouteError::Unauthorized => "oauth_reauthentication_required",
            super::route::RouteError::Forbidden => "provider_access_unavailable",
            super::route::RouteError::Network => "provider_connection_failed",
        })?;
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status().as_u16();
    let has_retry_after = response.headers().contains_key("retry-after");
    let body = read_bounded(response, PROVIDER_ERROR_LIMIT)
        .await
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    Err(classify_status(status, &body, has_retry_after).to_string())
}

pub(super) const fn backend_path(backend: XaiBackend) -> &'static str {
    match backend {
        XaiBackend::ChatCompletions => "/chat/completions",
        XaiBackend::Responses => "/responses",
    }
}

pub(super) fn classify_status(status: u16, body: &str, has_retry_after: bool) -> &'static str {
    match status {
        401 => "oauth_reauthentication_required",
        403 => "provider_access_unavailable",
        429 if !has_retry_after
            && crate::services::llm::provider_error::safe_details(body)
                .error_code
                .as_deref()
                == Some("resource-exhausted") =>
        {
            "provider_quota_exhausted"
        }
        429 => "rate_limit",
        500..=599 => "provider_temporarily_unavailable",
        _ => "provider_request_rejected",
    }
}
