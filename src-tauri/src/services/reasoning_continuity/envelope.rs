use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::bounded_json::serialized_len_bounded;
use super::contract::{ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId};
use super::limits::{
    validate_json_depth, validate_model_id, validate_remote_response_id, LimitError,
    MAX_ENVELOPE_BYTES, MAX_NATIVE_ITEMS,
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

impl ReasoningSource {
    pub(crate) fn from_target(target: &ReplayTarget) -> Self {
        Self {
            route_id: target.route_id,
            model_id: target.model_id.clone(),
            credential_scope: target.credential_scope.clone(),
            reasoning_mode: target.reasoning_mode,
        }
    }

    pub(crate) fn matches_target(&self, target: &ReplayTarget) -> bool {
        self.route_id == target.route_id
            && self.model_id == target.model_id
            && self.credential_scope == target.credential_scope
            && self.reasoning_mode == target.reasoning_mode
    }

    pub(crate) fn validate(&self) -> Result<(), LimitError> {
        validate_model_id(&self.model_id)?;
        super::limits::validate_credential_scope(self.credential_scope.as_str())?;
        self.credential_scope.validate_for_route(self.route_id)
    }
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
    AnthropicBlocks { blocks: Vec<Value> },
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
        self.source.validate()?;
        tool_links::validate(&self.tool_links)?;
        validate_continuation(&self.continuation)?;
        if let ContinuationState::AnthropicBlocks { blocks } = &self.continuation {
            validate_anthropic_blocks(blocks, &self.tool_links)?;
        }
        serialized_len_bounded(self, MAX_ENVELOPE_BYTES).map(|_| ())
    }
}

fn validate_continuation(state: &ContinuationState) -> Result<(), LimitError> {
    match state {
        ContinuationState::AnthropicBlocks { blocks: parts }
        | ContinuationState::GeminiParts { parts }
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

fn validate_anthropic_blocks(blocks: &[Value], tool_links: &[ToolLink]) -> Result<(), LimitError> {
    validate_items(blocks)?;
    let mut tool_ids = std::collections::HashSet::new();
    for block in blocks {
        let kind = block
            .get("type")
            .and_then(Value::as_str)
            .ok_or(LimitError::CaptureSkeleton)?;
        match kind {
            "thinking" => {
                block
                    .get("thinking")
                    .and_then(Value::as_str)
                    .ok_or(LimitError::CaptureSkeleton)?;
                block
                    .get("signature")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or(LimitError::CaptureSkeleton)?;
            }
            "redacted_thinking" => {
                block
                    .get("data")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or(LimitError::CaptureSkeleton)?;
            }
            "text" => {
                block
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or(LimitError::CaptureSkeleton)?;
            }
            "tool_use" => {
                let id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(LimitError::ProviderCallId)?;
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or(LimitError::ToolName)?;
                super::limits::validate_provider_call_id(id)?;
                super::limits::validate_tool_name(name)?;
                if !block.get("input").is_some_and(Value::is_object) || !tool_ids.insert(id) {
                    return Err(LimitError::CaptureSkeleton);
                }
            }
            _ => return Err(LimitError::CaptureSkeleton),
        }
    }
    let linked_ids = tool_links
        .iter()
        .map(|link| link.provider_call_id.as_str())
        .collect::<std::collections::HashSet<_>>();
    (tool_ids == linked_ids)
        .then_some(())
        .ok_or(LimitError::ProviderCallId)
}

fn validate_items(items: &[Value]) -> Result<(), LimitError> {
    if items.len() > MAX_NATIVE_ITEMS {
        return Err(LimitError::NativeItems);
    }
    for item in items {
        validate_json_depth(item)?;
    }
    Ok(())
}
