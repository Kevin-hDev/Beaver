use std::sync::atomic::{AtomicU32, Ordering};

use super::context_usage_buckets::RequestContextUsage;
use super::context_usage_record::ContextTokenCount;
use super::context_usage_runtime::ContextAttempt;

pub struct PreparedContextAttempt<'a> {
    context: ContextAttempt<'a>,
    breakdown: RequestContextUsage,
    input_tokens: AtomicU32,
    realtime_budget: Option<crate::services::compress::realtime_budget::RealtimeBudget>,
}

impl<'a> PreparedContextAttempt<'a> {
    pub fn new(context: ContextAttempt<'a>, breakdown: RequestContextUsage) -> Self {
        Self {
            context,
            breakdown,
            input_tokens: AtomicU32::new(0),
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
        let tokens = self
            .context
            .persist_prepared_count(count, self.breakdown)
            .await?;
        self.input_tokens.store(tokens, Ordering::Relaxed);
        Ok(())
    }

    pub fn input_tokens(&self) -> u32 {
        self.input_tokens.load(Ordering::Relaxed)
    }
}
