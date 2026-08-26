use serde::Serialize;

use super::envelope::{ContinuationState, ReasoningEnvelope};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningDecision {
    Captured,
    Persisted,
    Replayed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    Captured,
    Persisted,
    Replayed,
    Blocked,
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
        code,
        item_count,
        byte_count,
        hmac_prefix: fingerprint
            .ok()
            .map(|value| value.chars().take(16).collect()),
    }
}

pub async fn record_envelope(
    session_id: &str,
    request_id: &str,
    turn_id: &str,
    decision: ReasoningDecision,
    envelope: &ReasoningEnvelope,
) {
    let opaque = match serde_json::to_vec(envelope) {
        Ok(value) => value,
        Err(_) => return,
    };
    let contract_id = match serde_json::to_value(envelope.contract_id)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
    {
        Some(value) => value,
        None => return,
    };
    let diagnostic = record(
        decision,
        code_for_decision(decision),
        item_count(&envelope.continuation),
        opaque.len(),
        super::fingerprint::opaque_hmac(
            super::fingerprint::FingerprintContext {
                session_id,
                turn_id,
                contract_id: &contract_id,
            },
            &opaque,
        ),
    );
    record_safe(session_id, request_id, diagnostic).await;
}

pub async fn record_blocked(session_id: &str, request_id: &str) {
    record_safe(
        session_id,
        request_id,
        SafeReasoningDiagnostic {
            decision: ReasoningDecision::Blocked,
            code: ReasonCode::Disabled,
            item_count: 0,
            byte_count: 0,
            hmac_prefix: None,
        },
    )
    .await;
}

async fn record_safe(session_id: &str, request_id: &str, diagnostic: SafeReasoningDiagnostic) {
    let message = format_safe_message(&diagnostic);
    crate::services::agent_local::stream_diagnostics::record_reasoning(
        session_id, request_id, &message,
    )
    .await;
}

fn format_safe_message(diagnostic: &SafeReasoningDiagnostic) -> String {
    let decision = serde_json::to_string(&diagnostic.decision).unwrap_or_default();
    let code = serde_json::to_string(&diagnostic.code).unwrap_or_default();
    let prefix = diagnostic.hmac_prefix.as_deref().unwrap_or("none");
    format!(
        "reasoning decision={decision} code={code} items={} bytes={} hmac={prefix}",
        diagnostic.item_count, diagnostic.byte_count
    )
}

const fn code_for_decision(decision: ReasoningDecision) -> ReasonCode {
    match decision {
        ReasoningDecision::Captured => ReasonCode::Captured,
        ReasoningDecision::Persisted => ReasonCode::Persisted,
        ReasoningDecision::Replayed => ReasonCode::Replayed,
        ReasoningDecision::Blocked => ReasonCode::Blocked,
    }
}

fn item_count(state: &ContinuationState) -> usize {
    match state {
        ContinuationState::GeminiParts { parts }
        | ContinuationState::MistralChunks { chunks: parts }
        | ContinuationState::OpenRouterDetails { details: parts }
        | ContinuationState::ResponsesLocal { items: parts } => parts.len(),
        ContinuationState::OllamaNative { thinking }
        | ContinuationState::ChatReasoning {
            reasoning_content: thinking,
        }
        | ContinuationState::CerebrasReasoning {
            reasoning: thinking,
        } => (!thinking.is_empty()) as usize,
        ContinuationState::RemoteContinuation { .. } => 1,
    }
}

#[cfg(test)]
mod safe_message_tests {
    use super::*;

    #[test]
    fn safe_message_never_contains_opaque_input() {
        let sentinel = "opaque-secret-sentinel";
        let diagnostic = record(
            ReasoningDecision::Replayed,
            ReasonCode::Replayed,
            2,
            sentinel.len(),
            Ok("0123456789abcdefremaining".to_string()),
        );
        let message = format_safe_message(&diagnostic);

        assert!(!message.contains(sentinel));
        assert!(message.contains("decision=\"replayed\""));
        assert!(message.contains("hmac=0123456789abcdef"));
    }
}
