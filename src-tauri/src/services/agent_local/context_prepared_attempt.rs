use super::context_usage_buckets::RequestContextUsage;
use super::context_usage_record::ContextTokenCount;
use super::context_usage_runtime::ContextAttempt;

pub struct PreparedContextAttempt<'a> {
    context: ContextAttempt<'a>,
    breakdown: RequestContextUsage,
    realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
}

impl<'a> PreparedContextAttempt<'a> {
    pub fn new(context: ContextAttempt<'a>, breakdown: RequestContextUsage) -> Self {
        Self {
            context,
            breakdown,
            realtime_budget: None,
        }
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
        self.context
            .persist_prepared_count(count, self.breakdown)
            .await
    }
}
