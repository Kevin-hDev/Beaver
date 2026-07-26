use super::{EnsembleMember, ForecastEnsemble};
use crate::services::forecast::types::{ForecastResult, Prediction, Quantiles};

pub(super) fn weighted(
    forecasts: &[ForecastResult],
    members: Vec<EnsembleMember>,
) -> Result<ForecastEnsemble, String> {
    let reference = forecasts
        .first()
        .ok_or("Prévisions d'ensemble indisponibles")?;
    let predictions = combine_points(forecasts, &members, &reference.predictions);
    let quantiles = Quantiles {
        q10: combine_values(forecasts, &members, |forecast| &forecast.quantiles.q10),
        q50: predictions.iter().map(|point| point.value).collect(),
        q90: combine_values(forecasts, &members, |forecast| &forecast.quantiles.q90),
    };
    let finite = predictions.iter().all(|point| point.value.is_finite())
        && quantiles
            .q10
            .iter()
            .chain(&quantiles.q50)
            .chain(&quantiles.q90)
            .all(|value| value.is_finite());
    let ordered = quantiles
        .q10
        .iter()
        .zip(&quantiles.q50)
        .zip(&quantiles.q90)
        .all(|((lower, middle), upper)| lower <= middle && middle <= upper);
    if !finite || !ordered {
        return Err("Intervalles d'ensemble invalides".into());
    }
    Ok(ForecastEnsemble {
        created_at: chrono::Utc::now().to_rfc3339(),
        method: "inverse_mase_weighted".into(),
        validation_status: "members_backtested_ensemble_not_backtested".into(),
        members,
        predictions,
        quantiles,
    })
}

fn combine_points(
    forecasts: &[ForecastResult],
    members: &[EnsembleMember],
    reference: &[Prediction],
) -> Vec<Prediction> {
    reference
        .iter()
        .enumerate()
        .map(|(index, point)| Prediction {
            date: point.date.clone(),
            series_id: point.series_id.clone(),
            value: forecasts
                .iter()
                .zip(members)
                .map(|(forecast, member)| forecast.predictions[index].value * member.weight)
                .sum(),
        })
        .collect()
}

fn combine_values<F: Fn(&ForecastResult) -> &[f64]>(
    forecasts: &[ForecastResult],
    members: &[EnsembleMember],
    values: F,
) -> Vec<f64> {
    (0..forecasts[0].predictions.len())
        .map(|index| {
            forecasts
                .iter()
                .zip(members)
                .map(|(forecast, member)| values(forecast)[index] * member.weight)
                .sum()
        })
        .collect()
}
