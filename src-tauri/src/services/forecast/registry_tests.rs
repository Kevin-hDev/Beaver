use super::{catalog, registry};

#[test]
fn every_catalog_model_has_runtime_capabilities() {
    for model in catalog::FORECAST_MODELS {
        let runtime = registry::find_runtime(model.id)
            .unwrap_or_else(|| panic!("missing runtime for {}", model.id));
        assert_eq!(runtime.family_id, model.family_id);
        assert_eq!(
            runtime.capabilities.multivariate, model.multivariate,
            "{} exposes inconsistent multivariate metadata",
            model.id
        );
        assert!(
            !runtime.config_params.is_empty(),
            "{} has no config params",
            model.id
        );
    }
}

#[test]
fn runtime_capabilities_match_key_adapters() {
    let chronos_bolt = registry::find_runtime("chronos-bolt-tiny").unwrap();
    assert!(!chronos_bolt.capabilities.past_covariates);
    assert!(!chronos_bolt.capabilities.multi_series);
    assert!(!chronos_bolt.capabilities.multivariate);

    let chronos_2 = registry::find_runtime("chronos-2").unwrap();
    assert!(chronos_2.capabilities.past_covariates);
    assert!(chronos_2.capabilities.future_covariates);
    assert!(chronos_2.capabilities.multi_series);
    assert!(!chronos_2.capabilities.multivariate);

    let toto = registry::find_runtime("toto-2.0-2.5b").unwrap();
    assert!(!toto.capabilities.past_covariates);
    assert!(!toto.capabilities.future_covariates);
    assert!(toto.capabilities.multi_series);
    assert!(toto.capabilities.multivariate);
    assert!(toto.config_params.contains(&"decode_block_size"));

    let timesfm = registry::find_runtime("timesfm-2.5-200m").unwrap();
    assert!(timesfm.capabilities.past_covariates);
    assert!(timesfm.capabilities.future_covariates);
    assert!(timesfm.capabilities.multi_series);
    assert!(!timesfm.capabilities.multivariate);

    let timegpt_standard = registry::find_runtime("timegpt-2-standard").unwrap();
    assert!(timegpt_standard.capabilities.multi_series);
    assert!(!timegpt_standard.capabilities.multivariate);
    assert!(!timegpt_standard.capabilities.anomalies_ready);
    assert!(!timegpt_standard.capabilities.fine_tuning_ready);

    let timegpt_21 = registry::find_runtime("timegpt-2.1").unwrap();
    assert!(timegpt_21.capabilities.multi_series);
    assert!(timegpt_21.capabilities.multivariate);
}

#[test]
fn every_prediction_adapter_supports_rolling_backtests() {
    for runtime in registry::FORECAST_RUNTIMES {
        if registry::has_predict_adapter(runtime) {
            assert!(
                runtime.capabilities.backtesting_ready,
                "{}",
                runtime.model_id
            );
        }
    }
}

#[test]
fn legacy_tabpfn_id_resolves_without_duplicate_catalog_entry() {
    assert!(catalog::find_model("tabpfn-ts").is_some());
    assert!(!catalog::FORECAST_MODELS
        .iter()
        .any(|model| model.id == "tabpfn-ts"));
    assert!(catalog::FORECAST_MODELS
        .iter()
        .any(|model| model.id == "tabpfn-ts-3"));
}

#[test]
fn every_local_source_uses_an_immutable_revision() {
    for model in catalog::FORECAST_MODELS
        .iter()
        .filter(|model| !model.is_cloud)
    {
        if model.hf_repo.is_some() {
            assert!(is_sha(model.hf_revision), "{} HF", model.id);
        }
        if model.github_repo.is_some() {
            assert!(is_sha(model.github_revision), "{} GitHub", model.id);
        }
    }
}

fn is_sha(revision: Option<&str>) -> bool {
    revision.is_some_and(|value| {
        value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
    })
}
