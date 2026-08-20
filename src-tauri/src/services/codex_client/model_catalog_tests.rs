use super::*;

fn parse(json: &str) -> Result<Vec<CatalogModel>, String> {
    let response: ModelsResponse = serde_json::from_str(json).map_err(|_| unavailable())?;
    parse_response(response)
}

#[test]
fn computes_the_effective_context_from_openai_metadata() {
    let models = parse(
        r#"{"models":[{"slug":"gpt-5.6-sol","display_name":"GPT-5.6 Sol","supported_reasoning_levels":[{"effort":"low"},{"effort":"max"}],"default_reasoning_level":"low","visibility":"list","context_window":272000,"max_context_window":272000,"effective_context_window_percent":95,"input_modalities":["text","image"]}]}"#,
    )
    .unwrap();

    assert_eq!(models[0].info.context_length, Some(258_400));
    assert_eq!(models[0].info.reasoning_modes, ["low", "max"]);
    assert_eq!(models[0].info.default_reasoning_mode, None);
    assert!(models[0].info.supports_vision);
    assert!(models[0].visible);
}

#[test]
fn rejects_invalid_context_and_deduplicates_models() {
    let models = parse(
        r#"{"models":[{"slug":"bad","display_name":"Bad","supported_reasoning_levels":[],"context_window":99999999},{"slug":"gpt-ok","display_name":"OK","supported_reasoning_levels":[{"effort":"low"}],"context_window":100000},{"slug":"gpt-ok","display_name":"Duplicate","supported_reasoning_levels":[{"effort":"high"}],"context_window":200000}]}"#,
    )
    .unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].info.id, "gpt-ok");
    assert_eq!(models[0].info.context_length, Some(95_000));
}

#[test]
fn external_collections_are_bounded() {
    let entries = (0..=super::super::model_catalog_wire::MAX_CATALOG_MODELS)
        .map(|index| {
            format!(r#"{{"slug":"m-{index}","display_name":"M","context_window":100000}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(r#"{{"models":[{entries}]}}"#);

    assert!(serde_json::from_str::<ModelsResponse>(&json).is_err());
}

#[test]
fn fallback_matches_the_current_conservative_codex_limit() {
    let models = fallback_models();
    let sol = models
        .iter()
        .find(|model| model.id == "gpt-5.6-sol")
        .unwrap();
    let luna = models
        .iter()
        .find(|model| model.id == "gpt-5.6-luna")
        .unwrap();

    assert_eq!(sol.context_length, Some(258_400));
    assert_eq!(
        sol.reasoning_modes,
        ["low", "medium", "high", "xhigh", "max", "ultra"]
    );
    assert_eq!(
        luna.reasoning_modes,
        ["low", "medium", "high", "xhigh", "max"]
    );
}
