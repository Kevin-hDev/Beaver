use serde::Serialize;

use super::checkpoint_transaction::CompressionError;
pub use super::metrics_error::{CompressionMetricError, CompressionMetricPhase};
use super::profile_resolve::ResolvedCompressionProfile;
use super::profile_types::{CompressionTrigger, CompressionWindowBand};
use crate::services::provider_usage::CacheTokenTotals;

const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionMetricOutcome {
    Success,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompressionMetrics {
    pub session_id: String,
    pub request_id: String,
    pub profile_id: String,
    pub band: Option<CompressionWindowBand>,
    pub under_64k_allowed: bool,
    pub threshold_percent: u8,
    pub effective_threshold_percent: u8,
    pub trigger: CompressionTrigger,
    pub phase: CompressionMetricPhase,
    pub before_tokens: u32,
    pub system_head_tokens: u32,
    pub target_tokens: u32,
    pub after_tokens: u32,
    pub reduction_tokens: u32,
    pub summary_tokens: u32,
    pub retained_messages: u16,
    pub retained_user_tokens: u32,
    pub retained_tool_results: u16,
    pub dropped_tool_results: u16,
    pub retained_images: u16,
    pub dropped_images: u16,
    pub retained_subagent_reports: u16,
    pub target_overflow_tokens: u32,
    pub guard_consecutive_failures: u8,
    pub guard_suspended: bool,
    pub duration_ms: u64,
    pub outcome: CompressionMetricOutcome,
    pub error: Option<CompressionMetricError>,
    pub compression_count: u32,
    pub cache_read_tokens_before: u64,
    pub cache_read_tokens_after: u64,
    pub cache_write_tokens_before: u64,
    pub cache_write_tokens_after: u64,
}

pub struct CompressionMetricContext<'a> {
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub profile: &'a ResolvedCompressionProfile,
    pub trigger: CompressionTrigger,
    pub context_window: u64,
    pub before_tokens: u32,
    pub system_head_tokens: u32,
    pub target_tokens: u32,
    pub guard_consecutive_failures: u8,
    pub guard_suspended: bool,
    pub compression_count: u32,
    pub cache_before: CacheTokenTotals,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompressionSuccessFacts {
    pub after_tokens: u32,
    pub summary_tokens: u32,
    pub retained_messages: u16,
    pub retained_user_tokens: u32,
    pub retained_tool_results: u16,
    pub dropped_tool_results: u16,
    pub retained_images: u16,
    pub dropped_images: u16,
    pub retained_subagent_reports: u16,
    pub compression_count: u32,
}

impl CompressionMetrics {
    pub fn finish(
        context: CompressionMetricContext<'_>,
        facts: Option<CompressionSuccessFacts>,
        error: Option<CompressionError>,
        cancelled: bool,
        duration_ms: u64,
        cache_after: CacheTokenTotals,
    ) -> Self {
        let facts = facts.unwrap_or_default();
        let projected_tokens = if facts.after_tokens == 0 {
            context.before_tokens
        } else {
            facts.after_tokens
        };
        let metric_error = error.map(CompressionMetricError::from);
        let outcome = if cancelled {
            CompressionMetricOutcome::Cancelled
        } else if metric_error.is_some() {
            CompressionMetricOutcome::Failed
        } else {
            CompressionMetricOutcome::Success
        };
        Self {
            session_id: technical_id(context.session_id, false),
            request_id: technical_id(context.request_id, false),
            profile_id: technical_id(&context.profile.profile.id, true),
            band: context.profile.band(context.context_window),
            under_64k_allowed: context.profile.profile.allow_under_64k,
            threshold_percent: context.profile.profile.threshold_percent,
            effective_threshold_percent: context.profile.profile.threshold_percent.min(90),
            trigger: context.trigger,
            phase: metric_error.map_or(CompressionMetricPhase::Commit, |error| error.phase()),
            before_tokens: context.before_tokens,
            system_head_tokens: context.system_head_tokens,
            target_tokens: context.target_tokens,
            after_tokens: facts.after_tokens,
            reduction_tokens: context.before_tokens.saturating_sub(facts.after_tokens),
            summary_tokens: facts.summary_tokens,
            retained_messages: facts.retained_messages,
            retained_user_tokens: facts.retained_user_tokens,
            retained_tool_results: facts.retained_tool_results,
            dropped_tool_results: facts.dropped_tool_results,
            retained_images: facts.retained_images,
            dropped_images: facts.dropped_images,
            retained_subagent_reports: facts.retained_subagent_reports,
            target_overflow_tokens: projected_tokens.saturating_sub(context.target_tokens),
            guard_consecutive_failures: context.guard_consecutive_failures.min(3),
            guard_suspended: context.guard_suspended,
            duration_ms: duration_ms.min(MAX_DURATION_MS),
            outcome,
            error: metric_error,
            compression_count: facts.compression_count.max(context.compression_count),
            cache_read_tokens_before: context.cache_before.read_tokens,
            cache_read_tokens_after: cache_after.read_tokens,
            cache_write_tokens_before: context.cache_before.write_tokens,
            cache_write_tokens_after: cache_after.write_tokens,
        }
    }

    pub fn safe_log_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

fn technical_id(value: &str, allow_beaver: bool) -> String {
    if allow_beaver && value == super::profile_defaults::BEAVER_PROFILE_ID {
        return value.to_string();
    }
    uuid::Uuid::parse_str(value)
        .map(|id| id.to_string())
        .unwrap_or_else(|_| "invalid".to_string())
}
