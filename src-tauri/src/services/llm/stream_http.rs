use super::provider_error::ProviderErrorCode;
pub(super) use super::stream_http_error::{classify_error, RequestError};
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
    /// Aperçus vérifiés du dernier lot d'outils, gardés seulement en mémoire.
    pub tool_result_previews:
        Option<&'a crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch>,
    /// Décision d'admission conservée jusqu'au constructeur du payload.
    pub continuation_target:
        Option<&'a crate::services::reasoning_continuity::contract::ContinuationTarget>,
}

#[cfg(test)]
use super::stream_http_payload::build_chat_payload;
use super::stream_http_payload::build_chat_payload_with_evidence;

async fn read_provider_error(response: reqwest::Response) -> zeroize::Zeroizing<String> {
    match read_bounded(response, PROVIDER_ERROR_LIMIT).await {
        Ok(bytes) => zeroize::Zeroizing::new(String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => zeroize::Zeroizing::new(String::new()),
    }
}

pub(super) async fn post_chat_request_measured(
    cfg: &RequestConfig<'_>,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    request_id: Option<&str>,
) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout_and_policy(
        cfg,
        super::timeouts::request_timeout_for(cfg.provider_id),
        measurement,
        request_id,
        None,
    )
    .await
}

pub(super) async fn post_chat_request_with_payload_policy(
    cfg: &RequestConfig<'_>,
    payload_policy: super::route_profile::ResolvedPayloadPolicy,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    request_id: Option<&str>,
) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout_and_policy(
        cfg,
        super::timeouts::request_timeout_for(cfg.provider_id),
        measurement,
        request_id,
        Some(payload_policy),
    )
    .await
}

#[cfg(test)]
pub async fn post_chat_request_with_timeout(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout_measured(cfg, timeout, None, None).await
}

pub(super) async fn post_chat_request_with_timeout_measured(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    request_id: Option<&str>,
) -> Result<reqwest::Response, RequestError> {
    post_chat_request_with_timeout_and_policy(cfg, timeout, measurement, request_id, None).await
}

async fn post_chat_request_with_timeout_and_policy(
    cfg: &RequestConfig<'_>,
    timeout: std::time::Duration,
    mut measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
    request_id: Option<&str>,
    payload_policy: Option<super::route_profile::ResolvedPayloadPolicy>,
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
    let prepared = match payload_policy {
        Some(policy) => super::stream_http_payload::build_chat_payload_with_policy(
            cfg, &route, max_tokens, policy,
        ),
        None => build_chat_payload_with_evidence(cfg, &route, max_tokens),
    }
    .map_err(|_| RequestError::Fatal("reasoning_continuity_invalid".to_string()))?;
    let payload = prepared.payload;
    super::reasoning_wire::replay::record_evidence(cfg.session_id, request_id, &prepared.replayed)
        .await;
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
            super::provider_error::safe_log_code(route.error_policy, status.as_u16(), &body);
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
            route.error_policy,
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

#[cfg(test)]
#[path = "stream_http_classification_tests.rs"]
mod classification_tests;
#[cfg(test)]
#[path = "stream_http_tests.rs"]
mod tests;
