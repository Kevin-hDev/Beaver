use super::baselines::seasonal_period;
use super::folds::BacktestPlan;
use super::metrics::{self, Observation};
use super::types::{BacktestFailure, BacktestFoldMetric, BacktestKind, ModelBacktestResult};
use crate::services::forecast::sidecar::{ChronosSidecar, SidecarEndpoint};
use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use std::time::Instant;
use zeroize::Zeroizing;

enum Executor {
    Local(SidecarEndpoint),
    Cloud(Zeroizing<String>),
}

pub(super) async fn evaluate(
    analysis: &ForecastResult,
    model_id: &str,
    plan: &BacktestPlan,
    chronos: &ChronosSidecar,
) -> ModelBacktestResult {
    let started = Instant::now();
    let outcome = evaluate_inner(analysis, model_id, plan, chronos)
        .await
        .and_then(|(observations, folds, max_memory_mb)| {
            let metrics = metrics::summarize(&observations, analysis.confidence_level)
                .ok_or_else(|| "invalid_backtest_data".to_string())?;
            Ok((
                metrics,
                metrics::calibration(&observations, analysis.confidence_level),
                folds,
                max_memory_mb,
            ))
        });
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    match outcome {
        Ok((metrics, calibration, folds, max_memory_mb)) => ModelBacktestResult {
            model_id: model_id.to_string(),
            kind: BacktestKind::Model,
            metrics: Some(metrics),
            calibration,
            folds,
            duration_ms,
            max_memory_mb,
            rank: None,
            beats_best_baseline: None,
            warning: None,
            failure: None,
        },
        Err(code) => {
            let failure = BacktestFailure::from_code(&code);
            ModelBacktestResult {
                model_id: model_id.to_string(),
                kind: BacktestKind::Model,
                metrics: None,
                calibration: None,
                folds: Vec::new(),
                duration_ms,
                max_memory_mb: None,
                rank: None,
                beats_best_baseline: None,
                warning: Some(failure.code.clone()),
                failure: Some(failure),
            }
        }
    }
}

async fn evaluate_inner(
    analysis: &ForecastResult,
    model_id: &str,
    plan: &BacktestPlan,
    chronos: &ChronosSidecar,
) -> Result<(Vec<Observation>, Vec<BacktestFoldMetric>, Option<u64>), String> {
    let runtime = crate::services::forecast::registry::find_runtime(model_id)
        .filter(|runtime| crate::services::forecast::registry::has_predict_adapter(runtime))
        .ok_or("model_unavailable")?;
    if !crate::services::forecast::validation::supports_confidence(
        model_id,
        analysis.confidence_level,
    ) {
        return Err("confidence_unsupported".into());
    }
    if analysis.input_data.series_column.is_some() && !runtime.capabilities.multi_series {
        return Err("model_incompatible".into());
    }
    if !analysis.covariates_used.is_empty() && !runtime.capabilities.future_covariates {
        return Err("model_incompatible".into());
    }
    let _prediction_guard = if crate::services::forecast::registry::is_cloud(runtime) {
        None
    } else {
        Some(chronos.lock_prediction().await)
    };
    let executor = if crate::services::forecast::registry::is_cloud(runtime) {
        let policy =
            crate::services::forecast::selection_policy::get().map_err(|_| "cloud_not_allowed")?;
        if model_id != analysis.model && !policy.allow_cloud_in_auto {
            return Err("cloud_not_allowed".into());
        }
        Executor::Cloud(
            crate::services::api_keys::get_key("nixtla").map_err(|_| "cloud_not_configured")?,
        )
    } else {
        if !crate::services::forecast::model_manager::is_ready(model_id) {
            return Err("model_not_installed".into());
        }
        let spec =
            crate::services::forecast::catalog::find_model(model_id).ok_or("model_unavailable")?;
        crate::services::forecast::hardware_profile::validate_model_resources(spec)
            .map_err(|_| "resources_unavailable")?;
        Executor::Local(
            crate::services::forecast::sidecar::start(chronos, model_id, runtime.family_id)
                .await
                .map_err(|_| "model_start_failed")?,
        )
    };
    let sampler = match &executor {
        Executor::Local(endpoint) => super::memory_sampler::MemorySampler::start(endpoint.pid),
        Executor::Cloud(_) => None,
    };
    let outcome = evaluate_windows(analysis, model_id, plan, &executor).await;
    let max_memory_mb = sampler.and_then(super::memory_sampler::MemorySampler::finish);
    if matches!(executor, Executor::Local(_)) {
        crate::services::forecast::sidecar::schedule_idle_stop(chronos);
    }
    outcome.map(|(observations, folds)| (observations, folds, max_memory_mb))
}

async fn evaluate_windows(
    analysis: &ForecastResult,
    model_id: &str,
    plan: &BacktestPlan,
    executor: &Executor,
) -> Result<(Vec<Observation>, Vec<BacktestFoldMetric>), String> {
    let mut observations = Vec::new();
    let mut fold_metrics = Vec::new();
    let period = seasonal_period(&analysis.frequency);
    for fold in &plan.folds {
        let request = super::model_request::build(analysis, fold, model_id, plan.horizon)
            .map_err(|_| "model_request_invalid".to_string())?;
        let forecast = predict(executor, &request).await?;
        let (mut next, metric) = super::model_observations::collect(fold, &forecast, period)?;
        observations.append(&mut next);
        fold_metrics.push(metric);
    }
    Ok((observations, fold_metrics))
}

async fn predict(executor: &Executor, request: &ForecastRequest) -> Result<ForecastResult, String> {
    match executor {
        Executor::Local(endpoint) => crate::services::forecast::client_chronos::predict(
            &endpoint.base_url,
            endpoint.auth_token.as_str(),
            request,
            None,
        )
        .await
        .map_err(normalize_prediction_error),
        Executor::Cloud(key) => {
            crate::services::forecast::client_nixtla::predict(key, request, None)
                .await
                .map_err(|_| "prediction_runtime_failed".to_string())
        }
    }
}

fn normalize_prediction_error(error: String) -> String {
    match error.as_str() {
        "prediction_rejected" | "prediction_runtime_failed" | "invalid_prediction_output" => error,
        _ => "window_failed".to_string(),
    }
}
