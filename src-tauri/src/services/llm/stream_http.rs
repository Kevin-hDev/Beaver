use super::provider_error::ProviderErrorCode;
pub(super) use super::stream_http_error::RequestError;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::route;
use crate::services::secure_http::{read_bounded, AuthenticatedClient, PROVIDER_ERROR_LIMIT};
pub struct RequestConfig<'a> {
    pub provider_id: &'a str,
    pub model: &'a str,
    pub messages: &'a [ChatMessage],
    pub tools: &'a [serde_json::Value],
    pub think: bool,
    pub reasoning_mode: Option<&'a str>,
    pub max_tokens: Option<u32>,
    pub purpose: RequestPurpose,
    pub session_id: Option<&'a str>,
    pub fast_mode: super::fast_mode::FastModeRequest,
}

use super::stream_http_payload::build_chat_payload;

async fn read_provider_error(response: reqwest::Response) -> zeroize::Zeroizing<String> {
    match read_bounded(response, PROVIDER_ERROR_LIMIT).await {
        Ok(bytes) => zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => zeroize::Zeroizing::new(String::new()),
    }
}

pub(super) async fn post_chat_request_measured(
    cfg: &RequestConfig<'_>,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout_measured(
        cfg,
        super::timeouts::request_timeout_for(cfg.provider_id),
        measurement,
    )
    .await
}

#[cfg(test)]
pub async fn post_chat_request_with_timeout(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout_measured(cfg, timeout, None).await
}

pub(super) async fn post_chat_request_with_timeout_measured(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<reqwest::Response, RequestError> {
    if cfg.model.len() > 128 {
        return Err(RequestError::Fatal("nom de modèle trop long".into()));
    }
    let route = route::resolve(cfg.provider_id)
        .ok_or_else(|| RequestError::Fatal("Fournisseur inconnu".to_string()))?;
    let url = format!("{}/chat/completions", route.base_url);
    let estimated_input_tokens =
        crate::services::compress::token_estimate::estimate_request_tokens(cfg.messages, cfg.tools);
    let max_tokens = super::stream_max_tokens::resolve(
        route.canonical_provider_id,
        cfg.model,
        cfg.max_tokens,
        route.auto_max_tokens,
        route.fallback_max_tokens,
        estimated_input_tokens,
    )
    .await
    .map_err(request_error_for_limit)?;
    let payload = build_chat_payload(cfg, &route, max_tokens);
    #[cfg(test)]
    if let Some(response) = super::stream_test_transport::dispatch(cfg, &payload).await {
        return response;
    }
    let request_bytes = serde_json::to_vec(&payload)
        .map(zeroize::Zeroizing::new)
        .map_or(0, |bytes| bytes.len());

    let client = AuthenticatedClient::new(timeout).map_err(|_| {
        RequestError::Fatal(
            ProviderErrorCode::ProviderConfigurationInvalid
                .as_str()
                .into(),
        )
    })?;
    let usage_generation =
        crate::services::provider_usage::credential_generation(route.chat_provider_id);
    let resp = super::stream_http_send::send_json_request(
        &client,
        &route,
        &url,
        &payload,
        cfg.purpose,
        cfg.model,
        cfg.session_id,
    )
    .await?;
    if let Some(measurement) = measurement.as_mut() {
        measurement.mark_headers();
    }

    crate::services::provider_usage::capture_headers(
        route.chat_provider_id,
        usage_generation,
        resp.headers(),
    )
    .await;

    let status = resp.status();
    if !status.is_success() {
        let has_retry_after = resp.headers().contains_key("retry-after");
        let body = read_provider_error(resp).await;
        let log_code =
            super::provider_error::safe_log_code(route.chat_provider_id, status.as_u16(), &body);
        super::provider_diagnostics::record_http_failure(
            route.chat_provider_id,
            cfg.model,
            status.as_u16(),
            super::provider_error::safe_details(&body),
            request_bytes,
            cfg.tools.len(),
        );
        ::log::warn!("[llm stream] HTTP {status} code={log_code}");
        return Err(classify_error(
            status.as_u16(),
            &body,
            route.display_name,
            route.chat_provider_id,
            route.is_oauth(),
            has_retry_after,
        ));
    }
    Ok(resp)
}

fn request_error_for_limit(error: super::stream_max_tokens::ResolveError) -> RequestError {
    match error {
        super::stream_max_tokens::ResolveError::ContextExhausted => RequestError::PayloadTooLarge,
        super::stream_max_tokens::ResolveError::InvalidLimit => RequestError::InvalidConfiguration,
    }
}

fn classify_error(
    status: u16,
    body: &str,
    _provider_name: &str,
    provider_id: &str,
    oauth: bool,
    has_retry_after: bool,
) -> RequestError {
    match status {
        402 => RequestError::Fatal(
            super::provider_error::classify_http(provider_id, status, body)
                .as_str()
                .to_string(),
        ),
        401 if oauth => RequestError::Fatal("oauth_reauthentication_required".into()),
        403 if oauth => RequestError::Fatal("provider_access_unavailable".into()),
        401 | 403 => RequestError::Fatal("auth_failed".into()),
        413 => RequestError::PayloadTooLarge,
        429 if provider_id == "xai-oauth"
            && !has_retry_after
            && super::provider_error::safe_details(body)
                .error_code
                .as_deref()
                == Some("resource-exhausted") =>
        {
            RequestError::Fatal("provider_quota_exhausted".into())
        }
        429 => RequestError::Fatal("rate_limit".into()),
        500..=599 => RequestError::Fatal(
            ProviderErrorCode::ProviderTemporarilyUnavailable
                .as_str()
                .to_string(),
        ),
        _ => RequestError::Fatal(
            ProviderErrorCode::ProviderRequestRejected
                .as_str()
                .to_string(),
        ),
    }
}

#[cfg(test)]
#[path = "stream_http_classification_tests.rs"]
mod classification_tests;
#[cfg(test)]
#[path = "stream_http_tests.rs"]
mod tests;
