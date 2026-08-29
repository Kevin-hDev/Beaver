use std::collections::BTreeMap;

use serde_json::Value;

use super::stream_state_support::*;
use crate::services::provider_usage::{RequestUsage, UsageContext};
use crate::services::reasoning_continuity::limits::{MAX_ENVELOPE_BYTES, MAX_NATIVE_ITEMS};

#[derive(Debug)]
enum Block {
    Thinking { value: Value },
    Text { value: Value },
    Tool { value: Value, partial_json: String },
}

#[derive(Debug, Default)]
pub(super) struct StreamState {
    blocks: BTreeMap<usize, Block>,
    completed_blocks: Vec<Value>,
    total_bytes: usize,
    pub content: String,
    pub tool_calls: Vec<(String, Value)>,
    pub tool_call_ids: Vec<String>,
    pub usage: Option<RequestUsage>,
    pub finish_reason: Option<String>,
    complete: bool,
}

impl StreamState {
    pub fn apply(&mut self, event: &Value, context: UsageContext<'_>) -> Result<(), String> {
        match event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "message_start" => self.apply_usage(&event["message"]["usage"], context),
            "content_block_start" => self.start_block(event),
            "content_block_delta" => self.apply_delta(event),
            "content_block_stop" => self.finish_block(event),
            "message_delta" => self.apply_message_delta(event, context),
            "message_stop" => {
                self.complete = true;
                Ok(())
            }
            "ping" | "" => Ok(()),
            "error" => Err("provider_request_rejected".into()),
            _ => Ok(()),
        }
    }

    pub fn finish(self) -> Result<ConsumedStream, String> {
        if !self.complete || !self.blocks.is_empty() {
            return Err("provider_stream_invalid".into());
        }
        Ok(self.into_consumed())
    }

    pub fn finish_partial(self) -> Result<ConsumedStream, String> {
        if self.has_pending_tool() {
            return Err("provider_stream_invalid".into());
        }
        Ok(self.into_consumed())
    }

    pub fn has_pending_tool(&self) -> bool {
        self.blocks
            .values()
            .any(|block| matches!(block, Block::Tool { .. }))
    }

    fn into_consumed(self) -> ConsumedStream {
        ConsumedStream {
            content: self.content,
            continuation_blocks: self.completed_blocks,
            tool_calls: self.tool_calls,
            tool_call_ids: self.tool_call_ids,
            usage: self.usage,
            finish_reason: self.finish_reason,
        }
    }

    fn start_block(&mut self, event: &Value) -> Result<(), String> {
        let index = index(event)?;
        if index >= MAX_NATIVE_ITEMS || self.blocks.contains_key(&index) {
            return Err("provider_stream_invalid".into());
        }
        let value = event.get("content_block").cloned().ok_or_else(invalid)?;
        self.add_bytes(serialized_len(&value)?)?;
        let block = match value.get("type").and_then(Value::as_str) {
            Some("thinking" | "redacted_thinking") => Block::Thinking { value },
            Some("text") => {
                if let Some(text) = value.get("text").and_then(Value::as_str) {
                    self.content.push_str(text);
                }
                Block::Text { value }
            }
            Some("tool_use") => Block::Tool {
                value,
                partial_json: String::new(),
            },
            _ => return Err(invalid()),
        };
        self.blocks.insert(index, block);
        Ok(())
    }

    fn apply_delta(&mut self, event: &Value) -> Result<(), String> {
        let index = index(event)?;
        let delta = event.get("delta").ok_or_else(invalid)?;
        let delta_type = delta
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(invalid)?;
        let text = match delta_type {
            "text_delta" => delta.get("text").and_then(Value::as_str),
            "thinking_delta" => delta.get("thinking").and_then(Value::as_str),
            "signature_delta" => delta.get("signature").and_then(Value::as_str),
            "input_json_delta" => delta.get("partial_json").and_then(Value::as_str),
            _ => return Err(invalid()),
        }
        .ok_or_else(invalid)?;
        self.add_bytes(text.len())?;
        let block = self.blocks.get_mut(&index).ok_or_else(invalid)?;
        match (block, delta_type) {
            (Block::Text { value }, "text_delta") => {
                append_field(value, "text", text)?;
                self.content.push_str(text);
            }
            (Block::Thinking { value }, "thinking_delta") => {
                append_field(value, "thinking", text)?;
            }
            (Block::Thinking { value }, "signature_delta") => {
                if value.get("signature").is_none() {
                    value["signature"] = Value::String(String::new());
                }
                append_field(value, "signature", text)?
            }
            (Block::Tool { partial_json, .. }, "input_json_delta") => partial_json.push_str(text),
            _ => return Err(invalid()),
        }
        Ok(())
    }

    fn finish_block(&mut self, event: &Value) -> Result<(), String> {
        let index = index(event)?;
        let block = self.blocks.remove(&index).ok_or_else(invalid)?;
        let value = match block {
            Block::Thinking { value } | Block::Text { value } => value,
            Block::Tool {
                mut value,
                partial_json,
            } => {
                let input = if partial_json.is_empty() {
                    value
                        .get("input")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}))
                } else {
                    serde_json::from_str(&partial_json).map_err(|_| invalid())?
                };
                let id = bounded_string(&value, "id")?;
                let name = bounded_string(&value, "name")?;
                value["input"] = input.clone();
                self.tool_call_ids.push(id);
                self.tool_calls.push((name, input));
                value
            }
        };
        if self.completed_blocks.len() >= MAX_NATIVE_ITEMS {
            return Err(invalid());
        }
        self.completed_blocks.push(value);
        Ok(())
    }

    fn apply_message_delta(
        &mut self,
        event: &Value,
        context: UsageContext<'_>,
    ) -> Result<(), String> {
        if let Some(reason) = event.pointer("/delta/stop_reason").and_then(Value::as_str) {
            if !crate::services::provider_usage::valid_provider_metadata(reason) {
                return Err(invalid());
            }
            self.finish_reason = Some(reason.to_string());
        }
        self.apply_usage(&event["usage"], context)
    }

    fn apply_usage(&mut self, value: &Value, context: UsageContext<'_>) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }
        validate_usage_counts(value)?;
        let Some(update) = RequestUsage::from_json_with_context(value, context) else {
            return Ok(());
        };
        let current = self.usage.get_or_insert_with(RequestUsage::default);
        merge_usage(current, update);
        Ok(())
    }

    fn add_bytes(&mut self, bytes: usize) -> Result<(), String> {
        self.total_bytes = self.total_bytes.checked_add(bytes).ok_or_else(invalid)?;
        (self.total_bytes <= MAX_ENVELOPE_BYTES)
            .then_some(())
            .ok_or_else(invalid)
    }
}

#[derive(Debug)]
pub(super) struct ConsumedStream {
    pub content: String,
    pub continuation_blocks: Vec<Value>,
    pub tool_calls: Vec<(String, Value)>,
    pub tool_call_ids: Vec<String>,
    pub usage: Option<RequestUsage>,
    pub finish_reason: Option<String>,
}
