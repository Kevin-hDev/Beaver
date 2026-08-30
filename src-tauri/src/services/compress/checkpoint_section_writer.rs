use serde::Serialize;

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

    pub fn push_independent<T: Serialize>(
        &mut self,
        name: &str,
        value: &T,
        limit: u32,
    ) -> Result<(), CompressionError> {
        push_bounded_json(&mut self.sections, name, value, limit).map(|_| ())
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
    let serialized =
        serde_json::to_string(value).map_err(|_| CompressionError::CandidateInvalid)?;
    if matches!(serialized.as_str(), "[]" | "null" | "{}") {
        return Ok(0);
    }
    let content = if crate::services::token_counting::estimate_text_tokens(&serialized)
        > limit as usize
    {
        super::checkpoint_messages::bounded_excerpt(&serialized, limit, "\n[section truncated]", "")
    } else {
        serialized
    };
    let used = crate::services::token_counting::estimate_text_tokens(&content)
        .min(limit as usize)
        .min(u32::MAX as usize) as u32;
    output.push(CheckpointSection {
        name: name.to_string(),
        content,
    });
    Ok(used)
}
