use serde::Serialize;

use super::request_journal::{ProviderRequestMetric, RequestMetricStatus};

const SESSION_SNAPSHOT_LIMIT: usize = 50;

#[derive(Debug, Clone, Default, Serialize)]
pub struct RequestSessionSummary {
    pub session_id: String,
    pub attempt_count: u64,
    pub completed_count: u64,
    pub usage_complete_count: u64,
    pub cache_observation_count: u64,
    pub cache_read_observation_count: u64,
    pub cache_write_observation_count: u64,
    pub cache_miss_observation_count: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub cache_miss_tokens: u64,
    pub total_duration_ms: u64,
    pub latest_started_at_ms: i64,
}

pub(super) fn session_summaries(
    entries: &[ProviderRequestMetric],
    connection_id: &str,
) -> Vec<RequestSessionSummary> {
    let mut summaries = Vec::<RequestSessionSummary>::with_capacity(SESSION_SNAPSHOT_LIMIT);
    for metric in entries
        .iter()
        .rev()
        .filter(|metric| metric.connection_id == connection_id)
    {
        let Some(session_id) = metric.session_id.as_deref() else {
            continue;
        };
        let index = summaries
            .iter()
            .position(|summary| summary.session_id == session_id);
        let summary = match index {
            Some(index) => &mut summaries[index],
            None if summaries.len() < SESSION_SNAPSHOT_LIMIT => {
                summaries.push(RequestSessionSummary {
                    session_id: session_id.to_string(),
                    latest_started_at_ms: metric.started_at_ms,
                    ..Default::default()
                });
                summaries.last_mut().expect("summary was inserted")
            }
            None => continue,
        };
        add(summary, metric);
    }
    summaries
}

fn add(summary: &mut RequestSessionSummary, metric: &ProviderRequestMetric) {
    summary.attempt_count = summary.attempt_count.saturating_add(1);
    summary.completed_count = summary
        .completed_count
        .saturating_add(u64::from(metric.status == RequestMetricStatus::Completed));
    summary.usage_complete_count = summary
        .usage_complete_count
        .saturating_add(u64::from(metric.usage_complete));
    summary.total_duration_ms = summary
        .total_duration_ms
        .saturating_add(metric.timing.total_ms);
    let Some(usage) = &metric.usage else { return };
    if usage.cache_status == super::request_usage::CacheUsageStatus::Reported {
        summary.cache_observation_count = summary.cache_observation_count.saturating_add(1);
    }
    summary.cache_read_observation_count = summary
        .cache_read_observation_count
        .saturating_add(u64::from(usage.cached_input_tokens.is_some()));
    summary.cache_write_observation_count = summary
        .cache_write_observation_count
        .saturating_add(u64::from(usage.cache_write_input_tokens.is_some()));
    summary.cache_miss_observation_count = summary
        .cache_miss_observation_count
        .saturating_add(u64::from(usage.cache_miss_input_tokens.is_some()));
    summary.cache_read_tokens = summary
        .cache_read_tokens
        .saturating_add(usage.cached_input_tokens.unwrap_or(0));
    summary.cache_write_tokens = summary
        .cache_write_tokens
        .saturating_add(usage.cache_write_input_tokens.unwrap_or(0));
    summary.cache_miss_tokens = summary
        .cache_miss_tokens
        .saturating_add(usage.cache_miss_input_tokens.unwrap_or(0));
}
