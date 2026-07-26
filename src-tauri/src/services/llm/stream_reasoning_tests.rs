use super::stream_reasoning;
use serde_json::json;

fn payload(provider: &str, model: &str, mode: Option<&str>) -> serde_json::Value {
    let mut payload = json!({});
    stream_reasoning::apply(&mut payload, provider, model, mode != Some("off"), mode);
    payload
}

#[test]
fn groq_payloads_match_model_family() {
    assert_eq!(
        payload("groq", "qwen/qwen3-32b", Some("off"))["reasoning_effort"],
        "none"
    );
    assert_eq!(
        payload("groq", "qwen/qwen3-32b", Some("auto"))["reasoning_effort"],
        "default"
    );
    assert_eq!(
        payload("groq", "openai/gpt-oss-20b", Some("high"))["reasoning_effort"],
        "high"
    );
}

#[test]
fn deepseek_payload_uses_thinking_and_effort() {
    let high = payload("deepseek", "deepseek-v4-pro", Some("high"));
    assert_eq!(high["thinking"], json!({ "type": "enabled" }));
    assert_eq!(high["reasoning_effort"], "high");

    let max = payload("deepseek", "deepseek-v4-pro", Some("xhigh"));
    assert_eq!(max["reasoning_effort"], "max");

    let off = payload("deepseek", "deepseek-v4-pro", Some("off"));
    assert_eq!(off["thinking"], json!({ "type": "disabled" }));
}

#[test]
fn moonshot_switchable_can_disable_thinking() {
    let off = payload("moonshot", "kimi-k2.5", Some("off"));
    assert_eq!(off["thinking"], json!({ "type": "disabled" }));

    let auto = payload("moonshot", "kimi-k2.5", Some("auto"));
    assert_eq!(auto["thinking"], json!({ "type": "enabled" }));

    let forced = payload("moonshot", "kimi-k2-thinking", Some("auto"));
    assert_eq!(forced["thinking"], json!({ "type": "enabled" }));

    let k27 = payload("moonshot", "kimi-k2.7-code", Some("auto"));
    assert_eq!(k27["thinking"], json!({ "type": "enabled" }));
}

#[test]
fn moonshot_k3_sends_mandatory_thinking_effort() {
    for effort in ["low", "high", "max"] {
        assert_eq!(
            payload("moonshot", "k3", Some(effort))["thinking"],
            json!({ "type": "enabled", "effort": effort })
        );
    }
    assert_eq!(
        payload("moonshot", "k3", Some("off"))["thinking"],
        json!({ "type": "enabled", "effort": "max" })
    );
}

#[test]
fn zai_glm_52_uses_reasoning_effort() {
    let high = payload("zai", "glm-5.2", Some("high"));
    assert_eq!(high["thinking"], json!({ "type": "enabled" }));
    assert_eq!(high["reasoning_effort"], "high");

    let off = payload("zai", "glm-5.2", Some("off"));
    assert_eq!(off["thinking"], json!({ "type": "disabled" }));
    assert_eq!(off["reasoning_effort"], "none");
}

#[test]
fn google_gemini_35_requests_thought_summaries() {
    let payload = payload("google", "gemini-3.5-flash", Some("low"));
    assert_eq!(
        payload["extra_body"]["google"]["thinking_config"],
        json!({ "include_thoughts": true, "thinking_level": "low" })
    );
    assert!(payload["extra_body"]["google"]
        .get("thought_tag_marker")
        .is_none());
}

#[test]
fn google_gemini_25_uses_thinking_budget() {
    let payload = payload("google", "gemini-2.5-flash", Some("high"));
    assert_eq!(
        payload["extra_body"]["google"]["thinking_config"],
        json!({ "include_thoughts": true, "thinking_budget": 24576 })
    );
}

#[test]
fn mistral_adjustable_uses_reasoning_effort() {
    assert_eq!(
        payload("mistral", "mistral-small-latest", Some("off"))["reasoning_effort"],
        "none"
    );
    assert_eq!(
        payload("mistral", "mistral-small-latest", Some("high"))["reasoning_effort"],
        "high"
    );
}

#[test]
fn openai_chat_completions_uses_top_level_reasoning_effort() {
    let payload = payload("openai", "gpt-5.6-sol", Some("max"));

    assert_eq!(payload["reasoning_effort"], "max");
    assert!(payload.get("reasoning").is_none());
}

#[test]
fn openai_with_think_disabled_only_sends_explicit_off() {
    let mut active_payload = json!({});
    stream_reasoning::apply(
        &mut active_payload,
        "openai",
        "gpt-5.6-sol",
        false,
        Some("medium"),
    );
    assert!(active_payload.get("reasoning_effort").is_none());

    let mut off_payload = json!({});
    stream_reasoning::apply(
        &mut off_payload,
        "openai",
        "gpt-5.6-sol",
        false,
        Some("off"),
    );
    assert_eq!(off_payload["reasoning_effort"], "none");
}

#[test]
fn openrouter_gpt_56_keeps_nested_reasoning_shape() {
    let payload = payload("openrouter", "openai/gpt-5.6-terra", Some("max"));

    assert_eq!(payload["reasoning"], json!({ "effort": "max" }));
    assert!(payload.get("reasoning_effort").is_none());
}

#[test]
fn xai_only_sends_configurable_effort_for_supported_models() {
    assert_eq!(
        payload("xai", "grok-4.5", Some("medium"))["reasoning_effort"],
        "medium"
    );
    assert_eq!(
        payload("xai", "grok-4.3", Some("off"))["reasoning_effort"],
        "none"
    );
    assert!(payload("xai", "grok-4.5", Some("off"))
        .get("reasoning_effort")
        .is_none());
    assert!(payload("xai", "grok-4.20-0309-reasoning", Some("auto"))
        .get("reasoning_effort")
        .is_none());
    assert!(payload("xai", "grok-build-0.1", Some("auto"))
        .get("reasoning_effort")
        .is_none());
}
