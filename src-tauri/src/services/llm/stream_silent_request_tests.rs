use super::request_config;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::{build_chat_payload_for_test, route};

fn config<'a>(
    provider: &'a str,
    model: &'a str,
    messages: &'a [ChatMessage],
) -> super::RequestConfig<'a> {
    request_config(
        provider,
        FastModeRequest::Fast,
        model,
        messages,
        Some(1_234),
        RequestPurpose::Automation,
        Some("internal-session"),
    )
}

#[test]
fn internal_new_direct_models_request_low_without_fast_or_continuation() {
    for (provider, model) in [
        ("google", "gemini-3.8-flash"),
        ("zai", "glm-5.3-flash"),
        ("openai", "gpt-6-astra"),
    ] {
        let messages = [ChatMessage::user("résume ceci".into())];
        let cfg = config(provider, model, &messages);
        assert!(cfg.think, "{provider}/{model}");
        assert_eq!(cfg.reasoning_mode, Some("low"), "{provider}/{model}");
        assert_eq!(cfg.max_tokens, Some(1_234), "{provider}/{model}");
        assert_ne!(cfg.fast_mode, FastModeRequest::Fast, "{provider}/{model}");
        if provider == "openai" {
            assert_eq!(
                cfg.fast_mode,
                FastModeRequest::Standard,
                "{provider}/{model}"
            );
        }
        assert!(cfg.continuation_target.is_none(), "{provider}/{model}");
    }
}

#[test]
fn internal_existing_models_keep_their_previous_request_selection() {
    let messages = [ChatMessage::user("résume ceci".into())];
    let cfg = config("mistral", "mistral-large-latest", &messages);
    assert!(!cfg.think);
    assert_eq!(cfg.reasoning_mode, None);
    assert_eq!(cfg.max_tokens, Some(1_234));
    assert_eq!(cfg.fast_mode, FastModeRequest::Fast);
    assert!(cfg.continuation_target.is_none());
}

#[test]
fn internal_astra_config_reaches_the_real_responses_constructor() {
    let messages = [ChatMessage::user("résume ceci".into())];
    let cfg = config("openai", "gpt-6-astra", &messages);
    let body = crate::services::llm::openai_responses::build_request(&cfg);
    assert_eq!(
        body["reasoning"],
        serde_json::json!({"effort":"low","summary":"auto"})
    );
    assert_eq!(body["max_output_tokens"], 1_234);
    assert_eq!(body["service_tier"], "default");
    assert_eq!(body["store"], false);
    assert!(body["reasoning"].get("context").is_none());
}

#[test]
fn internal_google_and_zai_payloads_use_the_real_chat_constructor() {
    let messages = [ChatMessage::user("résume ceci".into())];
    let google = config("google", "gemini-3.8-flash", &messages);
    let google_route = route::resolve("google").expect("Google route");
    let google_body = build_chat_payload_for_test(&google, &google_route, google.max_tokens)
        .expect("Google payload");
    assert_eq!(
        google_body["extra_body"]["google"]["thinking_config"],
        serde_json::json!({"include_thoughts": true, "thinking_level": "low"})
    );
    assert_eq!(google_body["max_tokens"], 1_234);

    let zai = config("zai", "glm-5.3-flash", &messages);
    let zai_route = route::resolve("zai").expect("Z.AI route");
    let zai_body =
        build_chat_payload_for_test(&zai, &zai_route, zai.max_tokens).expect("Z.AI payload");
    assert_eq!(zai_body["reasoning_effort"], "low");
    assert_eq!(zai_body["thinking"]["clear_thinking"], false);
    assert_eq!(zai_body["max_tokens"], 1_234);

    let messages = [ChatMessage::user("résume ceci".into())];
    let tools = [serde_json::json!({
        "type": "function",
        "function": {"name": "lookup", "parameters": {"type": "object"}}
    })];
    let mut zai_with_tool = super::request_config(
        "zai",
        FastModeRequest::Fast,
        "glm-5.3-flash",
        &messages,
        Some(1_234),
        RequestPurpose::Automation,
        Some("internal-session"),
    );
    zai_with_tool.tools = &tools;
    let zai_tool_body =
        build_chat_payload_for_test(&zai_with_tool, &zai_route, zai_with_tool.max_tokens)
            .expect("Z.AI tool payload");
    assert_eq!(zai_tool_body["tool_stream"], true);
}
