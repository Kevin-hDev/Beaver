use super::fold_sources::SourcePoint;
use crate::services::forecast::limits::{MAX_BACKTEST_HORIZON, MAX_BACKTEST_WINDOWS};
use crate::services::forecast::types::ForecastResult;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) struct ActualPoint {
    pub date: String,
    pub series_id: Option<String>,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub(super) struct SeriesFold {
    pub series_id: Option<String>,
    pub training: Vec<f64>,
    pub actual: Vec<f64>,
}

#[derive(Debug, Clone)]
pub(super) struct RollingFold {
    pub index: usize,
    pub rows: Vec<Value>,
    pub actual: Vec<ActualPoint>,
    pub series: Vec<SeriesFold>,
}

#[derive(Debug, Clone)]
pub(super) struct BacktestPlan {
    pub horizon: usize,
    pub warning: Option<String>,
    pub folds: Vec<RollingFold>,
}

pub(super) fn build(
    analysis: &ForecastResult,
    requested_windows: Option<usize>,
) -> Result<BacktestPlan, String> {
    let grouped = group_points(super::fold_sources::load(analysis)?);
    let minimum = grouped.values().map(Vec::len).min().unwrap_or(0);
    if minimum < 4 {
        return Err("Historique trop court pour un backtest".into());
    }
    let requested_horizon = analysis.horizon as usize;
    if requested_horizon == 0 {
        return Err("Horizon de backtest invalide".into());
    }
    let horizon = requested_horizon
        .min(MAX_BACKTEST_HORIZON)
        .min((minimum / 3).max(1));
    let desired = requested_windows
        .unwrap_or(3)
        .clamp(1, MAX_BACKTEST_WINDOWS);
    let available = minimum.saturating_sub(3) / horizon;
    let windows = desired.min(available);
    if windows == 0 {
        return Err("Historique trop court pour un backtest".into());
    }
    let warning = warning(requested_horizon, horizon, windows);
    let folds = (0..windows)
        .map(|index| build_fold(&grouped, horizon, windows, index, &analysis.target_column))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BacktestPlan {
        horizon,
        warning,
        folds,
    })
}

fn group_points(points: Vec<SourcePoint>) -> BTreeMap<String, Vec<SourcePoint>> {
    let mut grouped = BTreeMap::<String, Vec<SourcePoint>>::new();
    for point in points {
        grouped
            .entry(point.series_id.clone().unwrap_or_default())
            .or_default()
            .push(point);
    }
    grouped
}

fn build_fold(
    grouped: &BTreeMap<String, Vec<SourcePoint>>,
    horizon: usize,
    windows: usize,
    index: usize,
    target_column: &str,
) -> Result<RollingFold, String> {
    let mut rows = Vec::new();
    let mut actual = Vec::new();
    let mut series = Vec::new();
    for (series_key, points) in grouped {
        let train_end = points.len() - horizon * (windows - index);
        let test_end = train_end + horizon;
        let training = points[..train_end]
            .iter()
            .map(|point| point.value)
            .collect();
        let expected = points[train_end..test_end]
            .iter()
            .map(|point| point.value)
            .collect();
        series.push(SeriesFold {
            series_id: (!series_key.is_empty()).then(|| series_key.clone()),
            training,
            actual: expected,
        });
        for (position, point) in points[..test_end].iter().enumerate() {
            let mut row = point.row.clone();
            if position >= train_end {
                row.as_object_mut()
                    .ok_or("Données de backtest invalides")?
                    .insert(target_column.to_string(), Value::Null);
                actual.push(ActualPoint {
                    date: point.date.clone(),
                    series_id: point.series_id.clone(),
                    value: point.value,
                });
            }
            rows.push(row);
        }
    }
    Ok(RollingFold {
        index,
        rows,
        actual,
        series,
    })
}

fn warning(requested: usize, effective: usize, windows: usize) -> Option<String> {
    if windows < 3 {
        Some("short_history".into())
    } else if effective < requested {
        Some("reduced_horizon".into())
    } else {
        None
    }
}

#[cfg(test)]
#[path = "folds_tests.rs"]
mod tests;
