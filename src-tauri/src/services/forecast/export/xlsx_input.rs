use super::common::ExportBundle;
use super::xlsx_style::finish_table;
use rust_xlsxwriter::Worksheet;
use serde_json::Value;

pub fn write(ws: &mut Worksheet, bundle: &ExportBundle) -> Result<(), String> {
    ws.set_name("Input data")
        .map_err(|_| "Export XLSX impossible".to_string())?;
    for (col, header) in bundle.analysis.input_data.columns.iter().enumerate() {
        ws.write_string(0, col as u16, header)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    for (row_idx, row_value) in bundle.analysis.input_data.rows.iter().enumerate() {
        for (col_idx, header) in bundle.analysis.input_data.columns.iter().enumerate() {
            let value = row_value.get(header).unwrap_or(&Value::Null);
            write_json_value(ws, row_idx as u32 + 1, col_idx as u16, value)?;
        }
    }
    let last_row = bundle.analysis.input_data.rows.len() as u32;
    let last_col = bundle.analysis.input_data.columns.len().saturating_sub(1) as u16;
    finish_table(ws, last_row, last_col)
}

fn write_json_value(ws: &mut Worksheet, row: u32, col: u16, value: &Value) -> Result<(), String> {
    match value {
        Value::Number(n) => ws.write_number(row, col, n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => ws.write_boolean(row, col, *b),
        Value::String(s) => ws.write_string(row, col, s),
        Value::Null => return Ok(()),
        _ => ws.write_string(row, col, value.to_string()),
    }
    .map(|_| ())
    .map_err(|_| "Export XLSX impossible".to_string())
}
