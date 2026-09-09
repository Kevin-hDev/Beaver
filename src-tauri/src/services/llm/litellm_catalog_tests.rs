#[cfg(test)]
mod tests {
    use crate::services::llm::catalog_limits::MAX_LITELLM_CATALOG_ENTRIES;
    use crate::services::llm::litellm_catalog::{
        is_body_size_ok, is_trusted_host, parse_catalog, CatalogParseError, MAX_BODY_BYTES,
    };
    use crate::services::llm::litellm_catalog_refresh;

    fn fake_entry() -> String {
        r#"{"litellm_provider":"openai","mode":"chat"}"#.to_string()
    }

    fn build_json(count: usize) -> String {
        build_json_from_ids(0..count)
    }

    fn build_json_from_ids(ids: impl Iterator<Item = usize>) -> String {
        let entries: Vec<String> = ids
            .map(|i| format!(r#""model-{i}": {}"#, fake_entry()))
            .collect();
        format!("{{{}}}", entries.join(","))
    }

    #[test]
    fn complete_catalog_above_old_limit_is_retained() {
        let map = parse_catalog(&build_json(3_818)).expect("valid bounded catalog");

        assert_eq!(map.len(), 3_818);
        assert!(map.contains_key("model-3817"));
    }

    #[test]
    fn complete_catalog_is_independent_from_input_order() {
        let ascending = parse_catalog(&build_json_from_ids(0..3_818)).unwrap();
        let descending = parse_catalog(&build_json_from_ids((0..3_818).rev())).unwrap();

        assert_eq!(
            ascending.keys().collect::<std::collections::HashSet<_>>(),
            descending.keys().collect()
        );
    }

    #[test]
    fn parses_valid_json() {
        let json = build_json(10);
        let map = parse_catalog(&json).unwrap();
        assert_eq!(map.len(), 10);
    }

    #[test]
    fn token_counts_accept_integral_floats_without_saturation() {
        let parsed = parse_catalog(r#"{"model":{"max_input_tokens":2000000.0}}"#).unwrap();
        assert_eq!(parsed["model"].max_input_tokens, Some(2_000_000));
        for count in ["1.5", "-1.0", "18446744073709551616.0"] {
            let body = format!(r#"{{"model":{{"max_input_tokens":{count}}}}}"#);
            assert!(matches!(
                parse_catalog(&body),
                Err(CatalogParseError::InvalidEntry)
            ));
        }
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(matches!(
            parse_catalog("not json at all"),
            Err(CatalogParseError::InvalidJson)
        ));
    }

    #[test]
    fn rejects_malformed_entries() {
        let json = r#"{"good": {"litellm_provider":"x","mode":"chat"}, "bad": "not an object"}"#;
        assert!(matches!(
            parse_catalog(json),
            Err(CatalogParseError::InvalidEntry)
        ));
    }

    #[test]
    fn rejects_one_entry_above_the_limit() {
        let json = build_json(MAX_LITELLM_CATALOG_ENTRIES + 1);
        assert!(matches!(
            parse_catalog(&json),
            Err(CatalogParseError::TooManyEntries)
        ));
    }

    #[test]
    fn exact_limit_accepted() {
        let json = build_json(MAX_LITELLM_CATALOG_ENTRIES);
        let map = parse_catalog(&json).unwrap();
        assert_eq!(map.len(), MAX_LITELLM_CATALOG_ENTRIES);
    }

    #[test]
    fn under_limit_accepted() {
        let json = build_json(100);
        let map = parse_catalog(&json).unwrap();
        assert_eq!(map.len(), 100);
    }

    #[test]
    fn embedded_registry_contains_recent_provider_models() {
        let source = include_str!("../../../resources/litellm-models.json");
        let raw: serde_json::Map<String, serde_json::Value> = serde_json::from_str(source).unwrap();
        for (id, value) in raw.into_iter().filter(|(id, _)| id != "sample_spec") {
            assert!(
                serde_json::from_value::<super::super::ModelEntry>(value).is_ok(),
                "invalid embedded entry: {id}"
            );
        }
        let map = parse_catalog(source).unwrap();

        assert!(!map.contains_key("sample_spec"));

        let gemini = map.get("gemini/gemini-3.5-flash").unwrap();
        assert!(gemini.supports_function_calling);
        assert!(gemini.supports_reasoning);
        assert!(gemini.supports_vision);
        assert_eq!(gemini.max_input_tokens, Some(1_048_576));

        let glm = map.get("zai/glm-5.2").unwrap();
        assert!(glm.supports_function_calling);
        assert!(glm.supports_reasoning);
        assert_eq!(glm.max_input_tokens, Some(1_000_000));

        let kimi = map.get("moonshot/kimi-k2.7-code").unwrap();
        assert!(kimi.supports_function_calling);
        assert!(kimi.supports_reasoning);
        assert!(kimi.supports_vision);
        assert_eq!(kimi.max_input_tokens, Some(262_144));

        let grok = map.get("xai/grok-4-1-fast").unwrap();
        assert_eq!(grok.max_input_tokens, Some(2_000_000));
        assert_eq!(grok.max_output_tokens, Some(2_000_000));
    }

    #[test]
    fn empty_json_object() {
        let map = parse_catalog("{}").unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn rejects_duplicate_ids_before_the_second_value_is_published() {
        let duplicate = r#"{"same":{"mode":"chat"},"same":{"mode":"chat"}}"#;

        assert!(matches!(
            parse_catalog(duplicate),
            Err(CatalogParseError::DuplicateId)
        ));
    }

    #[test]
    fn technical_entry_counts_toward_the_input_limit() {
        let models = build_json(MAX_LITELLM_CATALOG_ENTRIES);
        let body = format!(r#"{{"sample_spec":{{}},{}"#, &models[1..]);

        assert!(matches!(
            parse_catalog(&body),
            Err(CatalogParseError::TooManyEntries)
        ));
    }

    #[test]
    fn technical_entry_can_follow_a_model_and_is_filtered_before_conversion() {
        let body = r#"{"model":{"mode":"chat"},"sample_spec":{"max_tokens":"description"}}"#;
        let map = parse_catalog(body).unwrap();

        assert_eq!(map.len(), 1);
        assert!(map.contains_key("model"));
        assert!(!map.contains_key("sample_spec"));
    }

    #[test]
    fn duplicate_technical_entry_is_rejected_at_any_position() {
        let body = r#"{"sample_spec":{},"model":{"mode":"chat"},"sample_spec":{}}"#;

        assert!(matches!(
            parse_catalog(body),
            Err(CatalogParseError::DuplicateId)
        ));
    }

    #[tokio::test]
    async fn invalid_refresh_keeps_the_previous_registry_and_cache() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = temporary.path().join("litellm-models.json");
        let registry = tokio::sync::RwLock::new(std::collections::HashMap::new());
        let healthy = build_json(100);
        litellm_catalog_refresh::publish_catalog(&cache, &registry, &healthy)
            .await
            .unwrap();

        let rejection = litellm_catalog_refresh::publish_catalog(&cache, &registry, "broken")
            .await
            .unwrap_err();
        assert_eq!(rejection.reason(), "invalid_json");
        assert_eq!(rejection.entries(), 0);
        assert_eq!(std::fs::read_to_string(&cache).unwrap(), healthy);
        let current = registry.read().await;
        assert_eq!(current.len(), 100);
        assert!(current.contains_key("model-99"));
    }

    #[test]
    fn trusted_host_github() {
        assert!(is_trusted_host("raw.githubusercontent.com"));
    }

    #[test]
    fn rejects_unknown_host() {
        assert!(!is_trusted_host("evil.com"));
        assert!(!is_trusted_host("raw.githubusercontent.com.evil.com"));
        assert!(!is_trusted_host(""));
    }

    #[test]
    fn body_size_within_limit() {
        assert!(is_body_size_ok(0));
        assert!(is_body_size_ok(1024));
        assert!(is_body_size_ok(MAX_BODY_BYTES));
    }

    #[test]
    fn body_size_over_limit() {
        assert!(!is_body_size_ok(MAX_BODY_BYTES + 1));
        assert!(!is_body_size_ok(usize::MAX));
    }
}
