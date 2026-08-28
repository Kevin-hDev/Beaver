use std::path::PathBuf;

use serde_json::json;

use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

const V1_FIXTURE: &[u8] = include_bytes!("../../../test-fixtures/agent-session-v1-synthetic.json");

pub(super) fn fixture_session() -> super::types_session::AgentSession {
    super::session_migration::read(V1_FIXTURE, PathBuf::from("fixture.json"))
        .expect("fixture")
        .into_session()
}

pub(super) fn responses_envelope(completion: CompletionState) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::CodexResponsesV1,
        ReasoningSource {
            route_id: RouteId::CodexOauth,
            model_id: "gpt-5.6-luna".into(),
            credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
            reasoning_mode: ReasoningModeId::Medium,
        },
        completion,
        ContinuationState::ResponsesLocal {
            items: vec![json!({
                "type": "reasoning",
                "encrypted_content": "opaque-secret"
            })],
        },
        Vec::new(),
    )
}
