use serde::Serialize;

pub const CODE: &str = "context_capacity_exceeded";
const SEPARATOR: char = ',';
const MAX_SAFE_TOKENS: u64 = 16_777_216;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextCapacityDetails {
    pub system_tokens: u64,
    pub required_report_tokens: u64,
    pub tool_tokens: u64,
    pub required_tokens: u64,
    pub max_input_tokens: u64,
    pub context_window: u64,
}

impl ContextCapacityDetails {
    pub fn from_counts(
        system_tokens: usize,
        required_report_tokens: usize,
        tool_tokens: usize,
        max_input_tokens: usize,
        context_window: u64,
    ) -> Self {
        let system_tokens = bounded(system_tokens as u64);
        let required_report_tokens = bounded(required_report_tokens as u64);
        let tool_tokens = bounded(tool_tokens as u64);
        Self {
            system_tokens,
            required_report_tokens,
            tool_tokens,
            required_tokens: system_tokens
                .saturating_add(required_report_tokens)
                .saturating_add(tool_tokens),
            max_input_tokens: bounded(max_input_tokens as u64),
            context_window: bounded(context_window),
        }
    }

    fn is_valid(self) -> bool {
        let values = [
            self.system_tokens,
            self.required_report_tokens,
            self.tool_tokens,
            self.required_tokens,
            self.max_input_tokens,
            self.context_window,
        ];
        values.iter().all(|value| *value <= MAX_SAFE_TOKENS)
            && (self.context_window == 0 || self.max_input_tokens <= self.context_window)
            && self.required_tokens
                == self
                    .system_tokens
                    .saturating_add(self.required_report_tokens)
                    .saturating_add(self.tool_tokens)
            && self.required_tokens > self.max_input_tokens
    }
}

pub fn encode(details: ContextCapacityDetails) -> String {
    format!(
        "{CODE}:{}{SEPARATOR}{}{SEPARATOR}{}{SEPARATOR}{}{SEPARATOR}{}{SEPARATOR}{}",
        details.system_tokens,
        details.required_report_tokens,
        details.tool_tokens,
        details.required_tokens,
        details.max_input_tokens,
        details.context_window,
    )
}

pub fn decode(error: &str) -> Option<ContextCapacityDetails> {
    let values = error.strip_prefix(&format!("{CODE}:"))?;
    let mut parts = values.split(SEPARATOR);
    let system_tokens = parts.next()?.parse().ok()?;
    let required_report_tokens = parts.next()?.parse().ok()?;
    let tool_tokens = parts.next()?.parse().ok()?;
    let required_tokens = parts.next()?.parse().ok()?;
    let max_input_tokens = parts.next()?.parse().ok()?;
    let context_window = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let details = ContextCapacityDetails {
        system_tokens,
        required_report_tokens,
        tool_tokens,
        required_tokens,
        max_input_tokens,
        context_window,
    };
    details.is_valid().then_some(details)
}

pub fn public_error(error: &str) -> (String, Option<ContextCapacityDetails>) {
    let details = decode(error);
    let is_capacity_error = error == CODE || error.starts_with(&format!("{CODE}:"));
    let message = if is_capacity_error { CODE } else { error }.to_string();
    (message, details)
}

fn bounded(value: u64) -> u64 {
    value.min(MAX_SAFE_TOKENS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_capacity_encoding_round_trips_safe_numbers() {
        let details = ContextCapacityDetails::from_counts(8_000, 500, 5_000, 12_000, 16_000);
        assert_eq!(decode(&encode(details)), Some(details));
    }

    #[test]
    fn context_capacity_decoder_rejects_inconsistent_numbers() {
        assert!(decode("context_capacity_exceeded:8000,0,5000,1,12000,16000").is_none());
        assert!(decode("context_capacity_exceeded:not-a-number").is_none());
    }

    #[test]
    fn public_error_hides_malformed_capacity_counters() {
        let (message, details) = public_error(
            "context_capacity_exceeded:8000,0,5000,13000,12000,invalid",
        );

        assert_eq!(message, CODE);
        assert_eq!(details, None);
    }

    #[test]
    fn public_error_preserves_unrelated_errors() {
        let (message, details) = public_error("ollama-connection-error");

        assert_eq!(message, "ollama-connection-error");
        assert_eq!(details, None);
    }
}
