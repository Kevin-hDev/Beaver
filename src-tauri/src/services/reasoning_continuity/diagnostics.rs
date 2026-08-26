use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningDecision {
    Captured,
    Persisted,
    Replayed,
    Blocked,
    Compacted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    Captured,
    FingerprintUnavailable,
    ProvenanceMismatch,
    Partial,
    Compacted,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SafeReasoningDiagnostic {
    pub decision: ReasoningDecision,
    pub code: ReasonCode,
    pub item_count: usize,
    pub byte_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hmac_prefix: Option<String>,
}

pub fn record(
    decision: ReasoningDecision,
    code: ReasonCode,
    item_count: usize,
    byte_count: usize,
    fingerprint: Result<String, super::fingerprint::FingerprintError>,
) -> SafeReasoningDiagnostic {
    SafeReasoningDiagnostic {
        decision,
        code: match fingerprint {
            Ok(_) => code,
            Err(_) => ReasonCode::FingerprintUnavailable,
        },
        item_count,
        byte_count,
        hmac_prefix: fingerprint
            .ok()
            .map(|value| value.chars().take(16).collect()),
    }
}
