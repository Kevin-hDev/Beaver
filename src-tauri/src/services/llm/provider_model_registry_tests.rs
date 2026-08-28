use super::*;

fn source<'a>(provider_id: &'a str, json: &'a str) -> EmbeddedProviderModels<'a> {
    EmbeddedProviderModels { provider_id, json }
}

#[test]
fn embedded_registry_accepts_seventeen_valid_sources() {
    let provider_ids = (0..17)
        .map(|index| format!("provider-{}", char::from(b'a' + index)))
        .collect::<Vec<_>>();
    let json = provider_ids
        .iter()
        .map(|provider| {
            format!(
                r#"{{
                  "provider":"{provider}",
                  "schema_version":1,
                  "verified_at":"2026-08-28",
                  "source_urls":["https://example.com/models"],
                  "models":[{{
                    "id":"model",
                    "context_window":10,
                    "supports_tools":false,
                    "supports_vision":false,
                    "supports_thinking":false
                  }}]
                }}"#
            )
        })
        .collect::<Vec<_>>();
    let sources = provider_ids
        .iter()
        .zip(&json)
        .map(|(provider, json)| source(provider, json))
        .collect::<Vec<_>>();

    assert_eq!(parse_sources(&sources).unwrap().providers.len(), 17);
}

#[test]
fn every_supported_provider_has_one_valid_local_file() {
    let registry = parse_sources(SOURCES).expect("embedded provider registry");

    assert_eq!(
        registry.providers.len(),
        crate::services::llm::catalog::all().len()
    );
    for provider in crate::services::llm::catalog::all() {
        assert!(registry.providers.contains_key(provider.id));
    }
}

#[test]
fn static_providers_keep_the_verified_order_and_limits() {
    let xai = list("xai");
    let zai = list("zai");
    let google = list("google");

    assert_eq!(xai.first().unwrap().id, "grok-4.6");
    assert_eq!(xai.first().unwrap().context_window, 500_000);
    assert_eq!(xai.len(), 7);
    assert_eq!(zai.first().unwrap().id, "glm-5.3");
    assert_eq!(zai.first().unwrap().context_window, 1_000_000);
    assert_eq!(zai.len(), 20);
    assert_eq!(google.first().unwrap().id, "gemini-3.7-flash");
    assert_eq!(google.len(), 14);
}

#[test]
fn new_models_publish_their_official_reasoning_contracts() {
    let glm = lookup("zai", "glm-5.3").unwrap();
    let grok = lookup("xai", "grok-4.6").unwrap();
    let gemini = lookup("google", "gemini-3.7-flash").unwrap();

    assert_eq!(glm.reasoning_modes, ["low", "high", "max"]);
    assert_eq!(glm.default_reasoning_mode.as_deref(), Some("max"));
    assert_eq!(grok.reasoning_modes, ["low", "medium", "high", "xhigh"]);
    assert_eq!(grok.default_reasoning_mode.as_deref(), Some("high"));
    assert_eq!(gemini.reasoning_modes, ["low", "medium", "high"]);
    assert_eq!(gemini.default_reasoning_mode.as_deref(), Some("medium"));
    assert!(!glm.is_free);
    assert!(!grok.is_free);
    assert!(!gemini.is_free);
}

#[test]
fn aliases_resolve_without_duplicating_the_visible_inventory() {
    let alias = lookup("xai", "grok-4.5-latest").unwrap();
    let grok_46_alias = lookup("xai", "grok-4.6-latest").unwrap();

    assert_eq!(alias.id, "grok-4.5");
    assert_eq!(alias.context_window, 500_000);
    assert_eq!(grok_46_alias.id, "grok-4.6");
    assert_eq!(
        grok_46_alias.default_reasoning_mode.as_deref(),
        Some("high")
    );
    assert!(!list("xai")
        .iter()
        .any(|model| model.id == "grok-4.5-latest"));
}

#[test]
fn openrouter_declares_upstream_inheritance() {
    assert!(inherits_upstream("openrouter"));
    assert!(list("openrouter").is_empty());
}

#[test]
fn rejects_duplicate_models_and_impossible_limits() {
    let duplicate = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[
            {"id":"same","context_window":10,"max_output_tokens":5,"supports_tools":false,"supports_vision":false,"supports_thinking":false},
            {"id":"same","context_window":10,"max_output_tokens":5,"supports_tools":false,"supports_vision":false,"supports_thinking":false}
          ]
        }"#,
    );
    let impossible = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[
            {"id":"bad","context_window":10,"max_output_tokens":11,"supports_tools":false,"supports_vision":false,"supports_thinking":false}
          ]
        }"#,
    );

    assert_eq!(parse_sources(&[duplicate]).err(), Some("model_id"));
    assert_eq!(parse_sources(&[impossible]).err(), Some("output_limit"));
}

#[test]
fn accepts_an_official_maximum_equal_to_the_context() {
    let source = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"valid",
            "context_window":1048576,
            "max_output_tokens":1048576,
            "default_output_tokens":131072,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":false
          }]
        }"#,
    );

    assert!(parse_sources(&[source]).is_ok());
}

#[test]
fn rejects_invalid_automatic_output_defaults() {
    let above_maximum = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"bad",
            "context_window":100,
            "max_output_tokens":50,
            "default_output_tokens":51,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":false
          }]
        }"#,
    );
    let zero = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"bad",
            "context_window":100,
            "default_output_tokens":0,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":false
          }]
        }"#,
    );

    assert_eq!(
        parse_sources(&[above_maximum]).err(),
        Some("output_default")
    );
    assert_eq!(parse_sources(&[zero]).err(), Some("output_default"));
}

#[test]
fn rejects_duplicate_aliases() {
    let duplicate = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[
            {"id":"first","aliases":["shared"],"context_window":10,"supports_tools":false,"supports_vision":false,"supports_thinking":false},
            {"id":"second","aliases":["shared"],"context_window":10,"supports_tools":false,"supports_vision":false,"supports_thinking":false}
          ]
        }"#,
    );

    assert_eq!(parse_sources(&[duplicate]).err(), Some("model_id"));
}

#[test]
fn rejects_unbounded_aliases_and_missing_sources() {
    let aliases = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"model",
            "aliases":[
              "a00","a01","a02","a03","a04","a05","a06","a07","a08","a09","a10",
              "a11","a12","a13","a14","a15","a16","a17","a18","a19","a20","a21",
              "a22","a23","a24","a25","a26","a27","a28","a29","a30","a31","a32"
            ],
            "context_window":10,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":false
          }]
        }"#,
    );
    let no_sources = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":[],
          "models":[
            {"id":"model","context_window":10,"supports_tools":false,"supports_vision":false,"supports_thinking":false}
          ]
        }"#,
    );

    assert_eq!(parse_sources(&[aliases]).err(), Some("model_id"));
    assert_eq!(parse_sources(&[no_sources]).err(), Some("provenance"));
}

#[test]
fn rejects_invalid_reasoning_contracts() {
    let unknown = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-08-22",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"model",
            "context_window":10,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":true,
            "reasoning_modes":["turbo"]
          }]
        }"#,
    );
    let duplicate = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-08-22",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"model",
            "context_window":10,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":true,
            "reasoning_modes":["high","high"]
          }]
        }"#,
    );
    let invalid_default = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-08-22",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"model",
            "context_window":10,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":true,
            "reasoning_modes":["low","high"],
            "default_reasoning_mode":"medium"
          }]
        }"#,
    );
    assert_eq!(parse_sources(&[unknown]).err(), Some("reasoning_modes"));
    assert_eq!(parse_sources(&[duplicate]).err(), Some("reasoning_modes"));
    assert_eq!(
        parse_sources(&[invalid_default]).err(),
        Some("reasoning_default")
    );
}

#[test]
fn rejects_untrusted_provenance_and_provider_mismatch() {
    let mismatch = source(
        "expected",
        r#"{
          "provider":"other",
          "schema_version":1,
          "verified_at":"30/07/2026",
          "source_urls":["http://example.com/models"],
          "models":[]
        }"#,
    );
    let bad_provenance = source(
        "expected",
        r#"{
          "provider":"expected",
          "schema_version":1,
          "verified_at":"2026-19-42",
          "source_urls":["http://example.com/models"],
          "models":[]
        }"#,
    );
    let impossible_date = source(
        "expected",
        r#"{
          "provider":"expected",
          "schema_version":1,
          "verified_at":"2026-02-30",
          "source_urls":["https://example.com/models"],
          "models":[]
        }"#,
    );
    let empty = source(
        "expected",
        r#"{
          "provider":"expected",
          "schema_version":1,
          "verified_at":"2026-07-30",
          "source_urls":["https://example.com/models"],
          "models":[]
        }"#,
    );

    assert_eq!(parse_sources(&[mismatch]).err(), Some("provider_id"));
    assert_eq!(parse_sources(&[bad_provenance]).err(), Some("provenance"));
    assert_eq!(parse_sources(&[impossible_date]).err(), Some("provenance"));
    assert_eq!(parse_sources(&[empty]).err(), Some("model_count"));
}

#[test]
fn rejects_fast_mode_outside_openai() {
    let source = source(
        "test",
        r#"{
          "provider":"test",
          "schema_version":1,
          "verified_at":"2026-08-23",
          "source_urls":["https://example.com/models"],
          "models":[{
            "id":"model",
            "context_window":10,
            "supports_tools":false,
            "supports_vision":false,
            "supports_thinking":false,
            "supports_fast_mode":true
          }]
        }"#,
    );

    assert_eq!(parse_sources(&[source]).err(), Some("fast_mode_provider"));
}
