use super::*;
use serde_json::json;

#[test]
fn responses_stats_read_the_final_payload() {
    let payload = json!({
        "instructions": "règle",
        "tools": [{"type": "function"}, {"type": "web_search"}],
        "input": [
            {"type": "reasoning", "encrypted_content": "opaque"},
            {"role": "assistant", "content": [{"type": "output_text", "text": "réponse"}]},
            {"type": "function_call", "name": "grep", "arguments": "{}"},
            {"type": "function_call_output", "output": "résultat"}
        ]
    });

    let stats = payload_stats("responses", &payload);
    assert_eq!(stats.items, 4);
    assert_eq!(stats.assistant_items, 1);
    assert_eq!(stats.reasoning_fields, 1);
    assert!(stats.reasoning_chars > 0);
    assert_eq!(stats.assistant_content_chars, 7);
    assert_eq!(stats.tool_calls, 1);
    assert_eq!(stats.tool_results, 1);
    assert_eq!(stats.available_tools, 2);
    assert_eq!(stats.instructions_chars, 5);
}

#[test]
fn chat_stats_read_actual_policy_and_tools() {
    let payload = json!({
        "tools": [{"type": "function"}],
        "messages": [
            {"role": "system", "content": "système"},
            {"role": "assistant", "content": null, "reasoning_content": "opaque", "tool_calls": [{}]},
            {"role": "tool", "content": "résultat"},
            {"role": "user", "content": "suite"}
        ]
    });

    let stats = payload_stats("chat_completions", &payload);
    assert_eq!(stats.items, 4);
    assert_eq!(stats.assistant_items, 1);
    assert_eq!(stats.reasoning_fields, 1);
    assert_eq!(stats.reasoning_chars, 6);
    assert_eq!(stats.assistant_content_nulls, 1);
    assert_eq!(stats.tool_calls, 1);
    assert_eq!(stats.tool_results, 1);
    assert_eq!(stats.available_tools, 1);
    assert_eq!(stats.instructions_chars, 7);
}

#[test]
fn native_anthropic_and_ollama_shapes_are_counted() {
    let anthropic = json!({
        "system": [{"type": "text", "text": "règle"}],
        "messages": [
            {"role": "assistant", "content": [
                {"type": "thinking", "thinking": "opaque", "signature": "sig"},
                {"type": "text", "text": "answer"},
                {"type": "tool_use", "name": "grep"}
            ]},
            {"role": "user", "content": [{"type": "tool_result", "content": "ok"}]}
        ]
    });
    let ollama = json!({
        "messages": [{"role": "assistant", "content": "ok", "thinking": "réflexion"}]
    });

    let anthropic_stats = payload_stats("anthropic_messages", &anthropic);
    assert_eq!(anthropic_stats.items, 2);
    assert_eq!(anthropic_stats.assistant_items, 1);
    assert_eq!(anthropic_stats.reasoning_fields, 1);
    assert_eq!(anthropic_stats.assistant_content_chars, 6);
    assert_eq!(anthropic_stats.tool_calls, 1);
    assert_eq!(anthropic_stats.tool_results, 1);
    assert_eq!(anthropic_stats.instructions_chars, 5);

    let ollama_stats = payload_stats("ollama_chat", &ollama);
    assert_eq!(ollama_stats.reasoning_fields, 1);
    assert_eq!(ollama_stats.reasoning_chars, 9);
    assert_eq!(ollama_stats.assistant_content_chars, 2);
}
