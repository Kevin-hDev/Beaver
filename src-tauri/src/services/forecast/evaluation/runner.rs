use super::baselines::{seasonal_period, Baseline};
use super::types::{BacktestKind, BacktestRequest, ForecastEvaluation, ModelBacktestResult};
use crate::services::forecast::limits::{MAX_BACKTEST_MODELS, MAX_CONCURRENT_BACKTESTS};
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::types::ForecastResult;
use tokio::sync::Semaphore;

static BACKTEST_LIMIT: Semaphore = Semaphore::const_new(MAX_CONCURRENT_BACKTESTS);

pub async fn run(
    request: BacktestRequest,
    chronos: &ChronosSidecar,
) -> Result<ForecastResult, String> {
    let _permit = BACKTEST_LIMIT
        .try_acquire()
        .map_err(|_| "Un backtest Forecast est déjà en cours".to_string())?;
    validate_request(&request)?;
    let mut analysis = crate::services::forecast::storage::load(&request.analysis_id).await?;
    validate_analysis(&analysis)?;
    let plan = super::folds::build(&analysis, request.max_windows)?;
    let model_ids = model_ids(&request, &analysis)?;
    let period = seasonal_period(&analysis.frequency);
    let mut results: Vec<_> = Baseline::ALL
        .into_iter()
        .map(|baseline| {
            super::baseline_runner::evaluate(baseline, &plan, period, analysis.confidence_level)
        })
        .collect();
    if !has_successful_baseline(&results) {
        return Err("Aucune baseline exploitable pour ce backtest".into());
    }
    for model_id in model_ids {
        results.push(super::model_runner::evaluate(&analysis, &model_id, &plan, chronos).await);
    }
    if results.len() > super::super::limits::MAX_BACKTEST_RESULTS {
        return Err("Trop de résultats de backtest".into());
    }
    rank(&mut results);
    super::calibration::apply(&mut analysis, &results)?;
    analysis.evaluation = Some(ForecastEvaluation {
        schema_version: 1,
        created_at: chrono::Utc::now().to_rfc3339(),
        horizon: plan.horizon,
        windows: plan.folds.len(),
        warning: plan.warning,
        results,
    });
    analysis.ensemble = None;
    crate::services::forecast::storage::save(&mut analysis).await?;
    Ok(analysis)
}

fn validate_analysis(analysis: &ForecastResult) -> Result<(), String> {
    if !crate::services::forecast::interval_capability::valid_input_level(analysis.confidence_level)
    {
        return Err("Niveau de confiance Forecast invalide".into());
    }
    Ok(())
}

fn has_successful_baseline(results: &[ModelBacktestResult]) -> bool {
    results.iter().any(|result| {
        result.kind == BacktestKind::Baseline
            && result.metrics.is_some()
            && result.failure.is_none()
    })
}

fn validate_request(request: &BacktestRequest) -> Result<(), String> {
    if request.model_ids.len() > MAX_BACKTEST_MODELS {
        return Err("Trop de modèles à évaluer".into());
    }
    if request
        .max_windows
        .is_some_and(|value| value == 0 || value > super::super::limits::MAX_BACKTEST_WINDOWS)
    {
        return Err("Nombre de fenêtres invalide".into());
    }
    Ok(())
}

fn model_ids(request: &BacktestRequest, analysis: &ForecastResult) -> Result<Vec<String>, String> {
    let requested = if request.model_ids.is_empty() {
        vec![analysis.model.clone()]
    } else {
        request.model_ids.clone()
    };
    let mut unique = Vec::new();
    for id in requested {
        let trimmed = id.trim();
        if trimmed.is_empty()
            || trimmed.chars().count() > super::super::limits::MAX_MODEL_ID_CHARS
            || crate::services::forecast::registry::find_runtime(trimmed).is_none()
        {
            return Err("Modèle de backtest invalide".into());
        }
        if !unique.iter().any(|existing| existing == trimmed) {
            unique.push(trimmed.to_string());
        }
    }
    if unique.len() > MAX_BACKTEST_MODELS {
        return Err("Trop de modèles à évaluer".into());
    }
    Ok(unique)
}

fn rank(results: &mut [ModelBacktestResult]) {
    let best_baseline = results
        .iter()
        .enumerate()
        .filter(|(_, result)| result.kind == BacktestKind::Baseline)
        .filter(|(_, result)| result.metrics.is_some())
        .min_by(|(_, left), (_, right)| super::ranking::compare_results(left, right))
        .map(|(index, _)| index);
    let mut order: Vec<usize> = results
        .iter()
        .enumerate()
        .filter_map(|(index, result)| result.metrics.as_ref().map(|_| index))
        .collect();
    order.sort_by(|left, right| super::ranking::compare_results(&results[*left], &results[*right]));
    for (rank, index) in order.into_iter().enumerate() {
        results[index].rank = Some(rank + 1);
        if results[index].kind == BacktestKind::Baseline {
            results[index].beats_best_baseline = None;
        } else {
            results[index].beats_best_baseline = best_baseline.map(|baseline| {
                super::ranking::compare_results(&results[index], &results[baseline]).is_lt()
            });
        }
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
