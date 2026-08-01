use crate::services::agent_local::tool_office_utils::{
    coerce_values_array, normalize_formula, try_value_as_u16, try_value_as_u32, value_as_f64,
};
use crate::services::agent_local::tool_spreadsheet_write::parse_cell_ref;
use rust_xlsxwriter::Workbook;
use serde_json::Value;
use std::path::Path;

pub(super) fn create_xlsx(
    path: &Path,
    ops: &[Value],
) -> Result<(), super::tool_spreadsheet_error::SpreadsheetWriteError> {
    let mut workbook = Workbook::new();

    let extra_sheets: Vec<&str> = ops
        .iter()
        .filter(|op| op["type"].as_str() == Some("add_sheet"))
        .filter_map(|op| op["name"].as_str())
        .collect();

    let default_ws = workbook.add_worksheet();
    let default_name = default_ws.name().to_string();

    for name in &extra_sheets {
        let ws = workbook.add_worksheet();
        ws.set_name(*name)
            .map_err(|_| "Erreur création feuille".to_string())?;
    }

    for op in ops {
        let op_type = op["type"].as_str().unwrap_or("");
        let target = op["sheet"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&default_name);

        let ws = workbook
            .worksheet_from_name(target)
            .map_err(|_| format!("Feuille '{}' introuvable", target))?;

        match op_type {
            "set_cell" => apply_set_cell(ws, op)?,
            "set_formula" => apply_set_formula(ws, op)?,
            "set_row" => apply_set_row(ws, op)?,
            "set_column_width" => apply_set_column_width(ws, op)?,
            "set_format" => super::tool_spreadsheet_write_new_format::apply_set_format(ws, op)?,
            "set_number_format" => {
                super::tool_spreadsheet_write_new_format::apply_set_number_format(ws, op)?
            }
            "set_border" => super::tool_spreadsheet_write_new_format::apply_set_border(ws, op)?,
            "merge_cells" => super::tool_spreadsheet_write_new_format::apply_merge_cells(ws, op)?,
            "set_row_height" => {
                super::tool_spreadsheet_write_new_format::apply_set_row_height(ws, op)?
            }
            "add_sheet" => {}
            _ => return Err(format!("Opération inconnue: {op_type}").into()),
        }
    }

    workbook
        .save(path)
        .map_err(|_| {
            super::tool_spreadsheet_error::SpreadsheetWriteError::write(
                "Impossible de sauvegarder le fichier xlsx",
            )
        })
}

fn apply_set_cell(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    write_value(ws, row, col, &op["value"])
}

fn apply_set_formula(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    let raw = op["formula"].as_str().unwrap_or("");
    let formula = normalize_formula(raw);
    ws.write_formula(row, col, formula.as_str())
        .map(|_| ())
        .map_err(|_| "Erreur écriture formule".to_string())
}

fn apply_set_row(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let row = try_value_as_u32(&op["row"], "row")?;
    let values = match coerce_values_array(&op["values"]) {
        Some(v) => v,
        None => return Ok(()),
    };
    for (col_idx, val) in values.iter().enumerate() {
        let col = u16::try_from(col_idx).map_err(|_| "col trop grand".to_string())?;
        write_value(ws, row, col, val)?;
    }
    Ok(())
}

fn apply_set_column_width(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let col = try_value_as_u16(&op["col"], "col")?;
    let width = value_as_f64(&op["width"]).unwrap_or(8.43);
    ws.set_column_width(col, width)
        .map(|_| ())
        .map_err(|_| "Erreur largeur colonne".to_string())
}

pub(super) fn resolve_cell_position(op: &Value) -> Result<(u32, u16), String> {
    if let Some(cell_ref) = op["cell"].as_str() {
        parse_cell_ref(cell_ref).ok_or_else(|| "Référence de cellule invalide".to_string())
    } else {
        let row = try_value_as_u32(&op["row"], "row")?;
        let col = try_value_as_u16(&op["col"], "col")?;
        Ok((row, col))
    }
}

pub fn write_value(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    val: &Value,
) -> Result<(), String> {
    match val {
        Value::String(s) => {
            if let Ok(n) = s.parse::<f64>() {
                ws.write_number(row, col, n)
            } else if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
                ws.write_boolean(row, col, s.eq_ignore_ascii_case("true"))
            } else {
                ws.write_string(row, col, s)
            }
        }
        Value::Number(n) => ws.write_number(row, col, n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => ws.write_boolean(row, col, *b),
        Value::Null => return Ok(()),
        _ => ws.write_string(row, col, val.to_string()),
    }
    .map(|_| ())
    .map_err(|_| "Erreur écriture cellule".to_string())
}
