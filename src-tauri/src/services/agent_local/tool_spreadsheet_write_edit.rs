use crate::services::agent_local::tool_office_utils::{
    coerce_values_array, normalize_formula, try_value_as_u32, value_as_f64,
};
use crate::services::agent_local::tool_spreadsheet_write::parse_cell_ref;
use crate::services::agent_local::tool_spreadsheet_write_format::{
    apply_merge_cells, apply_set_border, apply_set_format, apply_set_number_format,
    apply_set_row_height,
};
use serde_json::Value;
use std::path::Path;

pub fn edit_xlsx(path: &Path, ops: &[Value]) -> Result<(), String> {
    super::tool_spreadsheet_write::validate_spreadsheet_input(path)?;
    let mut book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|_| "Impossible d'ouvrir le fichier xlsx".to_string())?;

    for op in ops {
        let op_type = op["type"].as_str().unwrap_or("");
        match op_type {
            "set_cell" => apply_set_cell(&mut book, op)?,
            "set_formula" => apply_set_formula(&mut book, op)?,
            "set_row" => apply_set_row(&mut book, op)?,
            "add_sheet" => {
                let name = op["name"].as_str().unwrap_or("Sheet");
                book.new_sheet(name)
                    .map_err(|_| "Erreur création feuille".to_string())?;
            }
            "set_column_width" => apply_set_column_width(&mut book, op)?,
            "set_format" => apply_set_format(&mut book, op)?,
            "set_number_format" => apply_set_number_format(&mut book, op)?,
            "set_border" => apply_set_border(&mut book, op)?,
            "merge_cells" => apply_merge_cells(&mut book, op)?,
            "set_row_height" => apply_set_row_height(&mut book, op)?,
            _ => return Err(format!("Opération inconnue: {op_type}")),
        }
    }

    umya_spreadsheet::writer::xlsx::write(&book, path)
        .map_err(|_| "Impossible de sauvegarder le fichier xlsx".to_string())
}

pub(super) fn resolve_sheet_name(book: &umya_spreadsheet::Workbook, op: &Value) -> String {
    op["sheet"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            book.sheet(0)
                .map(|sheet| sheet.name().to_string())
                .unwrap_or_else(|_| "Sheet1".to_string())
        })
}

fn apply_set_cell(book: &mut umya_spreadsheet::Workbook, op: &Value) -> Result<(), String> {
    let (col_1b, row_1b) = resolve_col_row_1based(op)?;
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;
    let cell = sheet.cell_mut((col_1b, row_1b));
    set_cell_value(cell, &op["value"]);
    Ok(())
}

fn apply_set_formula(book: &mut umya_spreadsheet::Workbook, op: &Value) -> Result<(), String> {
    let (col_1b, row_1b) = resolve_col_row_1based(op)?;
    let raw = op["formula"].as_str().unwrap_or("").to_string();
    let formula = normalize_formula(&raw);
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;
    sheet.cell_mut((col_1b, row_1b)).set_formula(&formula);
    Ok(())
}

fn apply_set_row(book: &mut umya_spreadsheet::Workbook, op: &Value) -> Result<(), String> {
    let row_idx = try_value_as_u32(&op["row"], "row")?;
    let row_1based = row_idx + 1;
    let values = match coerce_values_array(&op["values"]) {
        Some(v) => v,
        None => return Ok(()),
    };
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;

    for (col_idx, val) in values.iter().enumerate() {
        let col_1based = u32::try_from(col_idx)
            .map_err(|_| "col trop grand".to_string())?
            .saturating_add(1);
        let cell = sheet.cell_mut((col_1based, row_1based));
        set_cell_value(cell, val);
    }
    Ok(())
}

fn apply_set_column_width(
    book: &mut umya_spreadsheet::Workbook,
    op: &Value,
) -> Result<(), String> {
    let col_idx = try_value_as_u32(&op["col"], "col")?;
    let col_1based = col_idx + 1;
    let width = value_as_f64(&op["width"]).unwrap_or(8.43);
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;
    sheet
        .column_dimension_by_number_mut(col_1based)
        .set_width(width);
    Ok(())
}

pub(super) fn resolve_col_row_1based(op: &Value) -> Result<(u32, u32), String> {
    if let Some(cell_str) = op["cell"].as_str() {
        let (row_0b, col_0b) = parse_cell_ref(cell_str).ok_or("Référence de cellule invalide")?;
        Ok((col_0b as u32 + 1, row_0b + 1))
    } else {
        let row = try_value_as_u32(&op["row"], "row")?;
        let col = try_value_as_u32(&op["col"], "col")?;
        Ok((col + 1, row + 1))
    }
}

pub(super) fn set_cell_value(cell: &mut umya_spreadsheet::Cell, val: &Value) {
    match val {
        Value::String(s) => {
            if let Ok(n) = s.parse::<f64>() {
                cell.set_value_number(n);
            } else if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
                cell.set_value_bool(s.eq_ignore_ascii_case("true"));
            } else {
                cell.set_value(s);
            }
        }
        Value::Number(n) => {
            cell.set_value_number(n.as_f64().unwrap_or(0.0));
        }
        Value::Bool(b) => {
            cell.set_value_bool(*b);
        }
        Value::Null => {}
        _ => {
            cell.set_value(val.to_string());
        }
    }
}
