use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::summary_contract::{SummaryRawOutput, ValidatedSummary};
use super::summary_request::{SummaryAttemptError, SummaryCall, SummaryCollector};

const MAX_SUMMARY_INPUT_TOKENS: u32 = 1_000_000;
const SUMMARY_INPUT_SAFETY_TOKENS: u32 = 256;

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
    let band_kind = snapshot
        .profile
        .band(snapshot.context_window)
        .unwrap_or(super::profile_types::CompressionWindowBand::Compact);
    let band = snapshot.profile.profile.band_settings(band_kind);
    let target = super::checkpoint_target::checkpoint_target(
        snapshot.before_tokens,
        snapshot.system_head_tokens,
        band_kind,
    );
    let available = target.saturating_sub(snapshot.system_head_tokens);
    let output_limit =
        super::checkpoint_target::effective_summary_limit(band.summary_max_tokens, available)
            .map_err(|_| super::checkpoint_transaction::CompressionError::CapacityExceeded)?;
    let prompts = super::summary_request::SummaryPromptConfig {
        system_prompt: snapshot.profile.profile.system_prompt.clone(),
        handoff_request: snapshot.profile.profile.handoff_prompt.clone(),
    };
    let empty_call = super::summary_request::build_call(
        &[],
        &prompts,
        &snapshot.provider_id,
        &snapshot.source_session.model,
        1,
        output_limit,
    );
    let fixed_input = super::token_estimate::estimate_textual_request_tokens_for_provider(
        &snapshot.provider_id,
        &empty_call.messages,
        &[],
    )
    .min(u32::MAX as usize) as u32;
    let input_window = if snapshot.context_window > 0 {
        snapshot.context_window.min(u64::from(u32::MAX)) as u32
    } else {
        snapshot.before_tokens
    };
    if input_window
        <= fixed_input
            .saturating_add(output_limit)
            .saturating_add(SUMMARY_INPUT_SAFETY_TOKENS)
    {
        return Err(super::checkpoint_transaction::CompressionError::CapacityExceeded);
    }
    let history_limit = input_window
        .saturating_sub(output_limit)
        .saturating_sub(fixed_input)
        .saturating_sub(SUMMARY_INPUT_SAFETY_TOKENS)
        .clamp(1, MAX_SUMMARY_INPUT_TOKENS);
    let call = super::summary_request::build_call(
        &snapshot.source_messages,
        &prompts,
        &snapshot.provider_id,
        &snapshot.source_session.model,
        history_limit,
        output_limit,
    );
    match super::summary_request::execute(collector, &call, 0).await {
        Ok(summary) => Ok(Some(summary)),
        Err(super::summary_request::SummaryExecutionError::InvalidOutput) => {
            Err(super::checkpoint_transaction::CompressionError::SummaryInvalid)
        }
        Err(super::summary_request::SummaryExecutionError::Cancelled) => {
            Err(super::checkpoint_transaction::CompressionError::Cancelled)
        }
        Err(
            super::summary_request::SummaryExecutionError::Retryable
            | super::summary_request::SummaryExecutionError::Fatal,
        ) => Err(super::checkpoint_transaction::CompressionError::SummaryRequestFailed),
    }
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
