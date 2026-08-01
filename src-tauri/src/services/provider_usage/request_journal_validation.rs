use super::request_journal::{ProviderRequestMetric, RequestTiming};
use super::RequestUsage;

const MAX_ATTEMPT: u32 = 1_000;
const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1_000;
const MIN_STARTED_AT_MS: i64 = 946_684_800_000;
const MAX_CLOCK_SKEW_MS: i64 = 5 * 60 * 1_000;

impl ProviderRequestMetric {
    pub(super) fn is_valid(&self) -> bool {
        super::types::validate_connection_id(&self.connection_id).is_ok()
            && valid_provider(&self.canonical_provider_id)
            && valid_route(self)
            && valid_label(&self.model, 128)
            && valid_routed_endpoint(self)
            && valid_label(&self.request_id, 128)
            && self
                .session_id
                .as_deref()
                .is_none_or(|value| valid_label(value, 128))
            && matches!(
                self.workload.as_str(),
                "primary" | "subagent" | "compression"
            )
            && matches!(
                self.origin.as_str(),
                "manual_chat" | "external_channel" | "automation"
            )
            && (1..=MAX_ATTEMPT).contains(&self.attempt)
            && self.turn.is_none_or(|turn| turn <= 1_000_000)
            && valid_started_at(self.started_at_ms)
            && valid_timing(&self.timing)
            && (!self.usage_complete
                || (self.status == super::request_journal::RequestMetricStatus::Completed
                    && self.usage.is_some()))
            && self
                .usage
                .as_ref()
                .is_none_or(RequestUsage::is_valid_observation)
    }
}

fn valid_routed_endpoint(metric: &ProviderRequestMetric) -> bool {
    match (
        metric.routed_provider.as_deref(),
        metric.routed_model.as_deref(),
    ) {
        (None, None) => true,
        (Some(provider), Some(model)) => {
            metric.canonical_provider_id == "openrouter"
                && valid_router_label(provider)
                && valid_label(model, 128)
        }
        _ => false,
    }
}

fn valid_route(metric: &ProviderRequestMetric) -> bool {
    let (provider, format) = match metric.connection_id.as_str() {
        "codex-oauth" => ("openai", super::UsageApiFormat::Responses),
        "xai-oauth" => ("xai", super::UsageApiFormat::ChatCompletions),
        "moonshot-oauth" => ("moonshot", super::UsageApiFormat::ChatCompletions),
        provider => (provider, super::UsageApiFormat::ChatCompletions),
    };
    metric.canonical_provider_id == provider && metric.api_format == format
}

pub(super) fn valid_router_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.contains("..")
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(
                    character,
                    ' ' | '-' | '_' | '.' | '/' | ':' | '@' | '+' | '(' | ')'
                )
        })
}

fn valid_provider(value: &str) -> bool {
    matches!(
        value,
        "groq"
            | "google"
            | "mistral"
            | "cerebras"
            | "openrouter"
            | "openai"
            | "deepseek"
            | "xai"
            | "moonshot"
            | "zai"
    )
}

fn valid_started_at(value: i64) -> bool {
    value >= MIN_STARTED_AT_MS
        && value
            <= chrono::Utc::now()
                .timestamp_millis()
                .saturating_add(MAX_CLOCK_SKEW_MS)
}

fn valid_timing(timing: &RequestTiming) -> bool {
    timing.total_ms <= MAX_DURATION_MS
        && [
            timing.headers_ms,
            timing.first_event_ms,
            timing.first_useful_ms,
        ]
        .into_iter()
        .flatten()
        .all(|value| value <= timing.total_ms && value <= MAX_DURATION_MS)
        && ordered(timing.headers_ms, timing.first_event_ms)
        && ordered(timing.first_event_ms, timing.first_useful_ms)
        && ordered(timing.headers_ms, timing.first_useful_ms)
}

fn ordered(earlier: Option<u64>, later: Option<u64>) -> bool {
    earlier
        .zip(later)
        .is_none_or(|(earlier, later)| earlier <= later)
}

pub(super) fn valid_label(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && !value.contains("..")
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '-' | '_' | '.' | '/' | ':' | '@' | '+')
        })
}
