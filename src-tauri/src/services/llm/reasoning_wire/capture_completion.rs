use super::ReasoningCapture;
use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::envelope::{CompletionState, ReasoningEnvelope};
use serde_json::Value;

impl ReasoningCapture {
    pub(crate) fn observe_done(&mut self, event: &Value) {
        if self.partial {
            return;
        }
        self.provider_complete = match self.contract_id {
            ContractId::AnthropicMessagesV1 => {
                event.get("type").and_then(Value::as_str) == Some("message_stop")
            }
            ContractId::OllamaNativeV1 => event.get("done").and_then(Value::as_bool) == Some(true),
            ContractId::OpenAiResponsesV1
            | ContractId::XaiResponsesV1
            | ContractId::CodexResponsesV1 => {
                event.get("type").and_then(Value::as_str) == Some("response.completed")
            }
            _ => event
                .pointer("/choices/0/finish_reason")
                .and_then(Value::as_str)
                .is_some_and(|reason| !reason.is_empty() && reason != "error"),
        };
    }

    /// `[DONE]` est le signal terminal natif de plusieurs API Chat Completions.
    /// Responses exige son événement structuré `response.completed` et ne passe jamais ici.
    pub(crate) fn observe_transport_complete(&mut self) {
        if self.partial {
            return;
        }
        if !matches!(
            self.contract_id,
            ContractId::OllamaNativeV1
                | ContractId::OpenAiResponsesV1
                | ContractId::XaiResponsesV1
                | ContractId::CodexResponsesV1
        ) {
            self.provider_complete = true;
        }
    }

    pub(crate) fn finish_complete(&mut self) -> Option<ReasoningEnvelope> {
        if self.partial || !self.provider_complete {
            return None;
        }
        if matches!(
            self.continuation.as_ref(),
            Some(crate::services::reasoning_continuity::envelope::ContinuationState::MistralChunks { chunks })
                if chunks.is_empty()
        ) || matches!(
            self.continuation.as_ref(),
            Some(crate::services::reasoning_continuity::envelope::ContinuationState::AnthropicBlocks { blocks })
                if blocks.is_empty()
        ) {
            return None;
        }
        let envelope = ReasoningEnvelope::new(
            self.contract_id,
            self.context.source(),
            CompletionState::Complete,
            self.continuation.take()?,
            std::mem::take(&mut self.response_tool_links),
        );
        envelope.validate().ok().map(|_| envelope)
    }

    pub(crate) fn finish_partial(&mut self) -> Option<ReasoningEnvelope> {
        self.mark_partial();
        None
    }

    pub(super) fn mark_partial(&mut self) {
        self.partial = true;
        self.failure_code = Some("capture_limit_exceeded");
        self.continuation = None;
        self.budget = None;
    }
}
