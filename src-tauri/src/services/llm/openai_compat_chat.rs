use super::openai_compat::OpenAiCompatProvider;
use super::openai_compat_parsing::{build_payload, map_error_status, parse_chat_response};
use super::request_purpose::RequestPurpose;
use super::types::{ChatRequest, ChatResponse, LlmError};

pub(super) async fn chat_completion(
    provider: &OpenAiCompatProvider,
    request: ChatRequest,
    purpose: RequestPurpose,
    session_id: Option<&str>,
) -> Result<ChatResponse, LlmError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut measurement = (purpose == RequestPurpose::Automation)
        .then(|| {
            super::stream_metrics::start(
                provider.route.chat_provider_id,
                &request.model,
                session_id,
                &request_id,
                None,
                1,
                crate::services::provider_usage::UsageWorkload::Primary,
            )
        })
        .flatten();
    let url = format!("{}/chat/completions", provider.route.base_url);
    let mut payload = build_payload(&request, provider.route.canonical_provider_id, false);
    super::prompt_cache_policy::apply_payload(
        &mut payload,
        &provider.route,
        &request.model,
        session_id,
    );
    let policy_headers = super::prompt_cache_policy::request_headers(
        &provider.route,
        Some(&request.model),
        session_id,
        purpose,
    )
    .map_err(|_| {
        LlmError::KnownProvider(
            super::provider_error::ProviderErrorCode::ProviderConfigurationInvalid,
        )
    })?;
    let usage_generation =
        crate::services::provider_usage::credential_generation(provider.route.chat_provider_id);
    let response = match provider
        .send(purpose, |token, headers| {
            let mut headers = headers;
            headers.extend(policy_headers.clone());
            provider
                .client
                .post(&url)
                .headers(headers)
                .bearer_auth(token)
                .json(&payload)
        })
        .await
    {
        Ok(response) => response,
        Err(error) => return failed(measurement, error).await,
    };
    if let Some(measurement) = measurement.as_mut() {
        measurement.mark_headers();
    }
    crate::services::provider_usage::capture_headers(
        provider.route.chat_provider_id,
        usage_generation,
        response.headers(),
    )
    .await;
    if !response.status().is_success() {
        let error = map_error_status(response, provider.route.chat_provider_id).await;
        return failed(measurement, error).await;
    }
    let body = match super::openai_compat::read_json(response).await {
        Ok(body) => body,
        Err(error) => return failed(measurement, error).await,
    };
    if let Some(measurement) = measurement.as_mut() {
        measurement.observe_response_metadata(&body);
    }
    let result = parse_chat_response(&body, provider.route.canonical_provider_id, &request.model);
    match result {
        Ok(result) => completed(measurement, result).await,
        Err(error) => failed(measurement, error).await,
    }
}

async fn completed(
    mut measurement: Option<crate::services::provider_usage::RequestMeasurement>,
    result: ChatResponse,
) -> Result<ChatResponse, LlmError> {
    if !result.content.is_empty() {
        if let Some(measurement) = measurement.as_mut() {
            measurement.mark_first_useful();
        }
    }
    if let Some(measurement) = measurement {
        let complete = !result.usage.is_empty();
        measurement
            .finish(
                crate::services::provider_usage::RequestMetricStatus::Completed,
                Some(&result.usage),
                complete,
            )
            .await;
    }
    Ok(result)
}

async fn failed(
    measurement: Option<crate::services::provider_usage::RequestMeasurement>,
    error: LlmError,
) -> Result<ChatResponse, LlmError> {
    if let Some(measurement) = measurement {
        measurement
            .finish(
                crate::services::provider_usage::RequestMetricStatus::Failed,
                None,
                false,
            )
            .await;
    }
    Err(error)
}
