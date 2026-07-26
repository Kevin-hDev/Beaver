use super::common::ExportBundle;
use super::xlsx_style::finish_table;
use rust_xlsxwriter::Worksheet;

pub(super) fn write(ws: &mut Worksheet, bundle: &ExportBundle) -> Result<(), String> {
    ws.set_name("Advanced")
        .map_err(|_| "Export XLSX impossible".to_string())?;
    for (column, header) in ["category", "series", "name", "value", "details"]
        .iter()
        .enumerate()
    {
        ws.write_string(0, column as u16, *header)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    let mut row = 1u32;
    if let Some(advanced) = &bundle.analysis.advanced_analytics {
        for item in &advanced.decomposition {
            write_row(
                ws,
                row,
                "decomposition",
                item.series_id.as_deref().unwrap_or(""),
                &item.method,
                item.seasonal_strength,
                &format!("period={}", item.period),
            )?;
            row += 1;
        }
        for item in &advanced.anomalies {
            write_row(
                ws,
                row,
                "residual_anomaly",
                item.series_id.as_deref().unwrap_or(""),
                &item.date,
                Some(item.score),
                &format!(
                    "observed={:.6};expected={:.6}",
                    item.observed, item.expected
                ),
            )?;
            row += 1;
        }
        for item in &advanced.variable_importance.items {
            write_row(
                ws,
                row,
                "variable_importance",
                "",
                &item.name,
                Some(item.normalized_score),
                &format!(
                    "direction={};method={}",
                    item.direction, advanced.variable_importance.method
                ),
            )?;
            row += 1;
        }
        for item in &advanced.drift {
            write_row(
                ws,
                row,
                "drift",
                item.series_id.as_deref().unwrap_or(""),
                &item.severity,
                item.score,
                &format!("detected={}", item.detected),
            )?;
            row += 1;
        }
    }
    if let Some(ensemble) = &bundle.analysis.ensemble {
        for member in &ensemble.members {
            write_row(
                ws,
                row,
                "ensemble_member",
                "",
                &member.model_id,
                Some(member.weight),
                &format!("backtest_mase={:.6}", member.backtest_mase),
            )?;
            row += 1;
        }
    }
    finish_table(ws, row.saturating_sub(1), 4)?;
    ws.set_column_width(4, 54)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    Ok(())
}

fn write_row(
    ws: &mut Worksheet,
    row: u32,
    category: &str,
    series: &str,
    name: &str,
    value: Option<f64>,
    details: &str,
) -> Result<(), String> {
    for (column, text) in [(0, category), (1, series), (2, name), (4, details)] {
        ws.write_string(row, column, text)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    if let Some(value) = value {
        ws.write_number(row, 3, value)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    Ok(())
}
