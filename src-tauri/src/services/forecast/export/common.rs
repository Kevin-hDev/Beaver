use super::super::notes::ForecastNote;
use super::super::types::{ForecastResult, Prediction};
use super::quantile_labels::QuantileLabels;
use super::spreadsheet_text::safe_csv_cell;
use serde::Serialize;
use serde_json::json;
use std::path::Path;

#[derive(Serialize)]
pub struct ExportBundle {
    pub analysis: ForecastResult,
    pub notes: Vec<ForecastNote>,
}

pub struct TableRow {
    pub section: String,
    pub name: String,
    pub date: String,
    pub series_id: String,
    pub value: String,
    pub q10: String,
    pub q50: String,
    pub q90: String,
    pub text: String,
    pub source: String,
}

pub fn rows(bundle: &ExportBundle) -> Vec<TableRow> {
    let mut rows = Vec::new();
    for point in &bundle.analysis.input_data.history {
        rows.push(row("history", "", point, ["", "", ""]));
    }
    for (idx, point) in bundle.analysis.predictions.iter().enumerate() {
        rows.push(row(
            "prediction",
            "",
            point,
            [
                &q(&bundle.analysis.quantiles.q10, idx),
                &q(&bundle.analysis.quantiles.q50, idx),
                &q(&bundle.analysis.quantiles.q90, idx),
            ],
        ));
    }
    for scenario in &bundle.analysis.scenarios {
        for (idx, point) in scenario.predictions.iter().enumerate() {
            rows.push(row(
                "scenario",
                &scenario.name,
                point,
                [
                    &q(&scenario.quantiles.q10, idx),
                    &q(&scenario.quantiles.q50, idx),
                    &q(&scenario.quantiles.q90, idx),
                ],
            ));
        }
    }
    for annotation in &bundle.analysis.annotations {
        rows.push(TableRow {
            section: "annotation".into(),
            name: annotation.id.clone(),
            date: annotation.date.clone(),
            series_id: String::new(),
            value: String::new(),
            q10: String::new(),
            q50: String::new(),
            q90: String::new(),
            text: annotation.text.clone(),
            source: format!("{:?}", annotation.source).to_lowercase(),
        });
    }
    for note in &bundle.notes {
        rows.push(TableRow {
            section: "note".into(),
            name: note.title.clone(),
            date: note.date.clone(),
            series_id: String::new(),
            value: String::new(),
            q10: String::new(),
            q50: String::new(),
            q90: String::new(),
            text: note.content.clone(),
            source: note.source.clone(),
        });
    }
    rows.extend(super::advanced_rows::rows(bundle));
    rows
}

pub fn write_json(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    let quantile_labels = QuantileLabels::for_confidence(bundle.analysis.confidence_level);
    let value = json!({
        "analysis": bundle.analysis,
        "notes": bundle.notes,
        "quantile_labels": quantile_labels,
    });
    let body =
        serde_json::to_string_pretty(&value).map_err(|_| "Export JSON impossible".to_string())?;
    std::fs::write(path, body).map_err(|_| "Écriture export impossible".to_string())
}

pub fn clipboard_text(bundle: &ExportBundle) -> String {
    let a = &bundle.analysis;
    let labels = QuantileLabels::for_confidence(a.confidence_level);
    let [lower, median, upper] = labels.table_headers();
    let mut lines = vec![
        format!("Forecast: {}", a.name),
        format!("Model: {}", a.model),
        format!("Target: {}", a.target_column),
        format!("Horizon: {} {}", a.horizon, a.frequency),
        String::new(),
        format!("date\tseries\tprediction\t{lower}\t{median}\t{upper}"),
    ];
    for (idx, point) in a.predictions.iter().enumerate() {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            safe_csv_cell(&point.date),
            safe_csv_cell(point.series_id.as_deref().unwrap_or_default()),
            fmt(point.value),
            q(&a.quantiles.q10, idx),
            q(&a.quantiles.q50, idx),
            q(&a.quantiles.q90, idx)
        ));
    }
    lines.extend(super::report_advanced::lines(bundle));
    lines.join("\n")
}

pub fn safe_file_stem(value: &str) -> String {
    let mut out: String = value
        .chars()
        .filter_map(|c| {
            if c.is_ascii_alphanumeric() {
                Some(c.to_ascii_lowercase())
            } else if matches!(c, ' ' | '-' | '_') {
                Some('-')
            } else {
                None
            }
        })
        .take(64)
        .collect();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "forecast".into()
    } else {
        trimmed
    }
}

pub fn fmt(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.6}")
    } else {
        String::new()
    }
}

fn row(section: &str, name: &str, point: &Prediction, quantiles: [&str; 3]) -> TableRow {
    TableRow {
        section: section.into(),
        name: name.into(),
        date: point.date.clone(),
        series_id: point.series_id.clone().unwrap_or_default(),
        value: fmt(point.value),
        q10: quantiles[0].into(),
        q50: quantiles[1].into(),
        q90: quantiles[2].into(),
        text: String::new(),
        source: String::new(),
    }
}

fn q(values: &[f64], idx: usize) -> String {
    values.get(idx).map(|v| fmt(*v)).unwrap_or_default()
}
