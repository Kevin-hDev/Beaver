mod capture;
mod chat_text;
mod gemini;
mod mistral;
mod openrouter;
#[allow(
    dead_code,
    reason = "Task 18 prepares adapters; Task 19 alone connects live-validated routes"
)]
pub(crate) mod replay;
mod responses;
mod tool_link_capture;

use crate::services::reasoning_continuity::bounded_json::serialized_len_bounded_from;
use crate::services::reasoning_continuity::capture_budget::CaptureBudget;
use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use crate::services::reasoning_continuity::limits::{LimitError, MAX_ENVELOPE_BYTES};
use capture::{contract_for, empty_continuation, has_native_items};
use serde_json::Value;

/// Provenance fixée avant lecture du premier événement provider.
#[derive(Debug, Clone)]
pub(crate) struct ReasoningCaptureContext {
    pub route_id: RouteId,
    pub model_id: String,
    pub credential_scope: CredentialScope,
    pub reasoning_mode: ReasoningModeId,
}

impl ReasoningCaptureContext {
    pub(crate) fn from_target(
        target: &crate::services::reasoning_continuity::contract::ReplayTarget,
    ) -> Self {
        Self {
            route_id: target.route_id,
            model_id: target.model_id.clone(),
            credential_scope: target.credential_scope.clone(),
            reasoning_mode: target.reasoning_mode,
        }
    }

    fn source(&self) -> ReasoningSource {
        ReasoningSource {
            route_id: self.route_id,
            model_id: self.model_id.clone(),
            credential_scope: self.credential_scope.clone(),
            reasoning_mode: self.reasoning_mode,
        }
    }
}

/// Capture native bornée : le premier dépassement ferme définitivement le rejeu.
pub(crate) struct ReasoningCapture {
    context: ReasoningCaptureContext,
    contract_id: ContractId,
    continuation: Option<ContinuationState>,
    budget: Option<CaptureBudget>,
    text_bytes: usize,
    provider_complete: bool,
    partial: bool,
    failure_code: Option<&'static str>,
    response_item_events_seen: bool,
    response_tool_links: Vec<crate::services::reasoning_continuity::tool_links::ToolLink>,
}

impl ReasoningCapture {
    pub(crate) fn new(context: ReasoningCaptureContext) -> Result<Self, LimitError> {
        let contract_id = contract_for(context.route_id).ok_or(LimitError::CaptureSkeleton)?;
        let continuation = empty_continuation(contract_id);
        let skeleton = ReasoningEnvelope::new(
            contract_id,
            context.source(),
            CompletionState::Complete,
            continuation.clone(),
            Vec::new(),
        );
        let budget = has_native_items(&continuation)
            .then(|| CaptureBudget::from_envelope_skeleton(&skeleton))
            .transpose()?;
        let text_bytes = serde_json::to_vec(&skeleton)
            .map_err(|_| LimitError::EnvelopeBytes)?
            .len();
        Ok(Self {
            context,
            contract_id,
            continuation: Some(continuation),
            budget,
            text_bytes,
            provider_complete: false,
            partial: false,
            failure_code: None,
            response_item_events_seen: false,
            response_tool_links: Vec::new(),
        })
    }

    pub(crate) fn observe_json(&mut self, event: &Value) {
        if self.partial {
            return;
        }
        let result = match self.contract_id {
            ContractId::OllamaNativeV1 => self.append_ollama(event),
            ContractId::GeminiCompatV1 => self.append_items(gemini::parts(event)),
            ContractId::MistralChunksV1 => self.append_items(mistral::chunks(event)),
            ContractId::OpenRouterDetailsV1 => self.append_items(openrouter::details(event)),
            ContractId::OpenAiResponsesV1
            | ContractId::XaiResponsesV1
            | ContractId::CodexResponsesV1 => self.append_response_items(event),
            ContractId::CerebrasChatV1 => self.append_chat(event, true),
            ContractId::DeepSeekChatV1 | ContractId::KimiChatV1 | ContractId::ZaiChatV1 => {
                self.append_chat(event, false)
            }
        };
        if result.is_err() {
            self.mark_partial();
        }
    }

    pub(crate) fn observe_done(&mut self, event: &Value) {
        if self.partial {
            return;
        }
        self.provider_complete = match self.contract_id {
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

    pub(crate) fn finish_complete(&mut self) -> Option<ReasoningEnvelope> {
        if self.partial || !self.provider_complete {
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

    #[cfg(test)]
    pub(crate) const fn is_partial(&self) -> bool {
        self.partial
    }

    #[cfg(test)]
    pub(crate) const fn failure_code(&self) -> Option<&'static str> {
        self.failure_code
    }

    fn append_ollama(&mut self, event: &Value) -> Result<(), LimitError> {
        if let Some(thinking) = event.pointer("/message/thinking").and_then(Value::as_str) {
            self.append_text(thinking, false)?;
        }
        Ok(())
    }

    fn append_chat(&mut self, event: &Value, cerebras: bool) -> Result<(), LimitError> {
        for fragment in chat_text::fragments(event) {
            self.append_text(fragment, cerebras)?;
        }
        Ok(())
    }

    fn append_text(&mut self, fragment: &str, cerebras: bool) -> Result<(), LimitError> {
        if fragment.is_empty() {
            return Ok(());
        }
        let next = serialized_len_bounded_from(fragment, self.text_bytes, MAX_ENVELOPE_BYTES)?;
        let continuation = self
            .continuation
            .as_mut()
            .ok_or(LimitError::CaptureClosed)?;
        match continuation {
            ContinuationState::OllamaNative { thinking } => thinking.push_str(fragment),
            ContinuationState::ChatReasoning { reasoning_content } if !cerebras => {
                reasoning_content.push_str(fragment)
            }
            ContinuationState::CerebrasReasoning { reasoning } if cerebras => {
                reasoning.push_str(fragment)
            }
            _ => return Err(LimitError::CaptureSkeleton),
        }
        self.text_bytes = next;
        Ok(())
    }

    fn append_items(&mut self, items: Vec<Value>) -> Result<(), LimitError> {
        for item in items {
            self.budget
                .as_mut()
                .ok_or(LimitError::CaptureSkeleton)?
                .observe_item(&item)?;
            let continuation = self
                .continuation
                .as_mut()
                .ok_or(LimitError::CaptureClosed)?;
            match continuation {
                ContinuationState::GeminiParts { parts }
                | ContinuationState::MistralChunks { chunks: parts }
                | ContinuationState::OpenRouterDetails { details: parts }
                | ContinuationState::ResponsesLocal { items: parts } => parts.push(item),
                _ => return Err(LimitError::CaptureSkeleton),
            }
        }
        Ok(())
    }

    fn append_response_items(&mut self, event: &Value) -> Result<(), LimitError> {
        let item = responses::completed_item(event).map_err(|_| LimitError::CaptureSkeleton)?;
        if let Some(item) = item {
            self.response_item_events_seen = true;
            self.append_response_item(item)?;
            return Ok(());
        }
        if !self.response_item_events_seen {
            for item in responses::final_items(event).map_err(|_| LimitError::CaptureSkeleton)? {
                self.append_response_item(item)?;
            }
        }
        Ok(())
    }

    fn append_response_item(&mut self, item: Value) -> Result<(), LimitError> {
        responses::tool_link(&item).map_err(|_| LimitError::CaptureSkeleton)?;
        self.append_items(vec![item])?;
        Ok(())
    }

    fn mark_partial(&mut self) {
        self.partial = true;
        self.failure_code = Some("capture_limit_exceeded");
        self.continuation = None;
        self.budget = None;
    }
}

#[cfg(test)]
mod tests;
