#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_document_write::write_document;
    use serde_json::json;
    use std::io::Read;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    /// Lit une entrée du ZIP DOCX en String.
    fn read_zip_entry(path: &std::path::Path, entry: &str) -> String {
        let file = std::fs::File::open(path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut buf = String::new();
        archive
            .by_name(entry)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        buf
    }

    #[tokio::test]
    async fn write_paragraph_with_runs() {
        let dir = tmp();
        let path = dir.path().join("runs.docx");
        let content = json!([{
            "type": "paragraph",
            "runs": [
                {"text": "Hello "},
                {"text": "world", "bold": true}
            ]
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        // 2 runs séparés
        assert!(
            xml.matches("<w:r>").count() == 2,
            "devrait avoir 2 runs: {xml}"
        );
        assert!(xml.contains("Hello"));
        assert!(xml.contains("world"));
        assert!(xml.contains("w:b"), "le 2e run devrait être en gras");
    }

    #[tokio::test]
    async fn write_paragraph_with_underline() {
        let dir = tmp();
        let path = dir.path().join("underline.docx");
        let content = json!([{
            "type": "paragraph",
            "runs": [{"text": "souligné", "underline": true}]
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(xml.contains("<w:u "), "underline absent");
        assert!(xml.contains("w:val=\"single\""));
    }

    #[tokio::test]
    async fn write_paragraph_with_color() {
        let dir = tmp();
        let path = dir.path().join("color.docx");
        let content = json!([{
            "type": "paragraph",
            "runs": [{"text": "rouge", "color": "FF0000"}]
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(
            xml.contains("w:color") && xml.contains("FF0000"),
            "couleur absente: {xml}"
        );
    }

    #[tokio::test]
    async fn write_paragraph_color_invalid() {
        let dir = tmp();
        let path = dir.path().join("badcolor.docx");
        let content = json!([{
            "type": "paragraph",
            "runs": [{"text": "x", "color": "ZZZZZZ"}]
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(result.is_error, "devrait échouer sur couleur invalide");
    }

    #[tokio::test]
    async fn write_paragraph_align_center() {
        let dir = tmp();
        let path = dir.path().join("align.docx");
        let content = json!([{"type": "paragraph", "text": "centré", "align": "center"}]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(xml.contains("<w:jc "), "alignement absent");
        assert!(xml.contains("w:val=\"center\""));
    }

    #[tokio::test]
    async fn write_heading_align_right() {
        let dir = tmp();
        let path = dir.path().join("halign.docx");
        let content = json!([{
            "type": "heading",
            "text": "titre droite",
            "level": 2,
            "align": "right"
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(xml.contains("Heading2"));
        assert!(xml.contains("w:val=\"right\""));
    }

    #[tokio::test]
    async fn write_docx_has_styles_xml() {
        let dir = tmp();
        let path = dir.path().join("styles.docx");
        let content = json!([{"type": "paragraph", "text": "x"}]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let styles = read_zip_entry(&path, "word/styles.xml");
        assert!(styles.contains("Heading1"), "Heading1 absent de styles.xml");
        assert!(styles.contains("Heading6"), "Heading6 absent de styles.xml");
        assert!(styles.contains("Normal"));
    }

    #[tokio::test]
    async fn write_docx_has_numbering_xml() {
        let dir = tmp();
        let path = dir.path().join("numbering.docx");
        let content = json!([{"type": "paragraph", "text": "x"}]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let numbering = read_zip_entry(&path, "word/numbering.xml");
        // numId=1 (ordered) + numId=2 (bullet)
        assert!(
            numbering.contains("w:numId=\"1\"") || numbering.contains("<w:num w:numId=\"1\""),
            "numId 1 absent"
        );
        assert!(
            numbering.contains("w:numId=\"2\"") || numbering.contains("<w:num w:numId=\"2\""),
            "numId 2 absent"
        );
        assert!(numbering.contains("decimal"), "format decimal absent");
        assert!(numbering.contains("bullet"), "format bullet absent");
    }

    #[tokio::test]
    async fn write_list_uses_real_numbering() {
        let dir = tmp();
        let path = dir.path().join("real_list.docx");
        let content = json!([{
            "type": "list",
            "items": ["Premier", "Deuxième"],
            "ordered": true
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        // Vraie numérotation : <w:numPr> + <w:numId w:val="1"/> (ordered)
        assert!(
            xml.contains("<w:numPr>"),
            "devrait utiliser w:numPr (vraie numérotation): {xml}"
        );
        assert!(
            xml.contains("w:val=\"1\""),
            "devrait référencer numId 1 (ordered)"
        );
        // L'ancien préfixe texte "1. " ne doit plus être présent
        assert!(
            !xml.contains(">1. Premier<"),
            "l'ancien préfixe texte ne devrait plus être utilisé"
        );
    }

    #[tokio::test]
    async fn write_list_bullet_uses_numid_2() {
        let dir = tmp();
        let path = dir.path().join("bullet.docx");
        let content = json!([{
            "type": "list",
            "items": ["a", "b"],
            "ordered": false
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(
            xml.contains("w:val=\"2\""),
            "devrait référencer numId 2 (bullet)"
        );
    }

    #[tokio::test]
    async fn write_paragraph_text_still_works() {
        // Rétro-compat : un bloc avec juste 'text' (sans 'runs') doit marcher.
        let dir = tmp();
        let path = dir.path().join("legacy.docx");
        let content = json!([{
            "type": "paragraph",
            "text": "hello legacy",
            "bold": true
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(xml.contains("hello legacy"));
        assert!(xml.contains("w:b"));
    }

    #[tokio::test]
    async fn write_docx_has_document_rels() {
        let dir = tmp();
        let path = dir.path().join("rels.docx");
        let content = json!([{"type": "paragraph", "text": "x"}]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let rels = read_zip_entry(&path, "word/_rels/document.xml.rels");
        assert!(rels.contains("styles.xml"), "relationship styles manquant");
        assert!(
            rels.contains("numbering.xml"),
            "relationship numbering manquant"
        );
    }

    /// Régression #3 : un paragraphe avec align + italic (sans runs) doit
    /// quand même écrire son texte. Avant le fix, le texte était perdu.
    #[tokio::test]
    async fn write_paragraph_align_italic_writes_text() {
        let dir = tmp();
        let path = dir.path().join("align_italic.docx");
        let content = json!([{
            "type": "paragraph",
            "text": "Paragraphe aligné à droite",
            "italic": true,
            "align": "right"
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(
            xml.contains("Paragraphe aligné"),
            "le texte devrait être présent: {xml}"
        );
        assert!(xml.contains("w:i"), "italique devrait être présent");
        assert!(xml.contains("w:val=\"right\""), "alignement right absent");
    }

    /// Régression #1 : runs vides ne doit pas faire perdre le texte de fallback.
    #[tokio::test]
    async fn write_paragraph_empty_runs_fallback_text() {
        let dir = tmp();
        let path = dir.path().join("empty_runs.docx");
        let content = json!([{
            "type": "paragraph",
            "text": "texte de secours",
            "runs": []
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(
            xml.contains("texte de secours"),
            "le texte de fallback devrait être présent: {xml}"
        );
    }

    /// Régression #1bis : couleur vide doit être ignorée (pas d'erreur).
    #[tokio::test]
    async fn write_paragraph_empty_color_ignored() {
        let dir = tmp();
        let path = dir.path().join("empty_color.docx");
        let content = json!([{
            "type": "paragraph",
            "runs": [{"text": "ok", "color": ""}]
        }]);
        let result = write_document(path.to_str().unwrap(), &content, dir.path()).await;
        assert!(
            !result.is_error,
            "couleur vide ne devrait pas échouer: {}",
            result.content
        );
        let xml = read_zip_entry(&path, "word/document.xml");
        assert!(xml.contains("ok"));
    }
}
