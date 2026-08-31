use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::summary_contract::{SummaryRawOutput, ValidatedSummary};
use super::summary_request::{SummaryAttemptError, SummaryCall, SummaryCollector};

const MAX_SUMMARY_INPUT_TOKENS: u32 = 1_000_000;

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
    let call = super::summary_request::build_call(
        &snapshot.source_messages,
        &super::summary_request::SummaryPromptConfig {
            system_prompt: snapshot.profile.profile.system_prompt.clone(),
            handoff_request: snapshot.profile.profile.handoff_prompt.clone(),
        },
        &snapshot.provider_id,
        &snapshot.source_session.model,
        snapshot.before_tokens.clamp(1, MAX_SUMMARY_INPUT_TOKENS),
        output_limit,
    );
    super::summary_request::execute(collector, &call, 0)
        .await
        .map(Some)
        .map_err(|_| super::checkpoint_transaction::CompressionError::SummaryInvalid)
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
