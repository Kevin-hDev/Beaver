mod capture;
mod capture_completion;
mod capture_context;
pub(crate) mod chat_text;
mod gemini;
mod mistral;
mod openrouter;
#[allow(
    dead_code,
    reason = "Task 18 prepares adapters; Task 19 alone connects live-validated routes"
)]
pub(crate) mod replay;
pub(crate) mod responses;
mod tool_link_capture;

use crate::services::reasoning_continuity::bounded_json::serialized_len_bounded_from;
use crate::services::reasoning_continuity::capture_budget::CaptureBudget;
use crate::services::reasoning_continuity::contract::ContractId;
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope,
};
use crate::services::reasoning_continuity::limits::{LimitError, MAX_ENVELOPE_BYTES};
use capture::{contract_for, empty_continuation, has_native_items};
use serde_json::Value;

pub(crate) use capture_context::ReasoningCaptureContext;

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
}

#[cfg(test)]
mod tests;
