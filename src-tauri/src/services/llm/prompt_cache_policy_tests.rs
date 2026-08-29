use super::prompt_cache_policy::{apply_payload, include_usage, request_headers, routing_key};
use super::request_purpose::RequestPurpose;
use serde_json::json;

fn apply(provider: &str, model: &str, value: &mut serde_json::Value, session_id: Option<&str>) {
    let policy = super::route_profile::cache_policy(provider, model).unwrap();
    apply_payload(value, policy, session_id);
}

fn resolved_headers(
    provider: &str,
    model: &str,
    session_id: Option<&str>,
    purpose: RequestPurpose,
) -> reqwest::header::HeaderMap {
    let policy = super::route_profile::cache_policy(provider, model).unwrap();
    request_headers(policy, session_id, purpose).unwrap()
}

fn uses_usage(provider: &str) -> bool {
    include_usage(super::route_profile::cache_policy(provider, "model").unwrap())
}

#[test]
fn anthropic_cache_marker_does_not_depend_on_a_session() {
    let mut value = payload();
    apply("anthropic", "claude-haiku-4-5-20251001", &mut value, None);

    assert_eq!(value["cache_control"]["type"], "ephemeral");
    assert!(value.get("session_id").is_none());
    assert!(value.get("prompt_cache_key").is_none());
}

#[test]
fn cache_transformer_receives_a_resolved_policy() {
    let policy = super::route_profile::cache_policy("openrouter", "google/gemini-3.5-pro")
        .expect("known route");
    let mut value = payload();

    apply_payload(&mut value, policy, Some("session-1"));

    assert!(value["session_id"].as_str().is_some());
    assert_eq!(policy.route_id, "openrouter");
}

fn payload() -> serde_json::Value {
    json!({
        "messages": [
            { "role": "system", "content": "stable" },
            { "role": "user", "content": "variable" }
        ]
    })
}

fn long_stable_payload() -> serde_json::Value {
    json!({
        "messages": [
            { "role": "system", "content": "abcd".repeat(1_280) },
            { "role": "user", "content": "variable" }
        ]
    })
}

#[test]
fn openai_gpt_56_gets_only_its_explicit_contract() {
    let mut value = long_stable_payload();
    apply("openai", "gpt-5.6-sol", &mut value, Some("session-1"));

    assert!(value["prompt_cache_key"].as_str().is_some());
    assert_eq!(value["prompt_cache_options"]["mode"], "explicit");
    assert_eq!(value["prompt_cache_options"]["ttl"], "30m");
    assert_eq!(
        value["messages"][0]["content"][0]["prompt_cache_breakpoint"]["mode"],
        "explicit"
    );
    assert!(value.get("session_id").is_none());
}

#[test]
fn short_openai_prefix_keeps_implicit_cache_enabled() {
    let mut value = payload();
    apply("openai", "gpt-5.6-sol", &mut value, Some("session-1"));

    assert!(value["prompt_cache_key"].as_str().is_some());
    assert!(value.get("prompt_cache_options").is_none());
    assert!(value["messages"][0]["content"].is_string());
}

#[test]
fn explicit_breakpoint_covers_the_entire_system_prompt() {
    let content = format!(
        "{}\n\n## Available skills\n- audit\n\nRespond in French.",
        "abcd".repeat(1_280)
    );
    let mut value = json!({
        "messages": [
            { "role": "system", "content": content },
            { "role": "user", "content": "variable" }
        ]
    });

    apply("openai", "gpt-5.6-sol", &mut value, Some("session-1"));

    let blocks = value["messages"][0]["content"].as_array().unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["text"], content);
    assert_eq!(blocks[0]["prompt_cache_breakpoint"]["mode"], "explicit");
}

#[test]
fn the_entire_system_prompt_counts_toward_the_explicit_cache_threshold() {
    let content = format!("short instructions\n\n{}", "stable context ".repeat(2_000));
    let mut value = json!({
        "messages": [
            { "role": "system", "content": content },
            { "role": "user", "content": "variable" }
        ]
    });

    apply("openai", "gpt-5.6-sol", &mut value, Some("session-1"));

    assert_eq!(value["prompt_cache_options"]["mode"], "explicit");
    let blocks = value["messages"][0]["content"].as_array().unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0]["text"], content);
}

#[test]
fn older_openai_models_never_receive_gpt_56_fields() {
    let mut value = payload();
    apply("openai", "gpt-4o", &mut value, Some("session-1"));

    assert!(value.get("prompt_cache_key").is_none());
    assert!(value.get("prompt_cache_options").is_none());
    assert!(value["messages"][0]["content"].is_string());
}

#[test]
fn openrouter_keeps_affinity_without_optional_explicit_cache_fields() {
    let mut compatible = payload();
    apply(
        "openrouter",
        "openai/gpt-5.6-luna",
        &mut compatible,
        Some("session-1"),
    );
    assert!(compatible["session_id"].as_str().is_some());
    assert!(compatible.get("prompt_cache_key").is_none());
    assert!(compatible.get("prompt_cache_options").is_none());
    assert!(compatible["messages"][0]["content"].is_string());

    let mut other = payload();
    apply(
        "openrouter",
        "google/gemini-3.5-pro",
        &mut other,
        Some("session-1"),
    );
    assert!(other["session_id"].as_str().is_some());
    assert!(other.get("prompt_cache_key").is_none());

    let headers = resolved_headers(
        "openrouter",
        "openai/gpt-5.6-luna",
        Some("session-1"),
        RequestPurpose::ManualChat,
    );
    assert_eq!(headers["x-openrouter-metadata"], "enabled");
    assert!(resolved_headers(
        "openrouter",
        "openai/gpt-5.6-luna",
        None,
        RequestPurpose::AccountMetadata,
    )
    .is_empty());
}

#[test]
fn route_keys_are_stable_but_isolated() {
    let mut first = payload();
    let mut second = payload();
    let mut other_route = payload();
    apply("mistral", "mistral-large", &mut first, Some("session-1"));
    apply("mistral", "mistral-large", &mut second, Some("session-1"));
    apply(
        "moonshot",
        "mistral-large",
        &mut other_route,
        Some("session-1"),
    );

    assert_eq!(first["prompt_cache_key"], second["prompt_cache_key"]);
    assert_ne!(first["prompt_cache_key"], other_route["prompt_cache_key"]);
    assert!(first["prompt_cache_key"]
        .as_str()
        .is_some_and(|value| value.len() == 36));
}

#[test]
fn moonshot_api_and_kimi_oauth_never_share_a_routing_key() {
    let mut api_payload = payload();
    let mut oauth_payload = payload();

    apply("moonshot", "kimi-k2.7", &mut api_payload, Some("session-1"));
    apply(
        "moonshot-oauth",
        "kimi-k2.7",
        &mut oauth_payload,
        Some("session-1"),
    );

    assert_ne!(
        api_payload["prompt_cache_key"],
        oauth_payload["prompt_cache_key"]
    );
}

#[test]
fn codex_special_route_gets_an_isolated_routing_key() {
    let codex = routing_key("codex-oauth", "gpt-5.6-sol", Some("session-1")).unwrap();
    let openai = routing_key("openai", "gpt-5.6-sol", Some("session-1")).unwrap();

    assert!(codex.starts_with("bv1_"));
    assert_ne!(codex, openai);
}

#[test]
fn automatic_providers_receive_no_cache_controls() {
    for provider in ["google", "cerebras", "deepseek", "zai"] {
        let mut value = payload();
        apply(provider, "model", &mut value, Some("session-1"));

        assert!(value.get("prompt_cache_key").is_none(), "{provider}");
        assert!(value.get("prompt_cache_options").is_none(), "{provider}");
        assert!(value.get("session_id").is_none(), "{provider}");
    }
}

#[test]
fn xai_header_is_api_key_only_and_never_leaks_the_session_id() {
    let headers = resolved_headers(
        "xai",
        "grok-4.5",
        Some("private-session"),
        RequestPurpose::ManualChat,
    );
    let value = headers["x-grok-conv-id"].to_str().unwrap();
    assert!(value.starts_with("bv1_"));
    assert!(!value.contains("private-session"));

    assert!(resolved_headers(
        "xai-oauth",
        "grok-4.5",
        Some("private-session"),
        RequestPurpose::ManualChat,
    )
    .is_empty());
}

#[test]
fn google_requests_identify_beaver_without_a_session_identifier() {
    let headers = resolved_headers(
        "google",
        "gemini-3.5-flash",
        Some("private-session"),
        RequestPurpose::ManualChat,
    );

    assert_eq!(
        headers["x-goog-api-client"].to_str().unwrap(),
        concat!("beaver-desktop/", env!("CARGO_PKG_VERSION")),
    );
    assert!(headers.get("x-grok-conv-id").is_none());
}

#[test]
fn usage_option_is_omitted_for_strict_or_self_reporting_routes() {
    for provider in ["mistral", "openrouter", "zai"] {
        assert!(!uses_usage(provider), "{provider}");
    }
    for provider in ["openai", "cerebras", "moonshot"] {
        assert!(uses_usage(provider), "{provider}");
    }
}
