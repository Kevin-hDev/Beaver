use std::time::Instant;

use super::checkpoint_transaction::CompressionError;
use super::metrics::{CompressionMetricContext, CompressionMetrics, CompressionSuccessFacts};
use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::CompressionTrigger;
use crate::services::provider_usage::CacheTokenTotals;

pub struct Completion<'a> {
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub provider_id: &'a str,
    pub profile: &'a ResolvedCompressionProfile,
    pub trigger: CompressionTrigger,
    pub context_window: u64,
    pub before_tokens: u32,
    pub previous_compression_count: u32,
    pub cache_before: CacheTokenTotals,
    pub facts: Option<CompressionSuccessFacts>,
    pub error: Option<CompressionError>,
    pub cancelled: bool,
    pub started_at: Instant,
}

pub async fn record(completion: Completion<'_>) {
    let cache_after =
        crate::services::provider_usage::compression_cache_totals(completion.provider_id).await;
    let metric = CompressionMetrics::finish(
        CompressionMetricContext {
            session_id: completion.session_id,
            request_id: completion.request_id,
            profile: completion.profile,
            trigger: completion.trigger,
            context_window: completion.context_window,
            before_tokens: completion.before_tokens,
            projected_budget_tokens: super::metrics_projection::projected_budget(
                completion.profile,
                completion.context_window,
                completion.before_tokens,
            ),
            compression_count: completion.previous_compression_count,
            cache_before: completion.cache_before,
        },
        completion.facts,
        completion.error,
        completion.cancelled,
        completion
            .started_at
            .elapsed()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64,
        cache_after,
    );
    crate::services::agent_local::stream_diagnostics::record_compression_metrics(
        completion.session_id,
        completion.request_id,
        &metric,
    )
    .await;
}
