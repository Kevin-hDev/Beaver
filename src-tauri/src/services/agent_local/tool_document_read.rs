use crate::services::agent_local::security::validate_read_path;
use crate::services::agent_local::tool_office_limits::{
    ensure_source_size, ensure_zip_entry_safe, validate_zip_archive, MAX_DOCX_SOURCE_BYTES,
    MAX_DOCX_XML_BYTES,
};
use crate::services::agent_local::types_tools::ToolResult;
use std::io::Read;
use std::path::Path;

const MAX_EXTRACTED_DOC_CHARS: usize = 1_000_000;

pub async fn read_document(path: &str, _pages: Option<&str>, working_dir: &Path) -> ToolResult {
    if path.is_empty() {
        return ToolResult::err("Le paramètre 'path' est requis");
    }

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_read_path(&resolved, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => read_pdf(&validated),
        "docx" => read_docx(&validated),
        _ => ToolResult::err("Format non supporté. Formats acceptés : pdf, docx"),
    }
}

fn read_pdf(path: &Path) -> ToolResult {
    let path_str = match path.to_str() {
        Some(s) => s,
        None => return ToolResult::err("Chemin invalide"),
    };

    let text = match pdf_extract::extract_text(path_str) {
        Ok(t) => t,
        Err(_) => return ToolResult::err("Impossible de lire le fichier PDF"),
    };

    let char_count = text.chars().count();
    let json = serde_json::json!({
        "format": "pdf",
        "text": text,
        "char_count": char_count
    });
    ToolResult::ok(json.to_string())
}

fn read_docx(path: &Path) -> ToolResult {
    if let Err(e) = ensure_source_size(path, MAX_DOCX_SOURCE_BYTES, "Document DOCX") {
        return ToolResult::err(e);
    }
    if let Err(e) = validate_zip_archive(path, "Document DOCX") {
        return ToolResult::err(e);
    }

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return ToolResult::err("Impossible d'ouvrir le fichier"),
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return ToolResult::err("Fichier DOCX invalide ou corrompu"),
    };

    let xml_content = match archive.by_name("word/document.xml") {
        Ok(mut entry) => {
            if let Err(e) = ensure_zip_entry_safe(&entry, MAX_DOCX_XML_BYTES, "XML DOCX") {
                return ToolResult::err(e);
            }
            let mut buf = String::new();
            if entry
                .by_ref()
                .take(MAX_DOCX_XML_BYTES + 1)
                .read_to_string(&mut buf)
                .is_err()
            {
                return ToolResult::err("Impossible de lire le contenu du document");
            }
            if buf.len() as u64 > MAX_DOCX_XML_BYTES {
                return ToolResult::err("XML DOCX trop volumineux");
            }
            buf
        }
        Err(_) => return ToolResult::err("Structure DOCX invalide"),
    };

    let text = match extract_text_from_ooxml(&xml_content) {
        Ok(text) => text,
        Err(e) => return ToolResult::err(e),
    };
    let char_count = text.chars().count();
    let json = serde_json::json!({
        "format": "docx",
        "text": text,
        "char_count": char_count
    });
    ToolResult::ok(json.to_string())
}

fn extract_text_from_ooxml(xml: &str) -> Result<String, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    // IMPORTANT : trim_text désactivé. Sinon les espaces entre runs OOXML
    // (ex: "un " + "<w:r>mot</w:r>" + " et ") sont supprimés et le texte
    // des runs se retrouve collé ("unmot"). On gère le trim au niveau
    // paragraphe final, pas au niveau de chaque événement texte.
    reader.config_mut().trim_text(false);

    let mut result = String::new();
    let mut in_paragraph = false;
    let mut para_text = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                if e.local_name().as_ref() == b"p" {
                    in_paragraph = true;
                    para_text.clear();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.local_name().as_ref() == b"p" {
                    // Trim + collapse des espaces multiples (issus de l'indentation
                    // XML entre runs) en un seul espace. Préserve les espaces
                    // significatifs entre runs ("un " + "mot" = "un mot").
                    let normalized: String =
                        para_text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !normalized.is_empty() {
                        if !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str(&normalized);
                        if result.chars().count() > MAX_EXTRACTED_DOC_CHARS {
                            return Err("Document trop volumineux".to_string());
                        }
                    }
                    in_paragraph = false;
                    para_text.clear();
                }
            }
            Ok(Event::Text(ref e)) if in_paragraph => {
                if let Ok(decoded) = e.xml10_content() {
                    if let Ok(unescaped) = quick_xml::escape::unescape(&decoded) {
                        para_text.push_str(&unescaped);
                    }
                }
            }
            Ok(Event::Eof) if in_paragraph || !para_text.is_empty() => {
                return Err("Document XML malformé".to_string());
            }
            Ok(Event::Eof) => break,
            Err(_) => return Err("Document XML malformé".to_string()),
            _ => {}
        }
    }

    Ok(result)
}
