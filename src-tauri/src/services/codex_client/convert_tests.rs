use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

#[test]
fn convert_extracts_system_as_instructions() {
    let msgs = vec![
        ChatMessage::system("Tu es un assistant.".into()),
        ChatMessage::user("Bonjour".into()),
    ];
    let (instructions, input) = convert_messages(&msgs);
    assert_eq!(instructions, "Tu es un assistant.");
    assert_eq!(input.len(), 1);
    assert_eq!(input[0]["role"], "user");
}

#[test]
fn vision_converts_user_images_to_responses_parts() {
    let msgs = vec![
        ChatMessage::user("Decris cette image".into()).with_images(vec!["iVBORw0KGgo=".into()])
    ];
    let (_, input) = convert_messages(&msgs);
    assert_eq!(input[0]["role"], "user");
    assert_eq!(input[0]["content"][0]["type"], "input_text");
    assert_eq!(input[0]["content"][0]["text"], "Decris cette image");
    assert_eq!(input[0]["content"][1]["type"], "input_image");
    assert_eq!(
        input[0]["content"][1]["image_url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn convert_splits_tool_calls_into_separate_items() {
    let msgs = vec![
        ChatMessage::assistant(
            "Je vais lire le fichier.".into(),
            None,
            None,
            None,
            Some(vec![ToolCallOllama {
                id: Some("call_1".into()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "read_file".into(),
                    arguments: serde_json::json!({"path": "/tmp/test.txt"}),
                },
            }]),
        ),
        ChatMessage::tool("contenu du fichier".into(), Some("call_1".into()), None),
    ];
    let (_, input) = convert_messages(&msgs);
    assert_eq!(input.len(), 3);
    assert_eq!(input[0]["role"], "assistant");
    assert_eq!(input[1]["type"], "function_call");
    assert_eq!(input[1]["name"], "read_file");
    assert_eq!(input[1]["call_id"], "call_1");
    assert_eq!(input[2]["type"], "function_call_output");
    assert_eq!(input[2]["call_id"], "call_1");
}

#[test]
fn codex_opaque_legacy_items_never_bypass_a_forbidden_target() {
    let reasoning = serde_json::json!({
        "type": "reasoning",
        "id": "rs_1",
        "encrypted_content": "opaque"
    });
    let function = serde_json::json!({
        "type": "function_call",
        "id": "fc_1",
        "call_id": "call_1",
        "name": "read_file",
        "arguments": "{\"path\":\"/tmp/test.txt\"}"
    });
    let msgs = vec![ChatMessage::assistant(
        "Texte déjà présent dans les items Codex.".into(),
        None,
        None,
        None,
        Some(vec![ToolCallOllama {
            id: Some("call_1".into()),
            extra_content: Some(serde_json::json!({
                "codex": { "output_items": [reasoning.clone(), function.clone()] }
            })),
            function: ToolCallFunction {
                name: "read_file".into(),
                arguments: serde_json::json!({"path": "/tmp/test.txt"}),
            },
        }]),
    )];

    let target = crate::services::reasoning_continuity::contract::ContinuationTarget::Forbidden(
        crate::services::reasoning_continuity::contract::NonReplayTarget {
            route_id: crate::services::reasoning_continuity::contract::RouteId::CodexOauth,
            model_id: "gpt-5.6-sol".into(),
            reasoning_mode: crate::services::reasoning_continuity::contract::ReasoningModeId::High,
        },
    );
    let (_, input) = convert_messages_with_tools_and_continuity(&msgs, &[], Some(&target)).unwrap();

    assert!(!input
        .iter()
        .any(|item| item == &reasoning || item == &function));
    assert_eq!(input[0]["role"], "assistant");
    assert_eq!(input[1]["type"], "function_call");
}

#[test]
fn codex_aliases_extension_tool_names_in_definitions_and_history() {
    let name = "beaver.office.documents.create";
    let tools = vec![serde_json::json!({
        "type": "function",
        "function": {
            "name": name,
            "description": "Create a document",
            "parameters": {"type": "object", "properties": {}}
        }
    })];
    let messages = vec![ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![ToolCallOllama {
            id: Some("call_1".into()),
            extra_content: None,
            function: ToolCallFunction {
                name: name.into(),
                arguments: serde_json::json!({}),
            },
        }]),
    )];

    let policy =
        crate::services::llm::route_profile::tool_policy("codex-oauth", "gpt-5.6-sol").unwrap();
    let converted_tools = convert_tools_to_responses_api(policy, &tools);
    let (_, input) = convert_messages(&messages);
    let wire_name = converted_tools[0]["name"].as_str().unwrap();

    assert_ne!(wire_name, name);
    assert_eq!(input[0]["name"], wire_name);
    assert_eq!(converted_tools[0]["strict"], false);
}
