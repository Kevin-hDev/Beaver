use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use serde_json::{json, Value};

const CALL_ID: &str = "call_fixture_1";

fn messages() -> Vec<ChatMessage> {
    vec![
        ChatMessage::assistant(
            String::new(),
            None,
            None,
            None,
            Some(vec![ToolCallOllama {
                id: Some(CALL_ID.into()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "read_file".into(),
                    arguments: json!({"path": "README.md"}),
                },
            }]),
        ),
        ChatMessage::tool(
            "contenu".into(),
            Some(CALL_ID.into()),
            Some("read_file".into()),
        ),
    ]
}

fn anthropic_fixture(messages: &[ChatMessage], opaque: &Value) -> Vec<Value> {
    let call = &messages[0].tool_calls.as_ref().unwrap()[0];
    vec![
        json!({
            "role": "assistant",
            "content": [
                opaque,
                {
                    "type": "tool_use",
                    "id": call.id,
                    "name": call.function.name,
                    "input": call.function.arguments,
                }
            ]
        }),
        json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": messages[1].tool_call_id,
                "content": messages[1].content,
            }]
        }),
    ]
}

#[test]
fn wire_contract_places_tool_results_without_losing_ids_or_opaque_reasoning() {
    let messages = messages();
    let opaque = json!({"type": "thinking", "signature": "opaque-signature"});
    let envelope = ReasoningEnvelope::new(
        ContractId::OpenAiResponsesV1,
        ReasoningSource {
            route_id: RouteId::OpenAi,
            model_id: "fixture-model".into(),
            credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
            reasoning_mode: ReasoningModeId::High,
        },
        CompletionState::Complete,
        ContinuationState::ResponsesLocal {
            items: vec![opaque.clone()],
        },
        Vec::new(),
    );
    envelope.validate().unwrap();

    let anthropic = anthropic_fixture(&messages, &opaque);
    assert_eq!(anthropic[0]["content"][0], opaque);
    assert_eq!(anthropic[0]["content"][1]["id"], CALL_ID);
    assert_eq!(anthropic[1]["content"][0]["tool_use_id"], CALL_ID);

    let openai = super::stream_convert::messages_to_openai(&messages, "google");
    assert_eq!(openai[1]["role"], "tool");
    assert_eq!(openai[1]["tool_call_id"], CALL_ID);

    let (_, responses) = crate::services::codex_client::convert::convert_messages(&messages);
    let output = responses
        .iter()
        .find(|item| item["type"] == "function_call_output")
        .unwrap();
    assert_eq!(output["call_id"], CALL_ID);

    let ollama = crate::services::agent_local::ollama_tool_role::wrap_tool_results(&messages);
    assert_eq!(ollama[1].role, "user");
    assert!(ollama[1].content.contains("<tool_response>"));
}
