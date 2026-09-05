use super::{tool_policy, ExtensionToolPolicy};
use serde_json::Value;

#[test]
fn openrouter_groq_models_keep_their_explicit_extension_closure() {
    assert_eq!(
        tool_policy("openrouter", "groq/llama-3.3-70b-versatile")
            .expect("OpenRouter policy")
            .extensions,
        ExtensionToolPolicy::WithoutExtensions,
    );
    assert_eq!(
        tool_policy("openrouter", "groq/compound")
            .expect("OpenRouter policy")
            .extensions,
        ExtensionToolPolicy::NoTools,
    );
    assert_eq!(
        tool_policy("openrouter", "openai/gpt-5.6")
            .expect("OpenRouter policy")
            .extensions,
        ExtensionToolPolicy::All,
    );
}

#[test]
fn openrouter_groq_payload_fixture_uses_the_production_filter() {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-fixtures/extensions/openrouter-groq-tool-payload.json"
    )))
    .expect("valid Groq payload fixture");
    let tools = fixture["tools"].as_array().expect("fixture tools").to_vec();

    for (model, expected_key) in [
        ("groq/llama-3.3-70b-versatile", "expectedWithoutExtensions"),
        ("groq/compound", "expectedNoTools"),
    ] {
        let policy = tool_policy("openrouter", model).expect("OpenRouter policy");
        let filtered =
            crate::commands::agent_chat_task::tool_policy::apply(policy.extensions, tools.clone());
        let names = filtered
            .tools
            .iter()
            .filter_map(|tool| tool.pointer("/function/name").and_then(Value::as_str))
            .collect::<Vec<_>>();
        let expected = fixture[expected_key]
            .as_array()
            .expect("expected tool names")
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        assert_eq!(names, expected, "unexpected tools for {model}");
    }
}
