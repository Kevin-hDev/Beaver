use serde::Serialize;
use serde_json::{Map, Value};

use super::checkpoint_document::CheckpointSection;
use super::checkpoint_transaction::CompressionError;

pub struct SectionWriter {
    pub sections: Vec<CheckpointSection>,
    pub remaining: u32,
}

impl SectionWriter {
    pub fn new(remaining: u32) -> Self {
        Self {
            sections: Vec::new(),
            remaining,
        }
    }

    pub fn push_evidence<T: Serialize>(
        &mut self,
        name: &str,
        value: &T,
        category_limit: u32,
    ) -> Result<(), CompressionError> {
        let limit = category_limit.min(self.remaining);
        let used = push_bounded_json(&mut self.sections, name, value, limit)?;
        self.remaining = self.remaining.saturating_sub(used);
        Ok(())
    }

    pub fn push_required<T: Serialize>(
        &mut self,
        name: &str,
        value: &T,
        category_limit: u32,
    ) -> Result<(), CompressionError> {
        let limit = category_limit.min(self.remaining);
        let serialized =
            serde_json::to_string(value).map_err(|_| CompressionError::CandidateInvalid)?;
        if matches!(serialized.as_str(), "[]" | "null" | "{}") {
            return Ok(());
        }
        let used = token_count(&serialized);
        if used > limit {
            return Err(CompressionError::CapacityExceeded);
        }
        self.sections.push(CheckpointSection {
            name: name.to_string(),
            content: serialized,
        });
        self.remaining = self.remaining.saturating_sub(used);
        Ok(())
    }
}

fn push_bounded_json<T: Serialize>(
    output: &mut Vec<CheckpointSection>,
    name: &str,
    value: &T,
    limit: u32,
) -> Result<u32, CompressionError> {
    if limit == 0 {
        return Ok(0);
    }
    let json = serde_json::to_value(value).map_err(|_| CompressionError::CandidateInvalid)?;
    let serialized =
        serde_json::to_string(&json).map_err(|_| CompressionError::CandidateInvalid)?;
    if matches!(serialized.as_str(), "[]" | "null" | "{}") {
        return Ok(0);
    }
    let content = if token_count(&serialized) > limit {
        let bounded = bound_json(json, limit);
        serde_json::to_string(&bounded).map_err(|_| CompressionError::CandidateInvalid)?
    } else {
        serialized
    };
    let used = token_count(&content);
    if used > limit {
        return Ok(0);
    }
    output.push(CheckpointSection {
        name: name.to_string(),
        content,
    });
    Ok(used)
}

fn bound_json(value: Value, limit: u32) -> Value {
    let marker = serde_json::json!({"truncated": true});
    match value {
        Value::Array(values) => {
            let mut output = Vec::new();
            let item_limit = (limit / values.len().max(1) as u32).max(32);
            for value in values {
                let item = bound_json(value, item_limit);
                let mut candidate = output.clone();
                candidate.push(item.clone());
                candidate.push(marker.clone());
                if serialized_tokens(&Value::Array(candidate)) > limit {
                    break;
                }
                output.push(item);
            }
            output.push(marker);
            Value::Array(output)
        }
        Value::Object(values) => bound_object(values, limit),
        Value::String(value) => Value::String(super::checkpoint_messages::bounded_excerpt(
            &value,
            limit.saturating_sub(8),
            " [truncated]",
            "",
        )),
        _ => marker,
    }
}

fn bound_object(values: Map<String, Value>, limit: u32) -> Value {
    let mut output = Map::new();
    let field_limit = (limit / values.len().max(1) as u32).max(32);
    for (key, value) in values {
        let item = bound_json(value, field_limit);
        output.insert(key.clone(), item);
        output.insert("_truncated".into(), Value::Bool(true));
        if serialized_tokens(&Value::Object(output.clone())) > limit {
            output.remove(&key);
        }
        output.remove("_truncated");
    }
    output.insert("_truncated".into(), Value::Bool(true));
    Value::Object(output)
}

fn serialized_tokens(value: &Value) -> u32 {
    serde_json::to_string(value)
        .map(|value| token_count(&value))
        .unwrap_or(u32::MAX)
}

fn token_count(value: &str) -> u32 {
    crate::services::token_counting::estimate_text_tokens(value).min(u32::MAX as usize) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_sections_remain_valid_json() {
        let mut writer = SectionWriter::new(64);
        writer
            .push_evidence("items", &vec!["x".repeat(2_000), "kept".into()], 64)
            .unwrap();

        let content = &writer.sections[0].content;
        assert!(serde_json::from_str::<Value>(content).is_ok());
        assert!(token_count(content) <= 64);
        assert!(content.contains("truncated"));
    }

    #[test]
    fn required_sections_fail_instead_of_losing_references() {
        let mut writer = SectionWriter::new(8);
        let result = writer.push_required("reports", &vec!["x".repeat(200)], 8);

        assert_eq!(result, Err(CompressionError::CapacityExceeded));
        assert!(writer.sections.is_empty());
    }
}
