use crate::services::agent_local::tool_document_write_xml::{write_run, RunStyle};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Cursor;

/// numId pour les listes ordonnées dans numbering.xml (doit matcher tool_document_write_numbering).
const NUM_ID_ORDERED: &str = "1";
/// numId pour les listes à puces dans numbering.xml.
const NUM_ID_BULLET: &str = "2";

pub fn write_list(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let items = match block["items"].as_array() {
        Some(arr) => arr,
        None => return Ok(()),
    };
    let ordered = block["ordered"].as_bool().unwrap_or(false);
    let num_id = if ordered {
        NUM_ID_ORDERED
    } else {
        NUM_ID_BULLET
    };

    for item in items.iter() {
        let text = item.as_str().unwrap_or("");

        writer
            .write_event(Event::Start(BytesStart::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;

        // Propriétés de paragraphe avec référence à la définition de numérotation.
        writer
            .write_event(Event::Start(BytesStart::new("w:pPr")))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::Start(BytesStart::new("w:numPr")))
            .map_err(|e| format!("XML error: {e}"))?;
        let mut ilvl = BytesStart::new("w:ilvl");
        ilvl.push_attribute(("w:val", "0"));
        writer
            .write_event(Event::Empty(ilvl))
            .map_err(|e| format!("XML error: {e}"))?;
        let mut num_id_elem = BytesStart::new("w:numId");
        num_id_elem.push_attribute(("w:val", num_id));
        writer
            .write_event(Event::Empty(num_id_elem))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::End(BytesEnd::new("w:numPr")))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::End(BytesEnd::new("w:pPr")))
            .map_err(|e| format!("XML error: {e}"))?;

        write_run(writer, text, &RunStyle::default())?;

        writer
            .write_event(Event::End(BytesEnd::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    Ok(())
}
