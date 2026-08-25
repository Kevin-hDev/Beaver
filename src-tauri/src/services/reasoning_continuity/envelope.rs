use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::contract::{ContractId, CredentialScope, ReasoningModeId, RouteId};
use super::limits::{
    validate_model_id, validate_remote_response_id, CaptureBudget, LimitError, MAX_ENVELOPE_BYTES,
};
use super::tool_links::{self, ToolLink};

pub const REASONING_ENVELOPE_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionState {
    Complete,
    Partial,
    Compacted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningSource {
    pub route_id: RouteId,
    pub model_id: String,
    pub credential_scope: CredentialScope,
    pub reasoning_mode: ReasoningModeId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContinuationState {
    OllamaNative { thinking: String },
    ChatReasoning { reasoning_content: String },
    CerebrasReasoning { reasoning: String },
    GeminiParts { parts: Vec<Value> },
    MistralChunks { chunks: Vec<Value> },
    OpenRouterDetails { details: Vec<Value> },
    ResponsesLocal { items: Vec<Value> },
    RemoteContinuation { response_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningEnvelope {
    pub schema_version: u16,
    pub contract_id: ContractId,
    pub source: ReasoningSource,
    pub completion: CompletionState,
    pub continuation: ContinuationState,
    pub tool_links: Vec<ToolLink>,
}

impl ReasoningEnvelope {
    pub fn new(
        contract_id: ContractId,
        source: ReasoningSource,
        completion: CompletionState,
        continuation: ContinuationState,
        tool_links: Vec<ToolLink>,
    ) -> Self {
        Self {
            schema_version: REASONING_ENVELOPE_SCHEMA_VERSION,
            contract_id,
            source,
            completion,
            continuation,
            tool_links,
        }
    }

    pub fn validate(&self) -> Result<(), LimitError> {
        if self.schema_version != REASONING_ENVELOPE_SCHEMA_VERSION {
            return Err(LimitError::SchemaVersion);
        }
        validate_model_id(&self.source.model_id)?;
        super::limits::validate_credential_scope(self.source.credential_scope.as_str())?;
        tool_links::validate(&self.tool_links)?;
        validate_continuation(&self.continuation)?;
        let bytes = serde_json::to_vec(self)
            .map_err(|_| LimitError::EnvelopeBytes)?
            .len();
        (bytes <= MAX_ENVELOPE_BYTES)
            .then_some(())
            .ok_or(LimitError::EnvelopeBytes)
    }
}

fn validate_continuation(state: &ContinuationState) -> Result<(), LimitError> {
    match state {
        ContinuationState::GeminiParts { parts }
        | ContinuationState::MistralChunks { chunks: parts }
        | ContinuationState::OpenRouterDetails { details: parts }
        | ContinuationState::ResponsesLocal { items: parts } => validate_items(parts),
        ContinuationState::RemoteContinuation { response_id } => {
            validate_remote_response_id(response_id)
        }
        ContinuationState::OllamaNative { .. }
        | ContinuationState::ChatReasoning { .. }
        | ContinuationState::CerebrasReasoning { .. } => Ok(()),
    }
}

fn validate_items(items: &[Value]) -> Result<(), LimitError> {
    let mut budget = CaptureBudget::new();
    for item in items {
        budget.observe_item(item)?;
    }
    Ok(())
}
