use serde_json::Value;

use super::context_usage_record::{
    ContextCountCoverage, ContextCountSource, ContextTokenCount,
};
use crate::services::token_counting;

pub fn chat_completions(payload: &Value) -> ContextTokenCount {
    count_fields(payload, &["messages", "tools"])
}

pub fn responses(payload: &Value) -> ContextTokenCount {
    count_fields(payload, &["instructions", "input", "tools"])
}

pub fn anthropic(payload: &Value) -> ContextTokenCount {
    count_fields(payload, &["system", "messages", "tools"])
}

pub fn ollama(payload: &Value) -> ContextTokenCount {
    count_fields(payload, &["messages", "tools"])
}

fn count_fields(payload: &Value, fields: &[&str]) -> ContextTokenCount {
    let mut count = SemanticCount::default();
    for field in fields {
        if let Some(value) = payload.get(field) {
            count.value(value, Some(field));
        }
    }
    count.finish()
}

#[derive(Default)]
struct SemanticCount {
    units: usize,
    capacity_units: usize,
    partial: bool,
    unbounded: bool,
}

impl SemanticCount {
    fn value(&mut self, value: &Value, key: Option<&str>) {
        if matches!(key, Some("encrypted_content" | "signature" | "thought_signature")) {
            self.partial = true;
            self.unbounded = true;
            self.text(value.as_str().unwrap_or_default());
            return;
        }
        if key == Some("image_url") {
            self.image_url(value);
            return;
        }
        if key == Some("images") {
            self.images(value);
            return;
        }
        match value {
            Value::Null => {}
            Value::Bool(value) => self.text(if *value { "true" } else { "false" }),
            Value::Number(value) => self.text(&value.to_string()),
            Value::String(value) => self.text(value),
            Value::Array(values) => {
                for value in values {
                    self.value(value, None);
                }
            }
            Value::Object(values) => {
                if values.get("type").and_then(Value::as_str) == Some("image") {
                    self.text("image");
                    let data = values
                        .get("source")
                        .and_then(|source| source.get("data"))
                        .and_then(Value::as_str);
                    self.image(data);
                    return;
                }
                for (key, value) in values {
                    self.text(key);
                    self.value(value, Some(key));
                }
            }
        }
    }

    fn text(&mut self, value: &str) {
        let units = token_counting::text_units(value);
        self.units = self.units.saturating_add(units);
        self.capacity_units = self.capacity_units.saturating_add(units);
    }

    fn image_url(&mut self, value: &Value) {
        let url = value
            .as_str()
            .or_else(|| value.get("url").and_then(Value::as_str));
        self.image(url);
    }

    fn images(&mut self, value: &Value) {
        let Some(images) = value.as_array() else {
            self.partial = true;
            self.unbounded = true;
            return;
        };
        for image in images {
            self.image(image.as_str());
        }
    }

    fn image(&mut self, value: Option<&str>) {
        self.partial = true;
        let image_units = token_counting::max_text_units(crate::services::llm::vision::IMAGE_TOKEN_ESTIMATE);
        self.units = self.units.saturating_add(image_units);
        match value {
            Some(value) if value.starts_with("data:") || !value.contains("://") => {
                self.capacity_units = self.capacity_units.saturating_add(
                    image_units.max(token_counting::text_units(value)),
                );
            }
            _ => self.unbounded = true,
        }
    }

    fn finish(self) -> ContextTokenCount {
        if self.unbounded {
            return ContextTokenCount {
                tokens: None,
                capacity_tokens: None,
                source: None,
                coverage: ContextCountCoverage::Unknown,
            };
        }
        let tokens = bounded(token_counting::token_count_from_units(self.units));
        ContextTokenCount {
            tokens: Some(tokens),
            capacity_tokens: Some(bounded(
                token_counting::token_count_from_units(self.capacity_units).max(tokens as usize),
            )),
            source: Some(ContextCountSource::Heuristic),
            coverage: if self.partial {
                ContextCountCoverage::Partial
            } else {
                ContextCountCoverage::Complete
            },
        }
    }
}

fn bounded(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}
