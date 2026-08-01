use crate::services::agent_local::tool_office_utils::{
    border_style_name, try_value_as_u32, validate_color_hex, value_as_f64,
};
use rust_xlsxwriter::{Color, Format, FormatBorder, Worksheet};
use serde_json::Value;

pub(super) fn apply_set_format(ws: &mut Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = super::tool_spreadsheet_write_new::resolve_cell_position(op)?;
    let format = build_format(op)?;
    if op["value"].is_null() {
        return ws
            .write_blank(row, col, &format)
            .map(|_| ())
            .map_err(|_| "Erreur application format".to_string());
    }
    let value = &op["value"];
    match value {
        Value::String(text) => {
            if let Ok(number) = text.parse::<f64>() {
                ws.write_with_format(row, col, number, &format)
            } else if text.eq_ignore_ascii_case("true") || text.eq_ignore_ascii_case("false") {
                ws.write_with_format(row, col, text.eq_ignore_ascii_case("true"), &format)
            } else {
                ws.write_with_format(row, col, text, &format)
            }
        }
        Value::Number(number) => {
            ws.write_with_format(row, col, number.as_f64().unwrap_or(0.0), &format)
        }
        Value::Bool(value) => ws.write_with_format(row, col, *value, &format),
        _ => ws.write_with_format(row, col, value.to_string(), &format),
    }
    .map(|_| ())
    .map_err(|_| "Erreur application format".to_string())
}

pub(super) fn apply_set_number_format(ws: &mut Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = super::tool_spreadsheet_write_new::resolve_cell_position(op)?;
    let number_format = op["number_format"]
        .as_str()
        .ok_or_else(|| "number_format requis".to_string())?;
    let format = Format::new().set_num_format(number_format);
    let result = if op["value"].is_null() {
        ws.write_blank(row, col, &format)
    } else {
        match &op["value"] {
            Value::String(text) => match text.parse::<f64>() {
                Ok(number) => ws.write_number_with_format(row, col, number, &format),
                Err(_) => ws.write_string_with_format(row, col, text, &format),
            },
            Value::Number(number) => ws.write_number_with_format(
                row,
                col,
                number.as_f64().unwrap_or(0.0),
                &format,
            ),
            _ => ws.write_blank(row, col, &format),
        }
    };
    result
        .map(|_| ())
        .map_err(|_| "Erreur format nombre".to_string())
}

pub(super) fn apply_set_border(ws: &mut Worksheet, op: &Value) -> Result<(), String> {
    let (row, col) = super::tool_spreadsheet_write_new::resolve_cell_position(op)?;
    let format = build_border_format(op)?;
    ws.write_blank(row, col, &format)
        .map(|_| ())
        .map_err(|_| "Erreur application bordure".to_string())
}

pub(super) fn apply_merge_cells(ws: &mut Worksheet, op: &Value) -> Result<(), String> {
    let start = op["start_cell"]
        .as_str()
        .ok_or_else(|| "start_cell requis".to_string())?;
    let end = op["end_cell"]
        .as_str()
        .ok_or_else(|| "end_cell requis".to_string())?;
    let (first_row, first_col) = super::tool_spreadsheet_write::parse_cell_ref(start)
        .ok_or_else(|| "Référence start_cell invalide".to_string())?;
    let (last_row, last_col) = super::tool_spreadsheet_write::parse_cell_ref(end)
        .ok_or_else(|| "Référence end_cell invalide".to_string())?;
    ws.merge_range(
        first_row,
        first_col,
        last_row,
        last_col,
        "",
        &Format::new(),
    )
    .map(|_| ())
    .map_err(|_| "Erreur fusion cellules".to_string())
}

pub(super) fn apply_set_row_height(ws: &mut Worksheet, op: &Value) -> Result<(), String> {
    let row = try_value_as_u32(&op["row"], "row")?;
    let height = value_as_f64(&op["height"]).ok_or_else(|| "height requis".to_string())?;
    ws.set_row_height(row, height)
        .map(|_| ())
        .map_err(|_| "Erreur hauteur ligne".to_string())
}

fn build_format(op: &Value) -> Result<Format, String> {
    let mut format = Format::new();
    if op["bold"].as_bool().unwrap_or(false) {
        format = format.set_bold();
    }
    if op["italic"].as_bool().unwrap_or(false) {
        format = format.set_italic();
    }
    if op["underline"].as_bool().unwrap_or(false) {
        format = format.set_underline(rust_xlsxwriter::FormatUnderline::Single);
    }
    if let Some(hex) = validate_color_hex(&op["font_color"], "font_color")? {
        format = format.set_font_color(Color::from(hex.as_str()));
    }
    if let Some(hex) = validate_color_hex(&op["bg_color"], "bg_color")? {
        format = format.set_background_color(Color::from(hex.as_str()));
    }
    if let Some(size) = value_as_f64(&op["font_size"]) {
        format = format.set_font_size(size);
    }
    Ok(format)
}

fn build_border_format(op: &Value) -> Result<Format, String> {
    let border = match border_style_name(&op["border_style"])? {
        "medium" => FormatBorder::Medium,
        "thick" => FormatBorder::Thick,
        _ => FormatBorder::Thin,
    };
    let Some(sides) = super::tool_spreadsheet_border::parse(&op["border_sides"])? else {
        return Ok(Format::new().set_border(border));
    };
    if sides.is_empty() {
        return Ok(Format::new().set_border(border));
    }
    let mut format = Format::new();
    if sides.top {
        format = format.set_border_top(border);
    }
    if sides.bottom {
        format = format.set_border_bottom(border);
    }
    if sides.left {
        format = format.set_border_left(border);
    }
    if sides.right {
        format = format.set_border_right(border);
    }
    Ok(format)
}
