use super::*;
use serde_json::json;

#[test]
fn chat_ignores_explicit_agentic_tools() {
    let definitions = definitions_for_mode(
        true,
        &[json!({"type": "function", "function": {"name": "bash"}})],
    );
    let names = tool_catalog::tool_names(&definitions);

    assert_eq!(names, vec!["web_search", "web_fetch"]);
}

#[test]
fn ollama_thinking_uses_only_rust_metadata() {
    let capabilities = ["completion".into(), "thinking".into()];
    let canonical = canonical_ollama_think("qwen3.5:4b", Some("auto"), true, Some(&capabilities));
    assert_eq!(
        canonical,
        Ok(crate::services::agent_local::types_ollama::OllamaThink::Bool(true))
    );
}

#[test]
fn installed_thinking_families_emit_concrete_ollama_payloads() {
    assert_eq!(
        canonical_ollama_think("gpt-oss:20b", None, true, Some(&["thinking".into()])),
        Ok(crate::services::agent_local::types_ollama::OllamaThink::Level("medium".into()))
    );
    for model in ["qwen3.5:4b", "deepseek-r1:latest", "gemma4:e2b-it-q4_K_M"] {
        assert_eq!(
            canonical_ollama_think(model, None, true, Some(&["thinking".into()])),
            Ok(crate::services::agent_local::types_ollama::OllamaThink::Bool(true)),
            "{model}"
        );
    }
    assert_eq!(
        canonical_ollama_think("llama3.2:latest", None, true, Some(&["completion".into()])),
        Ok(crate::services::agent_local::types_ollama::OllamaThink::Bool(false))
    );
    assert!(canonical_ollama_think("qwen3.5:4b", None, true, None).is_err());
}
