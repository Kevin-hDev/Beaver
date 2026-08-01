use crate::services::agent_local::types_ollama::{StreamOutcome, StreamResult};
use crate::services::provider_usage::{
    RequestMeasurement, RequestMeasurementContext, RequestMetricStatus, UsageApiFormat,
    UsageWorkload,
};

pub(super) fn start(
    connection_id: &str,
    model: &str,
    session_id: Option<&str>,
    request_id: &str,
    turn: Option<u32>,
    attempt: u32,
    workload: UsageWorkload,
) -> Option<RequestMeasurement> {
    let (canonical_provider_id, api_format) = if connection_id == "codex-oauth" {
        ("openai", UsageApiFormat::Responses)
    } else {
        (
            super::route::canonical_provider_id(connection_id),
            UsageApiFormat::ChatCompletions,
        )
    };
    RequestMeasurement::start(RequestMeasurementContext {
        connection_id,
        canonical_provider_id,
        api_format,
        model,
        session_id,
        request_id,
        turn,
        attempt,
        workload,
    })
}

pub(super) async fn finish_stream(
    measurement: Option<RequestMeasurement>,
    result: &Result<StreamOutcome, String>,
) {
    let Some(measurement) = measurement else {
        return;
    };
    let (status, usage, complete) = match result {
        Ok(StreamOutcome::Completed(stream)) => (
            RequestMetricStatus::Completed,
            stream.usage.as_ref(),
            stream.usage.is_some(),
        ),
        Ok(StreamOutcome::InterruptedForCompression(stream)) => (
            RequestMetricStatus::Interrupted,
            stream.usage.as_ref(),
            false,
        ),
        Err(error) if error == "Annulé" => (RequestMetricStatus::Cancelled, None, false),
        Err(_) => (RequestMetricStatus::Failed, None, false),
    };
    measurement.finish(status, usage, complete).await;
}

pub(super) async fn finish_silent(
    measurement: Option<RequestMeasurement>,
    result: &Result<StreamResult, String>,
) {
    let Some(measurement) = measurement else {
        return;
    };
    let (status, usage, complete) = match result {
        Ok(stream) => (
            RequestMetricStatus::Completed,
            stream.usage.as_ref(),
            stream.usage.is_some(),
        ),
        Err(error) if error == "Annulé" => (RequestMetricStatus::Cancelled, None, false),
        Err(_) => (RequestMetricStatus::Failed, None, false),
    };
    measurement.finish(status, usage, complete).await;
}
