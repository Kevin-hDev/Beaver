use crate::services::agent_local::security::validate_write_path;
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::io::Write;
use std::path::Path;
use zip::{write::SimpleFileOptions, ZipWriter};

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml"
    ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml"
    ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/word/numbering.xml"
    ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>
</Types>"#;

const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
    Target="word/document.xml"/>
</Relationships>"#;

const DOCUMENT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles"
    Target="styles.xml"/>
  <Relationship Id="rId2"
    Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering"
    Target="numbering.xml"/>
</Relationships>"#;

pub async fn write_document(path: &str, content: &Value, working_dir: &Path) -> ToolResult {
    if path.is_empty() {
        return ToolResult::err("Le paramètre 'path' est requis");
    }

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_write_path(&resolved) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext != "docx" {
        return ToolResult::err("Seul le format .docx est supporté");
    }

    let blocks = match super::tool_spreadsheet_write::coerce_to_array(content) {
        Some(arr) => arr,
        None => return ToolResult::err("Le paramètre 'content' doit être un tableau de blocs"),
    };

    let block_count = blocks.len();

    let document_xml = match super::tool_document_write_xml::build_document_xml(&blocks) {
        Ok(xml) => xml,
        Err(e) => return ToolResult::err(format!("Erreur génération XML: {e}")),
    };

    match write_docx_zip(&validated, &document_xml) {
        Ok(_) => ToolResult::ok(format!(
            "Document écrit: {} ({} blocs)",
            validated.display(),
            block_count
        )),
        Err(e) => ToolResult::err(e),
    }
}

fn write_docx_zip(path: &Path, document_xml: &str) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|_| "Impossible de créer le fichier")?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opts)
        .map_err(|e| format!("ZIP error: {e}"))?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    zip.start_file("_rels/.rels", opts)
        .map_err(|e| format!("ZIP error: {e}"))?;
    zip.write_all(RELS_XML.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    zip.start_file("word/document.xml", opts)
        .map_err(|e| format!("ZIP error: {e}"))?;
    zip.write_all(document_xml.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    // styles.xml — définit les styles Heading1-6 et Normal pour un rendu cohérent
    zip.start_file("word/styles.xml", opts)
        .map_err(|e| format!("ZIP error: {e}"))?;
    let styles_xml = super::tool_document_write_styles::build_styles_xml();
    zip.write_all(styles_xml.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    // numbering.xml — définit les listes ordonnées et à puces (vraie numérotation OOXML)
    zip.start_file("word/numbering.xml", opts)
        .map_err(|e| format!("ZIP error: {e}"))?;
    let numbering_xml = super::tool_document_write_numbering::build_numbering_xml();
    zip.write_all(numbering_xml.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    // word/_rels/document.xml.rels — lie document.xml à styles.xml et numbering.xml
    zip.start_file("word/_rels/document.xml.rels", opts)
        .map_err(|e| format!("ZIP error: {e}"))?;
    zip.write_all(DOCUMENT_RELS_XML.as_bytes())
        .map_err(|e| format!("ZIP write error: {e}"))?;

    zip.finish().map_err(|e| format!("ZIP finish error: {e}"))?;
    Ok(())
}
