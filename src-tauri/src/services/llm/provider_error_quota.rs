use serde::Serialize;
use serde_json::Value;

const MAX_DETAILS: usize = 16;
const MAX_VIOLATIONS: usize = 16;
const MAX_RETRY_SECONDS: u64 = 86_400;

/// Diagnostic categories only: never persist quota IDs, projects or messages.
/// These hints do not authorize retries or change the user-facing error.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct QuotaDetails {
    pub requests_per_day: bool,
    pub requests_per_minute: bool,
    pub input_per_day: bool,
    pub input_per_minute: bool,
    pub zero_limit: bool,
    pub retry_after_seconds: Option<u64>,
}

pub(super) fn extract(document: Option<&Value>) -> Option<QuotaDetails> {
    let details = document?.pointer("/error/details")?.as_array()?;
    let mut result = QuotaDetails::default();
    let mut found = false;
    for detail in details.iter().take(MAX_DETAILS) {
        match detail["@type"].as_str() {
            Some("type.googleapis.com/google.rpc.QuotaFailure") => {
                found = true;
                if let Some(violations) = detail["violations"].as_array() {
                    for violation in violations.iter().take(MAX_VIOLATIONS) {
                        let id = violation["quotaId"].as_str().unwrap_or_default();
                        result.requests_per_day |= id.starts_with("GenerateRequestsPerDay");
                        result.requests_per_minute |= id.starts_with("GenerateRequestsPerMinute");
                        result.input_per_day |=
                            id.starts_with("GenerateContentInputTokens") && id.contains("PerDay");
                        result.input_per_minute |= id.starts_with("GenerateContentInputTokens")
                            && id.contains("PerMinute");
                        result.zero_limit |= violation["quotaValue"].as_str() == Some("0")
                            || violation["quotaValue"].as_u64() == Some(0);
                    }
                }
            }
            Some("type.googleapis.com/google.rpc.RetryInfo") => {
                found = true;
                result.retry_after_seconds = detail["retryDelay"]
                    .as_str()
                    .and_then(|delay| delay.strip_suffix('s'))
                    .and_then(|seconds| seconds.parse::<u64>().ok())
                    .filter(|seconds| *seconds <= MAX_RETRY_SECONDS);
            }
            _ => {}
        }
    }
    found.then_some(result)
}
