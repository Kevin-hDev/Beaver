use super::*;
use crate::services::agent_local::types_ollama::{ToolCallFunction, ToolCallOllama};

fn message(role: &str, content: &str) -> ChatMessage {
    match role {
        "system" => ChatMessage::system(content.into()),
        "user" => ChatMessage::user(content.into()),
        "assistant" => ChatMessage::assistant(content.into(), None, None, None, None),
        "tool" => ChatMessage::tool(content.into(), None, None),
        other => panic!("unsupported chat role in test/setup: {other}"),
    }
}

fn total(usage: RequestContextUsage) -> u32 {
    usage
        .messages
        .saturating_add(usage.system_tools)
        .saturating_add(usage.mcp_connectors)
        .saturating_add(usage.skills)
        .saturating_add(usage.memory)
        .saturating_add(usage.meta_context)
        .saturating_add(usage.system_prompt)
}

#[test]
fn partitions_the_prepared_request_without_changing_its_total() {
    let mut assistant = message("assistant", &"a".repeat(101));
    assistant.tool_calls = Some(vec![ToolCallOllama {
        id: None,
        extra_content: None,
        function: ToolCallFunction {
            name: "bash".into(),
            arguments: serde_json::json!({ "command": "pwd" }),
        },
    }]);
    let messages = vec![message("system", &"s".repeat(400)), assistant];
    let tools = vec![serde_json::json!({
        "type": "function",
        "function": { "name": "search_mcp_tools", "description": "MCP" }
    })];
    let seed = ContextUsageSeed {
        meta_context_tokens: 20,
        skill_context_tokens: 10,
        memory_context_tokens: 5,
    };

    let usage = RequestContextUsage::from_request("ollama", &messages, &tools, seed);
    let expected =
        crate::services::compress::token_estimate::estimate_request_tokens(&messages, &tools)
            as u32;

    assert_eq!(total(usage), expected);
    assert_eq!(usage.skills, 10);
    assert_eq!(usage.memory, 5);
    assert_eq!(usage.meta_context, 20);
    assert_eq!(usage.system_prompt, 65);
    assert!(usage.system_tools > 0);
    assert!(usage.mcp_connectors > 0);
}

#[test]
fn codex_omits_reasoning_that_is_not_replayed() {
    let mut assistant = message("assistant", &"a".repeat(40));
    assistant.legacy_tool_loop_reasoning = Some("r".repeat(400));

    let codex = RequestContextUsage::from_request(
        crate::services::codex_client::PROVIDER_ID,
        std::slice::from_ref(&assistant),
        &[],
        ContextUsageSeed::default(),
    );
    let ollama =
        RequestContextUsage::from_request("ollama", &[assistant], &[], ContextUsageSeed::default());

    assert!(!codex.reasoning_included);
    assert!(ollama.reasoning_included);
    assert_eq!(codex.messages, 10);
    assert_eq!(ollama.messages, 110);
}

#[test]
fn loaded_skill_content_is_meta_context_not_skill_catalogue() {
    let mut loaded = message("tool", &"i".repeat(400));
    loaded.tool_name = Some("load_skill".into());
    let usage = RequestContextUsage::from_request(
        "ollama",
        &[message("system", &"s".repeat(80)), loaded],
        &[],
        ContextUsageSeed {
            skill_context_tokens: 5,
            ..Default::default()
        },
    );

    assert_eq!(usage.skills, 5);
    assert_eq!(usage.meta_context, 100);
}
