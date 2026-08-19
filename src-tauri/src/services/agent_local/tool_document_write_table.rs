use super::tool_document_write_run::{write_run, RunStyle};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Cursor;

pub(super) fn write_table(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let headers = block["headers"]
        .as_array()
        .map(|values| values.as_slice())
        .unwrap_or(&[]);
    let rows = block["rows"]
        .as_array()
        .map(|values| values.as_slice())
        .unwrap_or(&[]);

    if headers.is_empty() && rows.is_empty() {
        return Ok(());
    }

    writer
        .write_event(Event::Start(BytesStart::new("w:tbl")))
        .map_err(|error| format!("XML error: {error}"))?;
    if !headers.is_empty() {
        write_table_row(writer, headers, true)?;
    }
    for row in rows {
        if let Some(cells) = row.as_array() {
            write_table_row(writer, cells, false)?;
        }
    }
    writer
        .write_event(Event::End(BytesEnd::new("w:tbl")))
        .map_err(|error| format!("XML error: {error}"))?;
    Ok(())
}

fn write_table_row(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    cells: &[serde_json::Value],
    is_header: bool,
) -> Result<(), String> {
    let style = RunStyle {
        bold: is_header,
        ..Default::default()
    };
    writer
        .write_event(Event::Start(BytesStart::new("w:tr")))
        .map_err(|error| format!("XML error: {error}"))?;
    for cell in cells {
        writer
            .write_event(Event::Start(BytesStart::new("w:tc")))
            .map_err(|error| format!("XML error: {error}"))?;
        writer
            .write_event(Event::Start(BytesStart::new("w:p")))
            .map_err(|error| format!("XML error: {error}"))?;
        write_run(writer, cell.as_str().unwrap_or(""), &style)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:p")))
            .map_err(|error| format!("XML error: {error}"))?;
        writer
            .write_event(Event::End(BytesEnd::new("w:tc")))
            .map_err(|error| format!("XML error: {error}"))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("w:tr")))
        .map_err(|error| format!("XML error: {error}"))?;
    Ok(())
}
