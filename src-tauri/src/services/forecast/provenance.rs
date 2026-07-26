use std::collections::BTreeSet;

use super::data_quality::DataProfile;
use super::provenance_types::{
    ForecastDependencyVersions, ForecastEffectiveConfig, ForecastProvenance, ForecastRunStatus,
    ForecastSelectionSource,
};
use super::selection_tickets::SelectionProof;
use super::types::{ForecastRequest, ForecastResult};

pub fn complete(
    result: &mut ForecastResult,
    request: &ForecastRequest,
    profile: &DataProfile,
    source: ForecastSelectionSource,
    proof: Option<&SelectionProof>,
    duration_ms: u64,
) -> Result<(), String> {
    let spec = super::catalog::find_model(&result.model).ok_or("Modèle inconnu")?;
    let runtime = super::registry::find_runtime(&result.model).ok_or("Moteur indisponible")?;
    let model_revision = spec
        .hf_revision
        .or(spec.github_revision)
        .or_else(|| spec.is_cloud.then_some("cloud_api"))
        .map(str::to_string);
    let model_parameters = super::model_config::effective_values(&result.model)?;
    let hardware_class = proof.map(|item| item.resource_fit).unwrap_or_else(|| {
        super::hardware_profile::resource_fit(spec, super::hardware_profile::detect())
    });
    let mut reasons = request.selection_reason_codes.clone();
    if source == ForecastSelectionSource::Manual {
        reasons.push("manual_selection".into());
    }
    if let Some(proof) = proof {
        reasons.extend(proof.reasons.iter().cloned());
    }
    reasons = bounded_unique(reasons);
    result.schema_version = super::types::CURRENT_SCHEMA_VERSION;
    result.provenance = ForecastProvenance {
        data_fingerprint: profile.fingerprint.clone(),
        quality_profile_id: Some(profile.id.clone()),
        model_revision,
        dependency_versions: ForecastDependencyVersions {
            application: env!("CARGO_PKG_VERSION").to_string(),
            forecast_runtime: runtime.family_id.to_string(),
        },
        effective_config: ForecastEffectiveConfig {
            horizon: request.horizon,
            frequency: request.frequency.clone(),
            confidence_level: request.confidence_level,
            series_count: profile.series_count,
            covariate_count: request.covariate_columns.len(),
            model_parameters,
        },
        selection_source: source,
        selection_reason_codes: reasons,
        hardware_class: Some(hardware_class.code().to_string()),
        selection_basis: Some(proof.map(|item| item.basis).unwrap_or("manual").to_string()),
        backtest: proof.and_then(|item| item.backtest.clone()),
        duration_ms,
        status: ForecastRunStatus::Complete,
    };
    result.advanced_analytics = Some(super::advanced::analyze(result));
    Ok(())
}

fn bounded_unique(reasons: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    reasons
        .into_iter()
        .filter(|reason| seen.insert(reason.clone()))
        .take(super::limits::MAX_SELECTION_REASON_CODES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasons_are_unique_and_bounded() {
        let reasons = vec!["a", "a", "b", "c", "d", "e", "f", "g", "h", "i"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let result = bounded_unique(reasons);
        assert_eq!(
            result.len(),
            super::super::limits::MAX_SELECTION_REASON_CODES
        );
        assert_eq!(result[0], "a");
        assert_eq!(result[1], "b");
    }
}
