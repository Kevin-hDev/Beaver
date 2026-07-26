use super::common::{fmt, ExportBundle, TableRow};

pub(super) fn rows(bundle: &ExportBundle) -> Vec<TableRow> {
    let mut rows = Vec::new();
    if let Some(advanced) = &bundle.analysis.advanced_analytics {
        for item in &advanced.anomalies {
            rows.push(row(
                "residual_anomaly",
                &item.id,
                &item.date,
                item.series_id.as_deref().unwrap_or(""),
                &fmt(item.observed),
                &format!("score={:.4};expected={:.6}", item.score, item.expected),
                &item.method,
            ));
        }
        for item in &advanced.variable_importance.items {
            rows.push(row(
                "variable_importance",
                &item.name,
                "",
                "",
                &fmt(item.normalized_score),
                &format!(
                    "direction={};validation_mae={:.6}",
                    item.direction, item.validation_mae
                ),
                &advanced.variable_importance.method,
            ));
        }
        for item in &advanced.drift {
            rows.push(row(
                "drift",
                item.series_id.as_deref().unwrap_or("series-1"),
                "",
                item.series_id.as_deref().unwrap_or(""),
                &item.score.map(fmt).unwrap_or_default(),
                &format!("detected={};severity={}", item.detected, item.severity),
                &item.method,
            ));
        }
    }
    if let Some(ensemble) = &bundle.analysis.ensemble {
        for member in &ensemble.members {
            rows.push(row(
                "ensemble_member",
                &member.model_id,
                "",
                "",
                &fmt(member.weight),
                &format!("backtest_mase={:.6}", member.backtest_mase),
                &ensemble.method,
            ));
        }
    }
    rows
}

fn row(
    section: &str,
    name: &str,
    date: &str,
    series_id: &str,
    value: &str,
    text: &str,
    source: &str,
) -> TableRow {
    TableRow {
        section: section.into(),
        name: name.into(),
        date: date.into(),
        series_id: series_id.into(),
        value: value.into(),
        q10: String::new(),
        q50: String::new(),
        q90: String::new(),
        text: text.into(),
        source: source.into(),
    }
}
