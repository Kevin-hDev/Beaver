use serde_json::Value;

use super::bounded_json::serialized_len_bounded_from;
use super::envelope::{ContinuationState, ReasoningEnvelope};
use super::limits::{
    checked_envelope_bytes, validate_json_depth, LimitError, MAX_ENVELOPE_BYTES, MAX_NATIVE_ITEMS,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureBudget {
    item_count: usize,
    serialized_bytes: usize,
    closed: bool,
}

impl CaptureBudget {
    pub fn from_envelope_skeleton(skeleton: &ReasoningEnvelope) -> Result<Self, LimitError> {
        if !has_empty_native_collection(&skeleton.continuation) {
            return Err(LimitError::CaptureSkeleton);
        }
        skeleton.validate()?;
        Ok(Self {
            item_count: 0,
            serialized_bytes: super::bounded_json::serialized_len_bounded(
                skeleton,
                MAX_ENVELOPE_BYTES,
            )?,
            closed: false,
        })
    }

    pub fn observe_item(&mut self, item: &Value) -> Result<(), LimitError> {
        self.ensure_open()?;
        let result = self.checked_item(item);
        if result.is_err() {
            self.closed = true;
        }
        result
    }

    #[cfg(test)]
    pub const fn item_count(&self) -> usize {
        self.item_count
    }

    #[cfg(test)]
    pub const fn serialized_bytes(&self) -> usize {
        self.serialized_bytes
    }

    fn checked_item(&mut self, item: &Value) -> Result<(), LimitError> {
        validate_json_depth(item)?;
        let next_items = self
            .item_count
            .checked_add(1)
            .ok_or(LimitError::ArithmeticOverflow)?;
        if next_items > MAX_NATIVE_ITEMS {
            return Err(LimitError::NativeItems);
        }
        let with_separator = if self.item_count == 0 {
            self.serialized_bytes
        } else {
            checked_envelope_bytes(self.serialized_bytes, 1)?
        };
        let next_bytes = serialized_len_bounded_from(item, with_separator, MAX_ENVELOPE_BYTES)?;
        self.item_count = next_items;
        self.serialized_bytes = next_bytes;
        Ok(())
    }

    fn ensure_open(&self) -> Result<(), LimitError> {
        (!self.closed)
            .then_some(())
            .ok_or(LimitError::CaptureClosed)
    }
}

fn has_empty_native_collection(state: &ContinuationState) -> bool {
    match state {
        ContinuationState::AnthropicBlocks { blocks: parts }
        | ContinuationState::GeminiParts { parts }
        | ContinuationState::MistralChunks { chunks: parts }
        | ContinuationState::OpenRouterDetails { details: parts }
        | ContinuationState::ResponsesLocal { items: parts } => parts.is_empty(),
        _ => false,
    }
}
