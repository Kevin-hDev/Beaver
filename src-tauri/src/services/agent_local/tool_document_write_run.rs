use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Cursor;

#[derive(Clone, Default)]
/// Style appliqué à un run (segment de texte).
pub(super) struct RunStyle {
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) underline: bool,
    /// Couleur hex RRGGBB. Le caller reste l'autorité de validation.
    pub(super) color: Option<String>,
}

impl RunStyle {
    /// Évite de produire un bloc de propriétés OOXML vide.
    fn has_any(&self) -> bool {
        self.bold || self.italic || self.underline || self.color.is_some()
    }
}

/// Extrait un style borné depuis l'objet JSON d'un segment.
pub(super) fn parse_run_style(run: &serde_json::Value) -> Result<RunStyle, String> {
    let color = super::tool_office_utils::validate_color_hex(&run["color"], "color")?;
    Ok(RunStyle {
        bold: run["bold"].as_bool().unwrap_or(false),
        italic: run["italic"].as_bool().unwrap_or(false),
        underline: run["underline"].as_bool().unwrap_or(false),
        color,
    })
}

pub(super) fn write_run(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    text: &str,
    style: &RunStyle,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("w:r")))
        .map_err(|e| format!("XML error: {e}"))?;

    if style.has_any() {
        write_properties(writer, style)?;
    }

    // OOXML exige cet attribut pour conserver les espaces de début et de fin.
    let mut text_start = BytesStart::new("w:t");
    text_start.push_attribute(("xml:space", "preserve"));
    writer
        .write_event(Event::Start(text_start))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::End(BytesEnd::new("w:t")))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::End(BytesEnd::new("w:r")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

fn write_properties(writer: &mut Writer<Cursor<Vec<u8>>>, style: &RunStyle) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("w:rPr")))
        .map_err(|e| format!("XML error: {e}"))?;
    if style.bold {
        writer
            .write_event(Event::Empty(BytesStart::new("w:b")))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    if style.italic {
        writer
            .write_event(Event::Empty(BytesStart::new("w:i")))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    if style.underline {
        let mut underline = BytesStart::new("w:u");
        underline.push_attribute(("w:val", "single"));
        writer
            .write_event(Event::Empty(underline))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    if let Some(color) = &style.color {
        let mut color_element = BytesStart::new("w:color");
        color_element.push_attribute(("w:val", color.as_str()));
        writer
            .write_event(Event::Empty(color_element))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("w:rPr")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}
