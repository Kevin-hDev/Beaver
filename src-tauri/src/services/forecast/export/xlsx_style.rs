use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, FormatPattern, Worksheet};

pub fn finish_table(ws: &mut Worksheet, last_row: u32, last_col: u16) -> Result<(), String> {
    let header = header_format();
    ws.set_freeze_panes(1, 0)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    ws.set_row_format(0, &header)
        .map_err(|_| "Export XLSX impossible".to_string())?;
    if last_col > 0 {
        ws.autofilter(0, 0, last_row.max(1), last_col)
            .map_err(|_| "Export XLSX impossible".to_string())?;
    }
    ws.autofit_to_max_width(52);
    Ok(())
}

pub fn label_format() -> Format {
    Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xF1F1F4))
        .set_pattern(FormatPattern::Solid)
        .set_border(FormatBorder::Thin)
}

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2A2A2F))
        .set_pattern(FormatPattern::Solid)
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::VerticalCenter)
}
