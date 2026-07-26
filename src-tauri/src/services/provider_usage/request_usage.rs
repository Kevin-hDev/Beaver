use serde::{Deserialize, Serialize};

const MAX_REQUEST_TOKENS: u64 = 10_000_000_000;
const MAX_COST_USD: f64 = 1_000_000.0;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub exact_cost_usd_micros: Option<u64>,
}

impl RequestUsage {
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        let input = count(
            value,
            &["prompt_tokens", "input_tokens", "promptTokenCount"],
        );
        let reasoning = nested_count(
            value,
            &[
                "/completion_tokens_details/reasoning_tokens",
                "/output_tokens_details/reasoning_tokens",
            ],
        )
        .or_else(|| count(value, &["thoughtsTokenCount"]));
        let output = count(value, &["completion_tokens", "output_tokens"]).or_else(|| {
            count(value, &["candidatesTokenCount"])
                .map(|tokens| tokens.saturating_add(reasoning.unwrap_or(0)))
        });
        let cached = nested_count(
            value,
            &[
                "/prompt_tokens_details/cached_tokens",
                "/input_tokens_details/cached_tokens",
            ],
        )
        .or_else(|| count(value, &["cachedContentTokenCount"]));
        let mut usage = Self {
            input_tokens: input,
            output_tokens: output,
            cached_input_tokens: cached,
            reasoning_output_tokens: reasoning,
            total_tokens: count(value, &["total_tokens", "totalTokenCount"]),
            exact_cost_usd_micros: parse_cost(value),
        };
        usage.normalize();
        (!usage.is_empty()).then_some(usage)
    }

    pub fn is_empty(&self) -> bool {
        self.input_tokens.is_none()
            && self.output_tokens.is_none()
            && self.cached_input_tokens.is_none()
            && self.reasoning_output_tokens.is_none()
            && self.total_tokens.is_none()
            && self.exact_cost_usd_micros.is_none()
    }

    fn normalize(&mut self) {
        if let (Some(cached), Some(input)) = (self.cached_input_tokens, self.input_tokens) {
            self.cached_input_tokens = Some(cached.min(input));
        }
        if let (Some(reasoning), Some(output)) = (self.reasoning_output_tokens, self.output_tokens)
        {
            self.reasoning_output_tokens = Some(reasoning.min(output));
        }
        if self.total_tokens.is_none() {
            self.total_tokens = match (self.input_tokens, self.output_tokens) {
                (Some(input), Some(output)) => Some(input.saturating_add(output)),
                _ => None,
            };
        }
    }
}

fn count(value: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(serde_json::Value::as_u64))
        .filter(|count| *count <= MAX_REQUEST_TOKENS)
}

fn nested_count(value: &serde_json::Value, paths: &[&str]) -> Option<u64> {
    paths
        .iter()
        .find_map(|path| value.pointer(path).and_then(serde_json::Value::as_u64))
        .filter(|count| *count <= MAX_REQUEST_TOKENS)
}

fn parse_cost(value: &serde_json::Value) -> Option<u64> {
    let raw = value.get("cost")?;
    let cost = raw.as_f64().or_else(|| {
        let text = raw.as_str().filter(|text| text.len() <= 32)?;
        text.parse::<f64>().ok()
    })?;
    if !cost.is_finite() || !(0.0..=MAX_COST_USD).contains(&cost) {
        return None;
    }
    Some((cost * 1_000_000.0).round() as u64)
}
