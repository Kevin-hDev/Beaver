use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_NATIVE_ITEMS: usize = 128;
pub const MAX_ENVELOPE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TOOL_CALLS: usize = 64;
pub const MAX_SESSION_CONTINUITY_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_JSON_DEPTH: usize = 32;
pub const MAX_MODEL_ID_BYTES: usize = 128;
pub const MAX_CREDENTIAL_SCOPE_BYTES: usize = 128;
pub const MAX_PROVIDER_CALL_ID_BYTES: usize = 512;
pub const MAX_TOOL_NAME_BYTES: usize = 256;
pub const MAX_REMOTE_RESPONSE_ID_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitError {
    NativeItems,
    EnvelopeBytes,
    ToolCalls,
    SessionBytes,
    JsonDepth,
    ModelId,
    CredentialScope,
    ProviderCallId,
    ToolName,
    RemoteResponseId,
    SchemaVersion,
    ArithmeticOverflow,
}

impl std::fmt::Display for LimitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("reasoning_continuity_invalid")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CaptureBudget {
    item_count: usize,
    serialized_bytes: usize,
}

impl CaptureBudget {
    pub const fn new() -> Self {
        Self {
            item_count: 0,
            serialized_bytes: 0,
        }
    }

    pub fn observe_item(&mut self, item: &Value) -> Result<(), LimitError> {
        if json_depth(item, 0)? > MAX_JSON_DEPTH {
            return Err(LimitError::JsonDepth);
        }
        let next_items = self
            .item_count
            .checked_add(1)
            .ok_or(LimitError::ArithmeticOverflow)?;
        if next_items > MAX_NATIVE_ITEMS {
            return Err(LimitError::NativeItems);
        }
        let item_bytes = serde_json::to_vec(item)
            .map_err(|_| LimitError::EnvelopeBytes)?
            .len();
        let next_bytes = self
            .serialized_bytes
            .checked_add(item_bytes)
            .ok_or(LimitError::ArithmeticOverflow)?;
        if next_bytes > MAX_ENVELOPE_BYTES {
            return Err(LimitError::EnvelopeBytes);
        }
        self.item_count = next_items;
        self.serialized_bytes = next_bytes;
        Ok(())
    }

    pub fn observe_serialized_bytes(&mut self, additional: usize) -> Result<(), LimitError> {
        let next = checked_envelope_bytes(self.serialized_bytes, additional)?;
        self.serialized_bytes = next;
        Ok(())
    }

    pub const fn item_count(self) -> usize {
        self.item_count
    }

    pub const fn serialized_bytes(self) -> usize {
        self.serialized_bytes
    }
}

pub fn checked_envelope_bytes(current: usize, additional: usize) -> Result<usize, LimitError> {
    let next = current
        .checked_add(additional)
        .ok_or(LimitError::ArithmeticOverflow)?;
    (next <= MAX_ENVELOPE_BYTES)
        .then_some(next)
        .ok_or(LimitError::EnvelopeBytes)
}

pub fn checked_tool_calls(current: usize, additional: usize) -> Result<usize, LimitError> {
    let next = current
        .checked_add(additional)
        .ok_or(LimitError::ArithmeticOverflow)?;
    (next <= MAX_TOOL_CALLS)
        .then_some(next)
        .ok_or(LimitError::ToolCalls)
}

pub fn checked_session_continuity_bytes(
    current: usize,
    additional: usize,
) -> Result<usize, LimitError> {
    let next = current
        .checked_add(additional)
        .ok_or(LimitError::ArithmeticOverflow)?;
    (next <= MAX_SESSION_CONTINUITY_BYTES)
        .then_some(next)
        .ok_or(LimitError::SessionBytes)
}

pub fn validate_session_continuity_bytes(bytes: usize) -> Result<(), LimitError> {
    checked_session_continuity_bytes(0, bytes).map(|_| ())
}

pub fn validate_model_id(value: &str) -> Result<(), LimitError> {
    let valid = !value.is_empty()
        && value.len() <= MAX_MODEL_ID_BYTES
        && !value.contains("..")
        && !value.starts_with('/')
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':')
        });
    valid.then_some(()).ok_or(LimitError::ModelId)
}

pub fn validate_credential_scope(value: &str) -> Result<(), LimitError> {
    validate_bounded_identifier(value, MAX_CREDENTIAL_SCOPE_BYTES)
        .map_err(|_| LimitError::CredentialScope)
}

pub fn validate_provider_call_id(value: &str) -> Result<(), LimitError> {
    validate_bounded_identifier(value, MAX_PROVIDER_CALL_ID_BYTES)
        .map_err(|_| LimitError::ProviderCallId)
}

pub fn validate_tool_name(value: &str) -> Result<(), LimitError> {
    validate_bounded_identifier(value, MAX_TOOL_NAME_BYTES).map_err(|_| LimitError::ToolName)
}

pub fn validate_remote_response_id(value: &str) -> Result<(), LimitError> {
    validate_bounded_identifier(value, MAX_REMOTE_RESPONSE_ID_BYTES)
        .map_err(|_| LimitError::RemoteResponseId)
}

fn validate_bounded_identifier(value: &str, max: usize) -> Result<(), ()> {
    (!value.is_empty()
        && value.len() <= max
        && value
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace()))
    .then_some(())
    .ok_or(())
}

fn json_depth(value: &Value, depth: usize) -> Result<usize, LimitError> {
    if depth > MAX_JSON_DEPTH {
        return Err(LimitError::JsonDepth);
    }
    match value {
        Value::Array(items) => items.iter().try_fold(depth, |maximum, item| {
            Ok(maximum.max(json_depth(item, depth + 1)?))
        }),
        Value::Object(items) => items.values().try_fold(depth, |maximum, item| {
            Ok(maximum.max(json_depth(item, depth + 1)?))
        }),
        _ => Ok(depth),
    }
}
