use std::collections::HashSet;

#[tokio::test]
async fn public_snapshots_produce_one_resolved_verdict_per_text_model() {
    let _guard = super::runtime_models::test_mutation_lock().await;
    let text: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/openrouter/catalog-text-2026-09-09.json"
    ))
    .unwrap();
    let all: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/openrouter/catalog-all-2026-09-09.json"
    ))
    .unwrap();
    let text_models = super::openai_compat_parsing::parse_models_list(&text, "openrouter").unwrap();
    let all_text_models =
        super::openai_compat_parsing::parse_models_list(&all, "openrouter").unwrap();

    assert_eq!(text_models.len(), 431);
    assert_eq!(all_text_models.len(), 431);
    let text_ids = text_models
        .iter()
        .map(|model| model.id.as_str())
        .collect::<HashSet<_>>();
    let all_text_ids = all_text_models
        .iter()
        .map(|model| model.id.as_str())
        .collect::<HashSet<_>>();
    assert_eq!(text_ids, all_text_ids);
    assert_eq!(
        text_models
            .iter()
            .filter(|model| model.id.starts_with('~'))
            .count(),
        13
    );
    assert_eq!(
        text_models
            .iter()
            .filter(|model| model.id.ends_with(":batch"))
            .count(),
        72
    );
    assert_eq!(
        text_models
            .iter()
            .filter(|model| model.supported_parameters.as_deref() == Some(&[]))
            .count(),
        3
    );
    assert!(text_models.iter().all(|model| {
        model
            .supported_parameters
            .as_ref()
            .is_some_and(|parameters| {
                parameters.len() <= 64
                    && parameters
                        .iter()
                        .all(|parameter| !parameter.is_empty() && parameter.len() <= 64)
            })
    }));

    let enriched = super::model_catalog::enrich_models("openrouter", text_models.clone(), false)
        .await
        .unwrap();
    assert_eq!(enriched.len(), 359);
    let mut verdict_ids = HashSet::new();
    for row in all["data"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert!(
            verdict_ids.insert(id),
            "each captured ID needs exactly one verdict"
        );
        if !text_ids.contains(id) {
            println!("id={id} treatment=excluded_non_text_output");
            continue;
        }
        let resolved = super::provider_model_lookup::resolve_local("openrouter", id);
        if !super::openrouter_model_metadata::supports_synchronous_chat(id) {
            assert!(resolved.is_none(), "batch ID published to synchronous chat");
            println!("id={id} treatment=excluded_documented_batch_api");
            continue;
        }
        let resolved = resolved.expect("synchronous text ID did not resolve");
        let model = super::runtime_models::lookup("openrouter", id).unwrap();
        assert_eq!(resolved.supports_tools, model.supports_tools);
        assert_eq!(resolved.supports_vision, model.supports_vision);
        assert_eq!(resolved.reasoning_contract, model.reasoning_contract);
        let control = model
            .reasoning_contract
            .as_ref()
            .map(|contract| format!("{:?}", contract.control))
            .unwrap_or_else(|| "unknown".to_string());
        println!(
            "id={} tools={} vision={} reasoning={} default={} treatment=accepted_text_catalog",
            model.id,
            model.supports_tools,
            model.supports_vision,
            control,
            model.default_reasoning_mode.as_deref().unwrap_or("none"),
        );
    }
    assert_eq!(verdict_ids.len(), 582);
}

#[test]
fn provenance_matches_the_checked_in_public_bytes() {
    use sha2::{Digest, Sha256};

    let text = include_bytes!("../../../tests/fixtures/openrouter/catalog-text-2026-09-09.json");
    let all = include_bytes!("../../../tests/fixtures/openrouter/catalog-all-2026-09-09.json");
    let provenance: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/openrouter/provenance.json"
    ))
    .unwrap();

    assert_eq!(text.len(), 710_140);
    assert_eq!(all.len(), 879_966);
    assert_eq!(
        hex::encode(Sha256::digest(text)),
        "8f2a7cec33b15322dc9a27089bd7c4494878badf50cd111b2a775d552cf89ff7"
    );
    assert_eq!(
        hex::encode(Sha256::digest(all)),
        "1de9ab28dc03f3bd258df8917fb7db601e70be542fcdcf4e38cfebf7e5926fca"
    );
    assert_eq!(
        provenance["captures"][0]["sha256"],
        hex::encode(Sha256::digest(text))
    );
    assert_eq!(
        provenance["captures"][1]["sha256"],
        hex::encode(Sha256::digest(all))
    );
    assert_eq!(provenance["captures"][0]["models"], 431);
    assert_eq!(provenance["captures"][1]["models"], 582);
}
