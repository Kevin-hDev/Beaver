use super::*;

#[test]
fn settings_search_still_reads_the_embedded_litellm_catalog() {
    let catalog = super::super::litellm_catalog::parse_catalog(include_str!(
        "../../../resources/litellm-models.json"
    ));

    let results = search_in(&catalog, "gpt-4o", 100);

    assert!(results.iter().any(|model| model.key == "gpt-4o"));
    assert!(results
        .iter()
        .all(|model| model.provider == "openai" || model.key.contains("gpt-4o")));
}

#[test]
fn settings_search_remains_bounded() {
    let catalog = super::super::litellm_catalog::parse_catalog(include_str!(
        "../../../resources/litellm-models.json"
    ));

    assert_eq!(search_in(&catalog, "", 3).len(), 3);
}
