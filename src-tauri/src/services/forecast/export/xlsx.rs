use super::common::ExportBundle;
use super::quantile_labels::QuantileLabels;
use super::xlsx_style::{finish_table, label_format};
use rust_xlsxwriter::{Workbook, Worksheet};
use std::path::Path;

pub fn write(bundle: &ExportBundle, path: &Path) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let labels = QuantileLabels::for_confidence(bundle.analysis.confidence_level);
    write_metadata(workbook.add_worksheet(), bundle)?;
    write_points(
        workbook.add_worksheet(),
        "Historique",
        &bundle.analysis.input_data.history,
        None,
        &labels,
    )?;
    write_points(
        workbook.add_worksheet(),
        "Prévisions",
        &bundle.analysis.predictions,
        Some(&bundle.analysis.quantiles),
        &labels,
    )?;
    write_scenarios(workbook.add_worksheet(), bundle, &labels)?;
    write_annotations(workbook.add_worksheet(), bundle)?;
    super::xlsx_advanced::write(workbook.add_worksheet(), bundle)?;
    if let Some(ensemble) = &bundle.analysis.ensemble {
        write_points(
            workbook.add_worksheet(),
            "Ensemble",
            &ensemble.predictions,
            Some(&ensemble.quantiles),
            &labels,
        )?;
    }
    super::xlsx_input::write(workbook.add_worksheet(), bundle)?;
    workbook
        .save(path)
        .map_err(|_| "Export XLSX impossible".to_string())
}

fn write_metadata(ws: &mut Worksheet, bundle: &ExportBundle) -> Result<(), String> {
    ws.set_name("Metadata")
        .map_err(|_| "Export XLSX impossible".to_string())?;
    let a = &bundle.analysis;
    let rows = [
        ("id", a.id.as_str()),
        ("name", a.name.as_str()),
        ("model", a.model.as_str()),
        ("provider", a.provider.as_str()),
        ("target", a.target_column.as_str()),
        ("frequency", a.frequency.as_str()),
        ("created_at", a.created_at.as_str()),
    ];
    for (idx, (key, value)) in rows.iter().enumerate() {
        ws.write_string(idx as u32, 0, *key)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_string(idx as u32, 1, *value)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    ws.write_string(rows.len() as u32, 0, "horizon")
        .map_err(|_| "Export XLSX impossible".to_string())?;
    ws.write_number(rows.len() as u32, 1, a.horizon as f64)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    ws.set_column_width(0, 18)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    ws.set_column_width(1, 58)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    ws.set_column_format(0, &label_format())
        .map_err(|_| "Export XLSX impossible".to_string())?;
    Ok(())
}

fn write_points(
    ws: &mut Worksheet,
    name: &str,
    points: &[super::super::types::Prediction],
    quantiles: Option<&super::super::types::Quantiles>,
    labels: &QuantileLabels,
) -> Result<(), String> {
    ws.set_name(name)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    let [lower, median, upper] = labels.table_headers();
    for (col, header) in ["date", "series_id", "value", lower, median, upper]
        .iter()
        .enumerate()
    {
        ws.write_string(0, col as u16, *header)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    for (idx, point) in points.iter().enumerate() {
        let row = idx as u32 + 1;
        ws.write_string(row, 0, &point.date)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_string(row, 1, point.series_id.as_deref().unwrap_or(""))
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_number(row, 2, point.value)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        if let Some(q) = quantiles {
            write_optional_number(ws, row, 3, q.q10.get(idx))?;
            write_optional_number(ws, row, 4, q.q50.get(idx))?;
            write_optional_number(ws, row, 5, q.q90.get(idx))?;
        }
    }
    finish_table(ws, points.len() as u32, 5)?;
    Ok(())
}

fn write_scenarios(
    ws: &mut Worksheet,
    bundle: &ExportBundle,
    labels: &QuantileLabels,
) -> Result<(), String> {
    ws.set_name("Scenarios")
        .map_err(|_| "Export XLSX impossible".to_string())?;
    let [lower, median, upper] = labels.table_headers();
    for (col, header) in [
        "scenario",
        "date",
        "series_id",
        "value",
        lower,
        median,
        upper,
    ]
    .iter()
    .enumerate()
    {
        ws.write_string(0, col as u16, *header)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    let mut row = 1;
    for scenario in &bundle.analysis.scenarios {
        for (idx, point) in scenario.predictions.iter().enumerate() {
            ws.write_string(row, 0, &scenario.name)
                .map_err(|_| "Export XLSX impossible".to_string())?;
            ws.write_string(row, 1, &point.date)
                .map_err(|_| "Export XLSX impossible".to_string())?;
            ws.write_string(row, 2, point.series_id.as_deref().unwrap_or(""))
                .map_err(|_| "Export XLSX impossible".to_string())?;
            ws.write_number(row, 3, point.value)
                .map_err(|_| "Export XLSX impossible".to_string())?;
            write_optional_number(ws, row, 4, scenario.quantiles.q10.get(idx))?;
            write_optional_number(ws, row, 5, scenario.quantiles.q50.get(idx))?;
            write_optional_number(ws, row, 6, scenario.quantiles.q90.get(idx))?;
            row += 1;
        }
    }
    finish_table(ws, row.saturating_sub(1), 6)?;
    Ok(())
}

fn write_annotations(ws: &mut Worksheet, bundle: &ExportBundle) -> Result<(), String> {
    ws.set_name("Notes")
        .map_err(|_| "Export XLSX impossible".to_string())?;
    for (col, header) in ["kind", "date", "title", "text", "source"]
        .iter()
        .enumerate()
    {
        ws.write_string(0, col as u16, *header)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    let mut row = 1;
    for note in &bundle.notes {
        ws.write_string(row, 0, "note")
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_string(row, 1, &note.date)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_string(row, 2, &note.title)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_string(row, 3, &note.content)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        ws.write_string(row, 4, &note.source)
            .map_err(|_| "Export XLSX impossible".to_string())?;
        row += 1;
    }
    finish_table(ws, row.saturating_sub(1), 4)?;
    ws.set_column_width(3, 80)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    Ok(())
}

fn write_optional_number(
    ws: &mut Worksheet,
    row: u32,
    col: u16,
    value: Option<&f64>,
) -> Result<(), String> {
    if let Some(value) = value {
        ws.write_number(row, col, *value)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    Ok(())
}
