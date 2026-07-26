use std::collections::BTreeSet;

use super::ensemble_combine::weighted;
use super::EnsembleMember;
use crate::services::forecast::evaluation::types::BacktestKind;
use crate::services::forecast::limits::MAX_ENSEMBLE_MODELS;
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::types::ForecastResult;

pub async fn create(
    analysis_id: &str,
    requested: &[String],
    chronos: Option<&ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let mut analysis = crate::services::forecast::storage::load(analysis_id).await?;
    let members = select_members(&analysis, requested)?;
    for member in &members {
        crate::services::forecast::catalog::find_model(&member.model_id)
            .ok_or("Modèle d'ensemble inconnu")?;
    }
    let mut forecasts = Vec::with_capacity(members.len());
    for member in &members {
        if member.model_id == analysis.model {
            forecasts.push(analysis.clone());
        } else {
            forecasts.push(
                crate::services::forecast::scenario_context_run::rerun_with_model(
                    &analysis,
                    analysis.input_data.rows.clone(),
                    &member.model_id,
                    chronos,
                )
                .await?,
            );
        }
    }
    validate_alignment(&forecasts)?;
    analysis.ensemble = Some(weighted(&forecasts, members)?);
    crate::services::forecast::storage::save(&mut analysis).await?;
    Ok(analysis)
}

fn select_members(
    analysis: &ForecastResult,
    requested: &[String],
) -> Result<Vec<EnsembleMember>, String> {
    if requested.len() > MAX_ENSEMBLE_MODELS {
        return Err("Trop de modèles pour l'ensemble".into());
    }
    if requested.iter().any(|id| {
        id.trim().is_empty()
            || id.chars().count() > crate::services::forecast::limits::MAX_MODEL_ID_CHARS
    }) {
        return Err("Liste de modèles d'ensemble invalide".into());
    }
    let evaluation = analysis
        .evaluation
        .as_ref()
        .ok_or("Un backtest multi-modèles est requis")?;
    let requested_count = requested.len();
    let requested: BTreeSet<_> = requested.iter().map(|id| id.trim()).collect();
    if requested.iter().any(|id| id.is_empty()) || requested.len() != requested_count {
        return Err("Liste de modèles d'ensemble invalide".into());
    }
    let mut candidates: Vec<_> = evaluation
        .results
        .iter()
        .filter(|result| result.kind == BacktestKind::Model && result.failure.is_none())
        .filter_map(|result| {
            let metrics = result.metrics.as_ref()?;
            (requested.is_empty() || requested.contains(result.model_id.as_str())).then(|| {
                (
                    result.rank.unwrap_or(usize::MAX),
                    result.model_id.clone(),
                    metrics.mase,
                )
            })
        })
        .collect();
    candidates.sort_by_key(|candidate| candidate.0);
    candidates.truncate(MAX_ENSEMBLE_MODELS);
    if candidates.len() < 2 || (!requested.is_empty() && candidates.len() != requested.len()) {
        return Err("Au moins deux modèles backtestés avec succès sont requis".into());
    }
    let inverse_total: f64 = candidates
        .iter()
        .map(|(_, _, mase)| 1.0 / mase.max(1e-9))
        .sum();
    Ok(candidates
        .into_iter()
        .map(|(_, model_id, mase)| EnsembleMember {
            model_id,
            weight: (1.0 / mase.max(1e-9)) / inverse_total,
            backtest_mase: mase,
        })
        .collect())
}

fn validate_alignment(forecasts: &[ForecastResult]) -> Result<(), String> {
    let Some(reference) = forecasts.first() else {
        return Err("Prévisions d'ensemble indisponibles".into());
    };
    let aligned = forecasts.iter().all(|forecast| {
        crate::services::forecast::result_validation::validate_stored_quantiles(forecast).is_ok()
            && forecast.predictions.len() == reference.predictions.len()
            && forecast
                .predictions
                .iter()
                .all(|point| point.value.is_finite())
            && forecast
                .predictions
                .iter()
                .zip(&reference.predictions)
                .all(|(left, right)| left.date == right.date && left.series_id == right.series_id)
    });
    if aligned {
        Ok(())
    } else {
        Err("Prévisions d'ensemble incompatibles".into())
    }
}

#[cfg(test)]
#[path = "ensemble_tests.rs"]
mod tests;
