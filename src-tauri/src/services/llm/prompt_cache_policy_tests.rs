use super::prompt_cache_policy::{apply_payload, include_usage, request_headers, routing_key};
use super::request_purpose::RequestPurpose;
use super::route;
use serde_json::json;

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
    let route = route::resolve("openai").unwrap();
    let mut value = long_stable_payload();
    apply_payload(&mut value, &route, "gpt-5.6-sol", Some("session-1"));

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
    let route = route::resolve("openai").unwrap();
    let mut value = payload();
    apply_payload(&mut value, &route, "gpt-5.6-sol", Some("session-1"));

    assert!(value["prompt_cache_key"].as_str().is_some());
    assert!(value.get("prompt_cache_options").is_none());
    assert!(value["messages"][0]["content"].is_string());
}

#[test]
fn explicit_breakpoint_excludes_dynamic_system_sections() {
    let route = route::resolve("openai").unwrap();
    let stable = "abcd".repeat(1_280);
    let dynamic = format!(
        "{}Active providers: Brave.\n\n## Available skills\n- audit",
        crate::services::agent_local::web_search_status::SECTION_START
    );
    let mut value = json!({
        "messages": [
            { "role": "system", "content": format!("{stable}{dynamic}") },
            { "role": "user", "content": "variable" }
        ]
    });

    apply_payload(&mut value, &route, "gpt-5.6-sol", Some("session-1"));

    let blocks = value["messages"][0]["content"].as_array().unwrap();
    assert_eq!(blocks[0]["text"], stable);
    assert_eq!(blocks[0]["prompt_cache_breakpoint"]["mode"], "explicit");
    assert_eq!(blocks[1]["text"], dynamic);
    assert!(blocks[1].get("prompt_cache_breakpoint").is_none());
}

#[test]
fn large_dynamic_tail_does_not_make_a_short_prefix_explicit() {
    let route = route::resolve("openai").unwrap();
    let stable = "abcd".repeat(1_279);
    let content = format!(
        "{stable}{}{}",
        crate::services::agent_local::web_search_status::SECTION_START,
        "dynamic".repeat(2_000)
    );
    let mut value = json!({
        "messages": [
            { "role": "system", "content": content },
            { "role": "user", "content": "variable" }
        ]
    });

    apply_payload(&mut value, &route, "gpt-5.6-sol", Some("session-1"));

    assert!(value.get("prompt_cache_options").is_none());
    assert!(value["messages"][0]["content"].is_string());
}

#[test]
fn older_openai_models_never_receive_gpt_56_fields() {
    let route = route::resolve("openai").unwrap();
    let mut value = payload();
    apply_payload(&mut value, &route, "gpt-4o", Some("session-1"));

    assert!(value.get("prompt_cache_key").is_none());
    assert!(value.get("prompt_cache_options").is_none());
    assert!(value["messages"][0]["content"].is_string());
}

#[test]
fn openrouter_keeps_affinity_without_optional_explicit_cache_fields() {
    let route = route::resolve("openrouter").unwrap();
    let mut compatible = payload();
    apply_payload(
        &mut compatible,
        &route,
        "openai/gpt-5.6-luna",
        Some("session-1"),
    );
    assert!(compatible["session_id"].as_str().is_some());
    assert!(compatible.get("prompt_cache_key").is_none());
    assert!(compatible.get("prompt_cache_options").is_none());
    assert!(compatible["messages"][0]["content"].is_string());

    let mut other = payload();
    apply_payload(
        &mut other,
        &route,
        "google/gemini-3.5-pro",
        Some("session-1"),
    );
    assert!(other["session_id"].as_str().is_some());
    assert!(other.get("prompt_cache_key").is_none());

    let headers = request_headers(
        &route,
        Some("openai/gpt-5.6-luna"),
        Some("session-1"),
        RequestPurpose::ManualChat,
    )
    .unwrap();
    assert_eq!(headers["x-openrouter-metadata"], "enabled");
    assert!(request_headers(
        &route,
        Some("openai/gpt-5.6-luna"),
        None,
        RequestPurpose::AccountMetadata,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn route_keys_are_stable_but_isolated() {
    let mistral = route::resolve("mistral").unwrap();
    let moonshot = route::resolve("moonshot").unwrap();
    let mut first = payload();
    let mut second = payload();
    let mut other_route = payload();
    apply_payload(&mut first, &mistral, "mistral-large", Some("session-1"));
    apply_payload(&mut second, &mistral, "mistral-large", Some("session-1"));
    apply_payload(
        &mut other_route,
        &moonshot,
        "mistral-large",
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
    let api = route::resolve("moonshot").unwrap();
    let oauth = route::resolve("moonshot-oauth").unwrap();
    let mut api_payload = payload();
    let mut oauth_payload = payload();

    apply_payload(&mut api_payload, &api, "kimi-k2.7", Some("session-1"));
    apply_payload(&mut oauth_payload, &oauth, "kimi-k2.7", Some("session-1"));

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
    for provider in ["groq", "google", "cerebras", "deepseek", "zai"] {
        let route = route::resolve(provider).unwrap();
        let mut value = payload();
        apply_payload(&mut value, &route, "model", Some("session-1"));

        assert!(value.get("prompt_cache_key").is_none(), "{provider}");
        assert!(value.get("prompt_cache_options").is_none(), "{provider}");
        assert!(value.get("session_id").is_none(), "{provider}");
    }
}

#[test]
fn xai_header_is_api_key_only_and_never_leaks_the_session_id() {
    let route = route::resolve("xai").unwrap();
    let headers = request_headers(
        &route,
        Some("grok-4.5"),
        Some("private-session"),
        RequestPurpose::ManualChat,
    )
    .unwrap();
    let value = headers["x-grok-conv-id"].to_str().unwrap();
    assert!(value.starts_with("bv1_"));
    assert!(!value.contains("private-session"));

    let oauth = route::resolve("xai-oauth").unwrap();
    assert!(request_headers(
        &oauth,
        Some("grok-4.5"),
        Some("private-session"),
        RequestPurpose::ManualChat,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn google_requests_identify_beaver_without_a_session_identifier() {
    let route = route::resolve("google").unwrap();
    let headers = request_headers(
        &route,
        Some("gemini-3.5-flash"),
        Some("private-session"),
        RequestPurpose::ManualChat,
    )
    .unwrap();

    assert_eq!(
        headers["x-goog-api-client"].to_str().unwrap(),
        concat!("beaver-desktop/", env!("CARGO_PKG_VERSION")),
    );
    assert!(headers.get("x-grok-conv-id").is_none());
}

#[test]
fn usage_option_is_omitted_for_strict_or_self_reporting_routes() {
    for provider in ["mistral", "openrouter", "zai"] {
        assert!(
            !include_usage(&route::resolve(provider).unwrap()),
            "{provider}"
        );
    }
    for provider in ["openai", "groq", "cerebras", "moonshot"] {
        assert!(
            include_usage(&route::resolve(provider).unwrap()),
            "{provider}"
        );
    }
}
