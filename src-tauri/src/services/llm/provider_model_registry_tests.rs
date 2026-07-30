use super::*;

fn source(provider_id: &'static str, json: &'static str) -> EmbeddedProviderModels {
    EmbeddedProviderModels { provider_id, json }
}

#[test]
fn every_supported_provider_has_one_valid_local_file() {
    let registry = parse_sources(SOURCES).expect("embedded provider registry");

    assert_eq!(
        registry.providers.len(),
        crate::services::llm::catalog::LLM_PROVIDERS.len()
    );
    for provider in crate::services::llm::catalog::LLM_PROVIDERS {
        assert!(registry.providers.contains_key(provider.id));
    }
}

#[test]
fn static_providers_keep_the_verified_order_and_limits() {
    let xai = list("xai");
    let zai = list("zai");

    assert_eq!(xai.first().unwrap().id, "grok-4.5");
    assert_eq!(xai.first().unwrap().context_window, 500_000);
    assert_eq!(xai.len(), 6);
    assert_eq!(zai.first().unwrap().id, "glm-5.2");
    assert_eq!(zai.first().unwrap().context_window, 1_000_000);
    assert_eq!(zai.len(), 19);
}

#[test]
fn aliases_resolve_without_duplicating_the_visible_inventory() {
    let alias = lookup("xai", "grok-4.5-latest").unwrap();

    assert_eq!(alias.id, "grok-4.5");
    assert_eq!(alias.context_window, 500_000);
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
