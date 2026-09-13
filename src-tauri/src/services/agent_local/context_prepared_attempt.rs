use super::context_usage_buckets::RequestContextUsage;
use super::context_usage_record::ContextTokenCount;
use super::context_usage_runtime::ContextAttempt;

pub struct PreparedContextAttempt<'a> {
    context: ContextAttempt<'a>,
    breakdown: RequestContextUsage,
    baseline_capacity_tokens: Option<u32>,
    realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
}

impl<'a> PreparedContextAttempt<'a> {
    pub fn new(context: ContextAttempt<'a>, breakdown: RequestContextUsage) -> Self {
        Self {
            context,
            breakdown,
            baseline_capacity_tokens: None,
            realtime_budget: None,
        }
    }

    pub fn with_baseline_count(mut self, count: ContextTokenCount) -> Self {
        self.baseline_capacity_tokens = count.capacity_tokens;
        self
    }

    pub fn with_realtime_budget(
        mut self,
        realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
    ) -> Self {
        self.realtime_budget = realtime_budget;
        self
    }

    pub async fn persist_payload(&self, count: ContextTokenCount) -> Result<(), String> {
        if let Some(budget) = &self.realtime_budget {
            budget.attach_prepared_count(&count);
        }
        let transient_overhead_tokens = count
            .capacity_tokens
            .zip(self.baseline_capacity_tokens)
            .map_or(0, |(prepared, baseline)| prepared.saturating_sub(baseline));
        self.context
            .persist_prepared_count(count, self.breakdown, transient_overhead_tokens)
            .await
    }
}
