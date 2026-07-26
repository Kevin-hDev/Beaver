use crate::services::agent_local::tool_office_utils::{
    border_style_name, coerce_values_array, normalize_formula, try_value_as_u16, try_value_as_u32,
    validate_color_hex, value_as_f64,
};
use crate::services::agent_local::tool_spreadsheet_write::parse_cell_ref;
use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook};
use serde_json::Value;
use std::path::Path;

pub fn create_xlsx(path: &Path, ops: &[Value]) -> Result<(), String> {
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
            "set_format" => apply_set_format(ws, op)?,
            "set_number_format" => apply_set_number_format(ws, op)?,
            "set_border" => apply_set_border(ws, op)?,
            "merge_cells" => apply_merge_cells(ws, op)?,
            "set_row_height" => apply_set_row_height(ws, op)?,
            "add_sheet" => {}
            _ => return Err(format!("Opération inconnue: {op_type}")),
        }
    }

    workbook
        .save(path)
        .map_err(|_| "Impossible de sauvegarder le fichier xlsx".to_string())
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

/// Construit un Format rust_xlsxwriter à partir des options de set_format.
fn build_format(op: &Value) -> Result<Format, String> {
    let mut fmt = Format::new();
    if op["bold"].as_bool().unwrap_or(false) {
        fmt = fmt.set_bold();
    }
    if op["italic"].as_bool().unwrap_or(false) {
        fmt = fmt.set_italic();
    }
    if op["underline"].as_bool().unwrap_or(false) {
        fmt = fmt.set_underline(rust_xlsxwriter::FormatUnderline::Single);
    }
    if let Some(hex) = validate_color_hex(&op["font_color"], "font_color")? {
        fmt = fmt.set_font_color(Color::from(hex.as_str()));
    }
    if let Some(hex) = validate_color_hex(&op["bg_color"], "bg_color")? {
        fmt = fmt.set_background_color(Color::from(hex.as_str()));
    }
    if let Some(size) = value_as_f64(&op["font_size"]) {
        fmt = fmt.set_font_size(size);
    }
    Ok(fmt)
}

/// Construit un Format bordure à partir de border_style + border_sides.
fn build_border_format(op: &Value) -> Result<Format, String> {
    let style_name = border_style_name(&op["border_style"])?;
    let border = match style_name {
        "thin" => FormatBorder::Thin,
        "medium" => FormatBorder::Medium,
        "thick" => FormatBorder::Thick,
        _ => FormatBorder::Thin,
    };
    let mut fmt = Format::new();
    let sides: Vec<String> = op["border_sides"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_ascii_lowercase()))
                .collect()
        })
        .unwrap_or_default();
    if sides.is_empty() {
        fmt = fmt.set_border(border);
    } else {
        for side in sides {
            match side.as_str() {
                "top" => fmt = fmt.set_border_top(border),
                "bottom" => fmt = fmt.set_border_bottom(border),
                "left" => fmt = fmt.set_border_left(border),
                "right" => fmt = fmt.set_border_right(border),
                _ => {}
            }
        }
    }
    Ok(fmt)
}

fn apply_set_format(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    let fmt = build_format(op)?;
    // Si une value est fournie, on réécrit la cellule avec le format.
    // Sinon on applique le format à la cellule existante via write_blank.
    if op["value"].is_null() {
        ws.write_blank(row, col, &fmt)
            .map(|_| ())
            .map_err(|_| "Erreur application format".to_string())
    } else {
        let val = &op["value"];
        match val {
            Value::String(s) => {
                if let Ok(n) = s.parse::<f64>() {
                    ws.write_with_format(row, col, n, &fmt)
                } else if s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("false") {
                    ws.write_with_format(row, col, s.eq_ignore_ascii_case("true"), &fmt)
                } else {
                    ws.write_with_format(row, col, s, &fmt)
                }
            }
            Value::Number(n) => ws.write_with_format(row, col, n.as_f64().unwrap_or(0.0), &fmt),
            Value::Bool(b) => ws.write_with_format(row, col, *b, &fmt),
            _ => ws.write_with_format(row, col, val.to_string(), &fmt),
        }
        .map(|_| ())
        .map_err(|_| "Erreur application format".to_string())
    }
}

fn apply_set_number_format(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    let number_format = op["number_format"]
        .as_str()
        .ok_or_else(|| "number_format requis".to_string())?;
    let fmt = Format::new().set_num_format(number_format);
    // Réécrit la cellule avec le format nombre si value fournie, sinon blank.
    if op["value"].is_null() {
        ws.write_blank(row, col, &fmt)
    } else {
        let val = &op["value"];
        match val {
            Value::String(s) => match s.parse::<f64>() {
                Ok(n) => ws.write_number_with_format(row, col, n, &fmt),
                Err(_) => ws.write_string_with_format(row, col, s, &fmt),
            },
            Value::Number(n) => {
                ws.write_number_with_format(row, col, n.as_f64().unwrap_or(0.0), &fmt)
            }
            _ => ws.write_blank(row, col, &fmt),
        }
    }
    .map(|_| ())
    .map_err(|_| "Erreur format nombre".to_string())
}

fn apply_set_border(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = resolve_cell_position(op)?;
    let fmt = build_border_format(op)?;
    ws.write_blank(row, col, &fmt)
        .map(|_| ())
        .map_err(|_| "Erreur application bordure".to_string())
}

fn apply_merge_cells(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let start = op["start_cell"]
        .as_str()
        .ok_or_else(|| "start_cell requis".to_string())?;
    let end = op["end_cell"]
        .as_str()
        .ok_or_else(|| "end_cell requis".to_string())?;
    let (first_row, first_col) =
        parse_cell_ref(start).ok_or_else(|| "Référence start_cell invalide".to_string())?;
    let (last_row, last_col) =
        parse_cell_ref(end).ok_or_else(|| "Référence end_cell invalide".to_string())?;
    let fmt = Format::new();
    ws.merge_range(first_row, first_col, last_row, last_col, "", &fmt)
        .map(|_| ())
        .map_err(|_| "Erreur fusion cellules".to_string())
}

fn apply_set_row_height(ws: &mut rust_xlsxwriter::Worksheet, op: &Value) -> Result<(), String> {
    let row = try_value_as_u32(&op["row"], "row")?;
    let height = value_as_f64(&op["height"]).ok_or_else(|| "height requis".to_string())?;
    ws.set_row_height(row, height)
        .map(|_| ())
        .map_err(|_| "Erreur hauteur ligne".to_string())
}

fn resolve_cell_position(op: &Value) -> Result<(u32, u16), String> {
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
