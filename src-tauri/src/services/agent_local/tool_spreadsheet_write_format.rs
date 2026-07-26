use super::tool_spreadsheet_write_edit::{
    resolve_col_row_1based, resolve_sheet_name, set_cell_value,
};
use super::tool_office_utils::{
    border_style_name, try_value_as_u32, validate_color_hex, value_as_f64,
};
use serde_json::Value;

pub(super) fn apply_set_format(
    book: &mut umya_spreadsheet::Workbook,
    op: &Value,
) -> Result<(), String> {
    let (col, row) = resolve_col_row_1based(op)?;
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;
    if !op["value"].is_null() {
        set_cell_value(sheet.cell_mut((col, row)), &op["value"]);
    }
    let style = sheet.style_mut((col, row));
    if op["bold"].as_bool().unwrap_or(false) {
        style.font_mut().set_bold(true);
    }
    if op["italic"].as_bool().unwrap_or(false) {
        style.font_mut().set_italic(true);
    }
    if op["underline"].as_bool().unwrap_or(false) {
        style.font_mut().set_underline("single");
    }
    if let Some(size) = value_as_f64(&op["font_size"]) {
        style.font_mut().set_size(size);
    }
    if let Some(hex) = validate_color_hex(&op["font_color"], "font_color")? {
        let mut color = umya_spreadsheet::Color::default();
        color.set_argb_str(format!("FF{hex}"));
        style.font_mut().set_color(color);
    }
    if let Some(hex) = validate_color_hex(&op["bg_color"], "bg_color")? {
        style.set_background_color(format!("FF{hex}"));
    }
    Ok(())
}

pub(super) fn apply_set_number_format(
    book: &mut umya_spreadsheet::Workbook,
    op: &Value,
) -> Result<(), String> {
    let (col, row) = resolve_col_row_1based(op)?;
    let number_format = op["number_format"]
        .as_str()
        .ok_or_else(|| "number_format requis".to_string())?;
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;
    if !op["value"].is_null() {
        set_cell_value(sheet.cell_mut((col, row)), &op["value"]);
    }
    let mut format = umya_spreadsheet::NumberingFormat::default();
    format.set_format_code(number_format);
    sheet.style_mut((col, row)).set_number_format(format);
    Ok(())
}

pub(super) fn apply_set_border(
    book: &mut umya_spreadsheet::Workbook,
    op: &Value,
) -> Result<(), String> {
    let (col, row) = resolve_col_row_1based(op)?;
    let style_name = border_style_name(&op["border_style"])?;
    let sheet_name = resolve_sheet_name(book, op);
    let sheet = book
        .sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?;
    let sides = border_sides(op);
    let borders = sheet.style_mut((col, row)).borders_mut();
    for side in sides {
        let mut border = umya_spreadsheet::Border::default();
        border.set_border_style(style_name);
        match side.as_str() {
            "top" => borders.set_top_border(border),
            "bottom" => borders.set_bottom_border(border),
            "left" => borders.set_left_border(border),
            "right" => borders.set_right_border(border),
            _ => borders,
        };
    }
    Ok(())
}

pub(super) fn apply_merge_cells(
    book: &mut umya_spreadsheet::Workbook,
    op: &Value,
) -> Result<(), String> {
    let start = op["start_cell"]
        .as_str()
        .ok_or_else(|| "start_cell requis".to_string())?;
    let end = op["end_cell"]
        .as_str()
        .ok_or_else(|| "end_cell requis".to_string())?;
    let sheet_name = resolve_sheet_name(book, op);
    book.sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?
        .add_merge_cells(format!("{start}:{end}"));
    Ok(())
}

pub(super) fn apply_set_row_height(
    book: &mut umya_spreadsheet::Workbook,
    op: &Value,
) -> Result<(), String> {
    let row = try_value_as_u32(&op["row"], "row")?.saturating_add(1);
    let height = value_as_f64(&op["height"]).ok_or_else(|| "height requis".to_string())?;
    let sheet_name = resolve_sheet_name(book, op);
    book.sheet_by_name_mut(&sheet_name)
        .map_err(|_| "Feuille introuvable")?
        .row_dimension_mut(row)
        .set_height(height);
    Ok(())
}

fn border_sides(op: &Value) -> Vec<String> {
    op["border_sides"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_ascii_lowercase))
                .take(4)
                .collect()
        })
        .unwrap_or_else(|| vec!["top".into(), "bottom".into(), "left".into(), "right".into()])
}
