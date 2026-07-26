use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Cursor;

/// Style appliqué à un run (segment de texte).
#[derive(Clone, Default)]
pub struct RunStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// Couleur hex RRGGBB (ex: "FF0000"). Aucune validation ici — le caller valide.
    pub color: Option<String>,
}

impl RunStyle {
    /// True si au moins un attribut de style est défini.
    fn has_any(&self) -> bool {
        self.bold || self.italic || self.underline || self.color.is_some()
    }
}

/// Extrait un RunStyle depuis un objet JSON de run.
fn parse_run_style(run: &serde_json::Value) -> Result<RunStyle, String> {
    let color = super::tool_office_utils::validate_color_hex(&run["color"], "color")?;
    Ok(RunStyle {
        bold: run["bold"].as_bool().unwrap_or(false),
        italic: run["italic"].as_bool().unwrap_or(false),
        underline: run["underline"].as_bool().unwrap_or(false),
        color,
    })
}

/// Génère le XML OOXML pour word/document.xml à partir d'une liste de blocs JSON.
pub fn build_document_xml(blocks: &[serde_json::Value]) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))
        .map_err(|e| format!("XML error: {e}"))?;

    let mut doc_start = BytesStart::new("w:document");
    doc_start.push_attribute((
        "xmlns:w",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    ));
    writer
        .write_event(Event::Start(doc_start))
        .map_err(|e| format!("XML error: {e}"))?;

    writer
        .write_event(Event::Start(BytesStart::new("w:body")))
        .map_err(|e| format!("XML error: {e}"))?;

    for block in blocks {
        let block_type = block["type"].as_str().unwrap_or("");
        match block_type {
            "heading" => write_heading(&mut writer, block)?,
            "paragraph" => write_paragraph(&mut writer, block)?,
            "table" => write_table(&mut writer, block)?,
            "list" => super::tool_document_write_list::write_list(&mut writer, block)?,
            _ => return Err(format!("Bloc document inconnu: {block_type}")),
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:body")))
        .map_err(|e| format!("XML error: {e}"))?;
    writer
        .write_event(Event::End(BytesEnd::new("w:document")))
        .map_err(|e| format!("XML error: {e}"))?;

    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 error: {e}"))
}

/// Écrit un `<w:pPr><w:jc w:val="..."/></w:pPr>` si un alignement est fourni.
fn write_paragraph_properties(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    align: Option<&str>,
    style: Option<&str>,
) -> Result<(), String> {
    if align.is_none() && style.is_none() {
        return Ok(());
    }
    writer
        .write_event(Event::Start(BytesStart::new("w:pPr")))
        .map_err(|e| format!("XML error: {e}"))?;
    if let Some(s) = style {
        let mut style_elem = BytesStart::new("w:pStyle");
        style_elem.push_attribute(("w:val", s));
        writer
            .write_event(Event::Empty(style_elem))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    if let Some(a) = align {
        let mut jc = BytesStart::new("w:jc");
        jc.push_attribute(("w:val", a));
        writer
            .write_event(Event::Empty(jc))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("w:pPr")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

fn write_heading(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let text = block["text"].as_str().unwrap_or("");
    let level = block["level"].as_u64().unwrap_or(1).clamp(1, 6);
    let style = format!("Heading{level}");
    let align = parse_align(&block["align"]);

    writer
        .write_event(Event::Start(BytesStart::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    write_paragraph_properties(writer, align, Some(style.as_str()))?;
    write_run(writer, text, &RunStyle::default())?;
    writer
        .write_event(Event::End(BytesEnd::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

/// Extrait l'alignement valide (left|center|right|justify) ou None.
fn parse_align(val: &serde_json::Value) -> Option<&str> {
    match val.as_str().map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("left") => Some("left"),
        Some("center") => Some("center"),
        Some("right") => Some("right"),
        Some("justify") => Some("both"),
        _ => None,
    }
}

fn write_paragraph(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let align = parse_align(&block["align"]);

    writer
        .write_event(Event::Start(BytesStart::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    write_paragraph_properties(writer, align, None)?;

    // Si 'runs' est présent ET non vide, on itère sur chaque segment
    // (un <w:r> par segment). Sinon, fallback sur 'text' avec style uniforme
    // (rétro-compatibilité). Un 'runs' vide ne doit pas produire un paragraphe vide.
    let runs = block["runs"].as_array().filter(|a| !a.is_empty());
    if let Some(runs) = runs {
        for run in runs {
            let text = run["text"].as_str().unwrap_or("");
            let style = parse_run_style(run)?;
            write_run(writer, text, &style)?;
        }
    } else {
        let text = block["text"].as_str().unwrap_or("");
        let style = RunStyle {
            bold: block["bold"].as_bool().unwrap_or(false),
            italic: block["italic"].as_bool().unwrap_or(false),
            ..Default::default()
        };
        write_run(writer, text, &style)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("w:p")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}

pub fn write_run(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    text: &str,
    style: &RunStyle,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("w:r")))
        .map_err(|e| format!("XML error: {e}"))?;

    if style.has_any() {
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
            let mut u = BytesStart::new("w:u");
            u.push_attribute(("w:val", "single"));
            writer
                .write_event(Event::Empty(u))
                .map_err(|e| format!("XML error: {e}"))?;
        }
        if let Some(color) = &style.color {
            let mut c = BytesStart::new("w:color");
            c.push_attribute(("w:val", color.as_str()));
            writer
                .write_event(Event::Empty(c))
                .map_err(|e| format!("XML error: {e}"))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("w:rPr")))
            .map_err(|e| format!("XML error: {e}"))?;
    }

    // xml:space="preserve" pour conserver les espaces de début/fin.
    let mut t = BytesStart::new("w:t");
    t.push_attribute(("xml:space", "preserve"));
    writer
        .write_event(Event::Start(t))
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

fn write_table(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    block: &serde_json::Value,
) -> Result<(), String> {
    let headers = block["headers"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    let rows = block["rows"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);

    if headers.is_empty() && rows.is_empty() {
        return Ok(());
    }

    writer
        .write_event(Event::Start(BytesStart::new("w:tbl")))
        .map_err(|e| format!("XML error: {e}"))?;

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
        .map_err(|e| format!("XML error: {e}"))?;
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
        .map_err(|e| format!("XML error: {e}"))?;
    for cell in cells {
        let text = cell.as_str().unwrap_or("");
        writer
            .write_event(Event::Start(BytesStart::new("w:tc")))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::Start(BytesStart::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
        write_run(writer, text, &style)?;
        writer
            .write_event(Event::End(BytesEnd::new("w:p")))
            .map_err(|e| format!("XML error: {e}"))?;
        writer
            .write_event(Event::End(BytesEnd::new("w:tc")))
            .map_err(|e| format!("XML error: {e}"))?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("w:tr")))
        .map_err(|e| format!("XML error: {e}"))?;
    Ok(())
}
