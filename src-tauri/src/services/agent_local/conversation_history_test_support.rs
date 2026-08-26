use chrono::Utc;

use super::super::conversation_input::ResolvedTurnInput;
use super::super::conversation_skills::ResolvedSkill;
use super::super::types_message::{
    AgentMessage, FileAttachment, ToolCallRequest, ToolCallRequestFunction,
};
use crate::services::reasoning_continuity::contract::{
    ContinuationUse, ContractId, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

pub const ERROR: &str = "conversation_admission_failed";

pub async fn create_session() -> super::super::types_session::AgentSession {
    let mut session = super::super::session_store::create_full(
        "History fixture",
        "model-a",
        "ollama",
        false,
        None,
    )
    .await
    .expect("create session");
    session.reasoning_mode = Some("auto".into());
    session.thinking_enabled = true;
    super::super::session_store::save(&session)
        .await
        .expect("persist target mode");
    session
}

pub async fn cleanup(id: &str) {
    super::super::session_store::delete_one(id)
        .await
        .expect("cleanup");
    super::super::session_store::remove_session_lock(id).await;
}

pub fn resolved(content: &str) -> ResolvedTurnInput {
    ResolvedTurnInput {
        user_content: content.into(),
        provider_content: format!("{content}\n\nresolved text"),
        files: vec![FileAttachment {
            name: "fixture.txt".into(),
            path: "/tmp/fixture.txt".into(),
            mime_type: "text/plain".into(),
            size: 8,
            thumbnail: None,
            access_grant: Some("fixture-grant".into()),
        }],
        images: Vec::new(),
        skills: vec![ResolvedSkill {
            id: "local:fixture".into(),
            name: "Skill local".into(),
            content: "instructions".into(),
        }],
    }
}

pub fn target(model: &str) -> ReplayTarget {
    ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: model.into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    }
}

pub fn complete_turn(
    suffix: &str,
    answer: &str,
    continuation: Option<ReasoningEnvelope>,
) -> Vec<AgentMessage> {
    let turn_id = format!("turn-{suffix}");
    vec![
        message(
            &format!("user-{suffix}"),
            &turn_id,
            "user",
            "question",
        ),
        AgentMessage {
            continuation,
            ..message(
                &format!("assistant-{suffix}"),
                &turn_id,
                "assistant",
                answer,
            )
        },
    ]
}

pub fn multi_tool_turn(turn_suffix: &str) -> Vec<AgentMessage> {
    let turn_id = format!("turn-{turn_suffix}");
    let mut assistant = message("assistant-calls", &turn_id, "assistant", "");
    assistant.tool_calls = Some(vec![
        tool_call("call-a", "read_file"),
        tool_call("call-b", "list_dir"),
    ]);
    vec![
        message("user-tools", &turn_id, "user", "inspect"),
        assistant,
        tool_result("result-b", &turn_id, "call-b", "list_dir", "b"),
        tool_result("result-a", &turn_id, "call-a", "read_file", "a"),
        message("assistant-final", &turn_id, "assistant", "done"),
    ]
}

fn tool_call(id: &str, name: &str) -> ToolCallRequest {
    ToolCallRequest {
        id: id.into(),
        extra_content: Some(serde_json::json!({"provider": id})),
        function: ToolCallRequestFunction {
            name: name.into(),
            arguments: serde_json::json!({"id": id}),
        },
    }
}

pub fn tool_result(
    id: &str,
    turn_id: &str,
    call_id: &str,
    name: &str,
    content: &str,
) -> AgentMessage {
    AgentMessage {
        tool_call_id: Some(call_id.into()),
        tool_name: Some(name.into()),
        ..message(id, turn_id, "tool", content)
    }
}

pub fn message(id: &str, turn_id: &str, role: &str, content: &str) -> AgentMessage {
    AgentMessage {
        id: id.into(),
        turn_id: turn_id.into(),
        role: role.into(),
        content: content.into(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        replay_source: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        skill_ids: None,
        stream_run_id: None,
        stream_part: None,
    }
}

pub fn envelope(route: RouteId, model: &str, thinking: &str) -> ReasoningEnvelope {
    ReasoningEnvelope::new(
        ContractId::OllamaNativeV1,
        ReasoningSource {
            route_id: route,
            model_id: model.into(),
            credential_scope: CredentialScope::local_uncredentialed(),
            reasoning_mode: ReasoningModeId::Auto,
        },
        CompletionState::Complete,
        ContinuationState::OllamaNative {
            thinking: thinking.into(),
        },
        Vec::new(),
    )
}

pub fn session_path(id: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{id}.json"))
}
