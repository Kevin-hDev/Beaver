use super::*;

fn request(model: &str) -> ChatRequest {
    ChatRequest {
        model: model.into(),
        messages: Vec::new(),
        stream: true,
        tools: Some(Vec::new()),
        options: None,
        keep_alive: None,
        think: Some(OllamaThink::Level("high".into())),
        capture_reasoning: false,
        live_replay_target: None,
        fixture_candidate: None,
    }
}

#[test]
fn mandatory_cloud_thinking_refusal_never_retries_with_a_weaker_payload() {
    for model in ["glm-5.3-flash:cloud", "GLM-5.3-FLASH:cloud"] {
        for error in [
            "does not support thinking",
            "does not support thinking; does not support tools",
        ] {
            assert!(build_retry_request(&request(model), error).is_none());
        }
    }
}

#[test]
fn optional_thinking_and_unrelated_feature_retries_keep_their_behavior() {
    for model in ["gpt-oss:20b", "qwen3.5:4b", "unknown:latest"] {
        let retry = build_retry_request(&request(model), "does not support thinking").unwrap();
        assert_eq!(retry.think, Some(OllamaThink::Bool(false)));
        assert_eq!(retry.tools, Some(Vec::new()));
    }
    let original = request("glm-5.3-flash:cloud");
    let retry = build_retry_request(&original, "does not support tools").unwrap();
    assert_eq!(retry.think, original.think);
    assert!(retry.tools.is_none());
    assert!(build_retry_request(&original, "unrelated error").is_none());
}
