use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::summary_contract::{SummaryRawOutput, ValidatedSummary};
use super::summary_request::{SummaryAttemptError, SummaryCall, SummaryCollector};

pub struct ProviderSummaryCollector<'a> {
    pub session_id: &'a str,
    pub request_id: &'a str,
    pub fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    pub cancel: CancellationToken,
}

#[async_trait]
impl SummaryCollector for ProviderSummaryCollector<'_> {
    async fn collect(&self, call: &SummaryCall) -> Result<SummaryRawOutput, SummaryAttemptError> {
        if self.cancel.is_cancelled() {
            return Err(SummaryAttemptError::Cancelled);
        }
        let purpose =
            crate::services::llm::request_purpose::RequestPurpose::for_session(self.session_id)
                .await;
        let result = crate::services::llm::stream::collect_chat_silent_for_compression(
            &call.provider,
            self.fast_mode,
            &call.model,
            &call.messages,
            call.maximum_output_tokens,
            purpose,
            self.session_id,
            Some(self.request_id),
            self.cancel.clone(),
        )
        .await
        .map_err(|error| classify(&error, self.cancel.is_cancelled()))?;
        crate::services::provider_usage::record_for_session(
            &call.provider,
            &call.model,
            self.session_id,
            crate::services::provider_usage::UsageWorkload::Compression,
            result.usage.as_ref(),
        )
        .await;
        Ok(SummaryRawOutput {
            content: result.content,
            tool_call_count: result.tool_calls.len(),
            truncated: result.done_reason.as_deref() == Some("length"),
            cancelled: self.cancel.is_cancelled(),
        })
    }
}

pub async fn generate(
    snapshot: &super::snapshot::CompressionSnapshot,
    collector: &dyn SummaryCollector,
) -> Result<Option<ValidatedSummary>, super::checkpoint_transaction::CompressionError> {
    if !snapshot.profile.profile.summary.enabled {
        return Ok(None);
    }
    let profile = &snapshot.profile.profile;
    let band = band(snapshot)?;
    let input_limit = super::profile_budget::resolve_budget(
        &profile.summary.input_budget,
        budget_window(snapshot),
    );
    let output_limit = super::profile_budget::summary_output_limit(
        &band.summary_output,
        budget_window(snapshot),
        u64::from(snapshot.before_tokens),
    );
    let (provider, model) = selected_model(
        &profile.summary.model,
        &snapshot.provider_id,
        &snapshot.source_session.model,
    );
    let call = build_call(snapshot, provider, model, input_limit, output_limit);
    match super::summary_request::execute(collector, &call, profile.summary.ordinary_retries).await
    {
        Ok(summary) => Ok(Some(summary)),
        Err(super::summary_request::SummaryExecutionError::Retryable)
            if profile.summary.failure_policy
                == super::profile_types::SummaryFailurePolicy::TryFallback =>
        {
            let Some(selection) = profile.summary.fallback_model.as_ref() else {
                return Err(super::checkpoint_transaction::CompressionError::SummaryInvalid);
            };
            let (provider, model) = selected_model(
                selection,
                &snapshot.provider_id,
                &snapshot.source_session.model,
            );
            let fallback = build_call(snapshot, provider, model, input_limit, output_limit);
            super::summary_request::execute(collector, &fallback, profile.summary.ordinary_retries)
                .await
                .map(Some)
                .map_err(|_| super::checkpoint_transaction::CompressionError::SummaryInvalid)
        }
        Err(_)
            if profile.summary.failure_policy
                == super::profile_types::SummaryFailurePolicy::DeterministicCheckpoint =>
        {
            Ok(None)
        }
        Err(_) => Err(super::checkpoint_transaction::CompressionError::SummaryInvalid),
    }
}

fn build_call(
    snapshot: &super::snapshot::CompressionSnapshot,
    provider: &str,
    model: &str,
    input_limit: u32,
    output_limit: u32,
) -> SummaryCall {
    super::summary_request::build_call(
        &snapshot.source_messages,
        &super::summary_request::SummaryPromptConfig {
            system_prompt: snapshot.profile.profile.summary.system_prompt.clone(),
            handoff_request: snapshot.profile.profile.summary.handoff_prompt.clone(),
        },
        provider,
        model,
        input_limit,
        output_limit,
    )
}

fn band(
    snapshot: &super::snapshot::CompressionSnapshot,
) -> Result<
    &super::profile_types::CompressionBandSettings,
    super::checkpoint_transaction::CompressionError,
> {
    match snapshot.profile.band(snapshot.context_window) {
        Some(super::profile_types::CompressionWindowBand::Under64K) => {
            Ok(&snapshot.profile.profile.under_64k)
        }
        Some(super::profile_types::CompressionWindowBand::Compact) | None => {
            Ok(&snapshot.profile.profile.compact)
        }
        Some(super::profile_types::CompressionWindowBand::Large) => {
            Ok(&snapshot.profile.profile.large)
        }
    }
}

fn selected_model<'a>(
    selection: &'a super::profile_types::SummaryModelSelection,
    current_provider: &'a str,
    current_model: &'a str,
) -> (&'a str, &'a str) {
    match selection {
        super::profile_types::SummaryModelSelection::Current => (current_provider, current_model),
        super::profile_types::SummaryModelSelection::Explicit { provider, model } => {
            (provider, model)
        }
    }
}

fn budget_window(snapshot: &super::snapshot::CompressionSnapshot) -> u64 {
    super::profile_budget::effective_budget_window(snapshot.context_window, snapshot.before_tokens)
}

pub(super) fn classify(error: &str, cancelled: bool) -> SummaryAttemptError {
    if cancelled {
        return SummaryAttemptError::Cancelled;
    }
    if matches!(
        error,
        "rate_limit" | "provider_temporarily_unavailable" | "provider_connection_failed"
    ) {
        SummaryAttemptError::Retryable
    } else {
        SummaryAttemptError::Fatal
    }
}
