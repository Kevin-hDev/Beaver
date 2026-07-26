use serde_json::Value;

use super::{AutoCandidate, CandidateBacktest};
use crate::services::forecast::data_quality::DataProfile;
use crate::services::forecast::evaluation::types::{BacktestIndexResult, BacktestIndexSummary};
use crate::services::forecast::hardware_profile::{HardwareProfile, ResourceFit};
use crate::services::forecast::{catalog, limits, registry, validation};

pub(super) fn evaluate(
    model: &Value,
    profile: &DataProfile,
    cloud_allowed: bool,
    hardware: HardwareProfile,
    evidence: &[BacktestIndexSummary],
    explicitly_requested: bool,
) -> Result<AutoCandidate, &'static str> {
    let id = model["id"].as_str().ok_or("invalid_model")?;
    let spec = catalog::find_model(id).ok_or("unknown_model")?;
    let runtime = registry::find_runtime(id).ok_or("adapter_unavailable")?;
    let runtime_ready = model["runtime_ready"].as_bool().unwrap_or(false);
    if !model["runnable"].as_bool().unwrap_or(false) {
        return Err(if !spec.is_cloud && !runtime_ready {
            "runtime_not_ready"
        } else {
            "not_runnable"
        });
    }
    if !runtime_ready {
        return Err("runtime_not_ready");
    }
    if spec.is_cloud && !cloud_allowed {
        return Err("cloud_not_allowed");
    }
    let confidence = profile.confidence_level.ok_or("profile_outdated")?;
    if !validation::supports_confidence(id, confidence) {
        return Err("confidence_unsupported");
    }
    if !task_compatible(spec, runtime, profile) {
        return Err("task_incompatible");
    }
    let resource_fit = crate::services::forecast::hardware_profile::resource_fit(spec, hardware);
    if resource_fit == ResourceFit::Insufficient {
        return Err("resources_insufficient");
    }
    if resource_fit == ResourceFit::Unknown && spec.ram_mb > limits::MAX_UNKNOWN_RESOURCE_RAM_MB {
        return Err("resources_unknown");
    }
    let full_evidence = evidence_for(id, evidence);
    let backtest = full_evidence
        .as_ref()
        .and_then(|summary| compact_backtest(id, summary));
    let mut reasons = reasons(profile, resource_fit, explicitly_requested);
    if backtest.is_some() {
        reasons.push("backtest_available");
    }
    if backtest.as_ref().and_then(|item| item.beats_best_baseline) == Some(true) {
        reasons.push("beats_best_baseline");
    }
    reasons.truncate(limits::MAX_AUTO_REASONS);
    Ok(AutoCandidate {
        model_id: id.to_string(),
        compatibility: "compatible",
        resource_fit,
        reasons,
        interval_capability: crate::services::forecast::interval_capability::for_model(id),
        backtest,
        evidence: full_evidence,
        estimated_ram_mb: spec.ram_mb,
    })
}

fn task_compatible(
    spec: &catalog::ForecastModelSpec,
    runtime: &registry::ForecastRuntimeSpec,
    profile: &DataProfile,
) -> bool {
    let needs_covariates = !profile.covariate_columns.is_empty();
    let needs_future = needs_covariates && profile.future_rows > 0;
    validation::effective_horizon_max(spec.id, spec.horizon_max)
        .is_ok_and(|maximum| profile.horizon <= maximum)
        && validation::supports_frequency(spec, &profile.frequency)
        && (profile.series_count <= 1 || runtime.capabilities.multi_series)
        && (!needs_covariates || runtime.capabilities.past_covariates)
        && (!needs_future || runtime.capabilities.future_covariates)
        && runtime.capabilities.probabilistic
}

fn evidence_for(model_id: &str, evidence: &[BacktestIndexSummary]) -> Option<BacktestIndexSummary> {
    evidence
        .iter()
        .find(|summary| {
            let baseline = summary.results.iter().any(|result| {
                result.kind == crate::services::forecast::evaluation::types::BacktestKind::Baseline
                    && result.metrics.is_some()
            });
            baseline
                && summary.results.iter().any(|result| {
                    result.model_id == model_id
                        && result.kind
                            == crate::services::forecast::evaluation::types::BacktestKind::Model
                        && result.metrics.is_some()
                })
        })
        .cloned()
}

fn compact_backtest(model_id: &str, summary: &BacktestIndexSummary) -> Option<CandidateBacktest> {
    let result: &BacktestIndexResult = summary.results.iter().find(|result| {
        result.model_id == model_id
            && result.kind == crate::services::forecast::evaluation::types::BacktestKind::Model
    })?;
    Some(CandidateBacktest {
        evaluated_at: summary.created_at.clone(),
        windows: summary.windows,
        metrics: result.metrics.clone()?,
        duration_ms: result.duration_ms,
        max_memory_mb: result.max_memory_mb,
        beats_best_baseline: result.beats_best_baseline,
        calibration: result.calibration.clone(),
    })
}

fn reasons(profile: &DataProfile, fit: ResourceFit, requested: bool) -> Vec<&'static str> {
    let mut reasons = vec![
        "horizon_supported",
        "frequency_supported",
        "confidence_supported",
    ];
    if requested {
        reasons.push("user_requested");
    }
    if profile.series_count > 1 {
        reasons.push("multi_series_supported");
    }
    if !profile.covariate_columns.is_empty() {
        reasons.push("covariates_supported");
    }
    if profile.future_rows > 0 && !profile.covariate_columns.is_empty() {
        reasons.push("future_covariates_supported");
    }
    if fit != ResourceFit::Unknown {
        reasons.push("resources_checked");
    }
    reasons
}
