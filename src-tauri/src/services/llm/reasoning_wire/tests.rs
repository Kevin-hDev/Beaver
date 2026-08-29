use super::{ReasoningCapture, ReasoningCaptureContext};
use crate::services::reasoning_continuity::contract::{
    ContractId, CredentialScope, ReasoningModeId, RouteId,
};
use crate::services::reasoning_continuity::envelope::ContinuationState;
use crate::services::reasoning_continuity::limits::{
    MAX_ENVELOPE_BYTES, MAX_NATIVE_ITEMS, MAX_TOOL_CALLS,
};
use serde_json::{json, Value};

fn context(route_id: RouteId, model_id: &str) -> ReasoningCaptureContext {
    let credential_scope = if route_id == RouteId::Ollama {
        CredentialScope::local_uncredentialed()
    } else {
        CredentialScope::authenticated("fixture-scope").expect("fixture scope")
    };
    ReasoningCaptureContext {
        route_id,
        model_id: model_id.into(),
        credential_scope,
        reasoning_mode: ReasoningModeId::Medium,
    }
}

fn fixture(name: &str) -> Value {
    serde_json::from_str(match name {
        "chat" => include_str!("../../../../test-fixtures/reasoning/chat-reasoning-empty.json"),
        "zai" => include_str!("../../../../test-fixtures/reasoning/zai-fragments.json"),
        "gemini" => include_str!("../../../../test-fixtures/reasoning/gemini-signature-late.json"),
        "mistral" => include_str!("../../../../test-fixtures/reasoning/mistral-chunks.json"),
        "openrouter" => include_str!("../../../../test-fixtures/reasoning/openrouter-details.json"),
        "ollama" => include_str!("../../../../test-fixtures/reasoning/ollama-done.json"),
        "codex" => include_str!("../../../../test-fixtures/reasoning/codex-completed.json"),
        _ => unreachable!(),
    })
    .expect("fixture JSON")
}

#[test]
fn r02_r03_preserve_native_values_and_serialized_bytes() {
    let cases = [
        ("gemini", RouteId::Google, "gemini-3.7-flash"),
        ("mistral", RouteId::Mistral, "mistral-small-2603"),
        ("openrouter", RouteId::OpenRouter, "moonshotai/kimi-k2.5"),
        ("codex", RouteId::CodexOauth, "gpt-5.6-luna"),
    ];
    for (name, route, model) in cases {
        let mut capture = ReasoningCapture::new(context(route, model)).expect("capture");
        for event in fixture(name)["events"].as_array().expect("events") {
            capture.observe_json(event);
        }
        capture.observe_done(fixture(name)["events"].as_array().unwrap().last().unwrap());
        let envelope = capture.finish_complete().expect("complete envelope");
        let expected = fixture(name)["expected_native"].clone();
        let actual = match envelope.continuation {
            ContinuationState::GeminiParts { parts } => Value::Array(parts),
            ContinuationState::MistralChunks { chunks } => Value::Array(chunks),
            ContinuationState::OpenRouterDetails { details } => Value::Array(details),
            ContinuationState::ResponsesLocal { items } => Value::Array(items),
            _ => panic!("unexpected continuation"),
        };
        assert_eq!(actual, expected, "{name}");
        assert_eq!(
            serde_json::to_vec(&actual).unwrap(),
            serde_json::to_vec(&expected).unwrap()
        );
    }
}

#[test]
fn chat_and_ollama_complete_only_on_their_native_terminal_signal() {
    let mut chat = ReasoningCapture::new(context(RouteId::Moonshot, "kimi-k2.7-code")).unwrap();
    for event in fixture("chat")["events"].as_array().unwrap() {
        chat.observe_json(event);
        chat.observe_done(event);
    }
    assert!(chat.finish_complete().is_some());

    let mut ollama = ReasoningCapture::new(context(RouteId::Ollama, "qwen3.5:4b")).unwrap();
    for event in fixture("ollama")["events"].as_array().unwrap() {
        ollama.observe_json(event);
        ollama.observe_done(event);
    }
    assert!(ollama.finish_complete().is_some());
}

#[test]
fn chat_capture_accepts_the_native_done_marker_without_a_finish_reason() {
    let mut capture = ReasoningCapture::new(context(RouteId::Moonshot, "kimi-k2.7-code")).unwrap();
    capture.observe_json(&json!({
        "choices": [{"delta": {"reasoning_content": "opaque"}}]
    }));
    capture.observe_transport_complete();

    let envelope = capture.finish_complete().expect("complete envelope");
    assert_eq!(
        envelope.continuation,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque".into()
        }
    );
}

#[test]
fn anthropic_transport_end_without_message_stop_cannot_complete_capture() {
    let mut capture =
        ReasoningCapture::new(context(RouteId::Anthropic, "claude-haiku-4-5-20251001")).unwrap();
    capture.observe_anthropic_block(json!({
        "type": "thinking",
        "thinking": "opaque",
        "signature": "AAE+/=="
    }));
    capture.observe_transport_complete();

    assert!(capture.finish_complete().is_none());
}

#[test]
fn qwen_capture_concatenates_fragmented_reasoning_exactly_once() {
    let mut capture = ReasoningCapture::new(ReasoningCaptureContext {
        route_id: RouteId::Qwen,
        model_id: "qwen3.8-flash".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Xhigh,
    })
    .unwrap();
    capture.observe_json(&json!({
        "choices": [{"delta": {"reasoning_content": "opaque-"}}]
    }));
    capture.observe_json(&json!({
        "choices": [{"delta": {"reasoning_content": "qwen"}}]
    }));
    capture.observe_transport_complete();

    let envelope = capture.finish_complete().expect("complete envelope");
    assert_eq!(envelope.contract_id, ContractId::QwenChatV1);
    assert_eq!(
        envelope.continuation,
        ContinuationState::ChatReasoning {
            reasoning_content: "opaque-qwen".into()
        }
    );
}

#[test]
fn r07_first_limit_excess_releases_native_state_and_cannot_recover() {
    let mut capture =
        ReasoningCapture::new(context(RouteId::OpenRouter, "moonshotai/kimi-k2.5")).unwrap();
    for index in 0..MAX_NATIVE_ITEMS {
        capture
            .observe_json(&json!({"choices":[{"delta":{"reasoning_details":[{"index":index}]}}]}));
    }
    capture.observe_json(
        &json!({"choices":[{"delta":{"reasoning_details":[{"index":MAX_NATIVE_ITEMS}]}}]}),
    );
    for _ in 0..10 {
        capture
            .observe_json(&json!({"choices":[{"delta":{"reasoning_details":[{"ignored":true}]}}]}));
    }
    capture.observe_done(&json!({"choices":[{"finish_reason":"stop"}]}));
    assert!(capture.is_partial());
    assert_eq!(capture.failure_code(), Some("capture_limit_exceeded"));
    assert!(capture.finish_complete().is_none());

    let mut bytes =
        ReasoningCapture::new(context(RouteId::OpenRouter, "moonshotai/kimi-k2.5")).unwrap();
    bytes.observe_json(&json!({"choices":[{"delta":{"reasoning_details":[{"text":"x".repeat(MAX_ENVELOPE_BYTES)}]}}]}));
    bytes.observe_json(&json!({"choices":[{"delta":{"reasoning_details":[{"text":"x"}]}}]}));
    assert!(bytes.is_partial());

    let mut nested = Value::Null;
    for _ in 0..=32 {
        nested = Value::Array(vec![nested]);
    }
    let mut depth =
        ReasoningCapture::new(context(RouteId::OpenRouter, "moonshotai/kimi-k2.5")).unwrap();
    depth.observe_json(&json!({"choices":[{"delta":{"reasoning_details":[nested]}}]}));
    assert!(depth.is_partial());
}

#[test]
fn persisted_tool_link_limit_stops_capture_before_the_65th_link_is_stored() {
    let mut capture = ReasoningCapture::new(context(RouteId::OpenAi, "gpt-5.6-luna")).unwrap();
    let calls = (0..MAX_TOOL_CALLS)
        .map(|index| ("fixture.write_note".into(), json!({ "index": index })))
        .collect::<Vec<_>>();
    let ids = (0..MAX_TOOL_CALLS)
        .map(|index| format!("call-{index}"))
        .collect::<Vec<_>>();
    capture.observe_persisted_tool_links(&calls, &ids);
    assert_eq!(capture.response_tool_links.len(), MAX_TOOL_CALLS);
    capture.observe_persisted_tool_links(
        &[("fixture.read_note".into(), json!({}))],
        &["call-overflow".into()],
    );
    assert!(capture.is_partial());
    assert_eq!(capture.response_tool_links.len(), MAX_TOOL_CALLS);
    capture.observe_persisted_tool_links(
        &[("fixture.read_note".into(), json!({}))],
        &["call-after-stop".into()],
    );
    assert_eq!(capture.response_tool_links.len(), MAX_TOOL_CALLS);
}

#[test]
fn persisted_tool_links_use_canonical_names_without_changing_native_items() {
    let mut capture = ReasoningCapture::new(context(RouteId::OpenAi, "gpt-5.6-luna")).unwrap();
    capture.observe_json(&json!({
        "type": "response.output_item.done",
        "item": {"type":"function_call", "call_id":"call-1", "name":"fixture_write_note"}
    }));
    capture.observe_persisted_tool_links(
        &[("fixture.write_note".into(), json!({"value":"fixture"}))],
        &["call-1".into()],
    );
    capture.observe_done(&json!({"type":"response.completed"}));

    let envelope = capture.finish_complete().expect("complete envelope");
    assert_eq!(envelope.tool_links[0].provider_call_id, "call-1");
    assert_eq!(envelope.tool_links[0].tool_name, "fixture.write_note");
    let ContinuationState::ResponsesLocal { items } = envelope.continuation else {
        panic!("responses envelope");
    };
    assert_eq!(items[0]["name"], "fixture_write_note");
}

#[test]
fn anthropic_capture_keeps_completed_signed_blocks_and_canonical_tool_links() {
    let mut capture =
        ReasoningCapture::new(context(RouteId::Anthropic, "claude-haiku-4-5-20251001")).unwrap();
    let blocks = vec![
        json!({"type":"thinking","thinking":"opaque","signature":"AAE+/=="}),
        json!({"type":"tool_use","id":"toolu_1","name":"read_file","input":{"path":"README.md"}}),
    ];
    for block in &blocks {
        capture.observe_anthropic_block(block.clone());
    }
    capture.observe_persisted_tool_links(
        &[("read_file".into(), json!({"path":"README.md"}))],
        &["toolu_1".into()],
    );
    capture.observe_done(&json!({"type":"message_stop"}));

    let envelope = capture.finish_complete().unwrap();
    assert_eq!(envelope.tool_links[0].provider_call_id, "toolu_1");
    assert_eq!(
        envelope.continuation,
        ContinuationState::AnthropicBlocks { blocks }
    );
}
