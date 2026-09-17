#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_document_write::write_document;
    use tempfile::TempDir;

    fn working_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn write_docx_heading() {
        let tmp = working_dir();
        let path = tmp.path().join("test.docx");
        let content = serde_json::json!([
            { "type": "heading", "text": "Mon titre", "level": 1 }
        ]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        assert!(path.exists());
        // Vérifier que c'est un ZIP valide contenant word/document.xml
        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("word/document.xml").is_ok());
    }

    #[tokio::test]
    async fn write_docx_paragraph_bold() {
        let tmp = working_dir();
        let path = tmp.path().join("test.docx");
        let content = serde_json::json!([
            { "type": "paragraph", "text": "Texte en gras", "bold": true }
        ]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        // Lire le XML et vérifier que <w:b/> est présent
        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut doc = archive.by_name("word/document.xml").unwrap();
        let mut xml = String::new();
        use std::io::Read;
        doc.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("w:b"));
        assert!(xml.contains("Texte en gras"));
    }

    #[tokio::test]
    async fn write_docx_table() {
        let tmp = working_dir();
        let path = tmp.path().join("test.docx");
        let content = serde_json::json!([
            { "type": "table", "headers": ["Nom", "Age"], "rows": [["Alice", "30"], ["Bob", "25"]] }
        ]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        assert!(path.exists());
    }

    #[tokio::test]
    async fn write_invalid_path() {
        let tmp = working_dir();
        let content = serde_json::json!([{ "type": "paragraph", "text": "test" }]);
        let result = write_document("/nonexistent/dir/test.docx", &content, tmp.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn write_docx_list_ordered() {
        let tmp = working_dir();
        let path = tmp.path().join("test.docx");
        let content = serde_json::json!([
            { "type": "list", "items": ["Premier", "Deuxième", "Troisième"], "ordered": true }
        ]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut doc = archive.by_name("word/document.xml").unwrap();
        let mut xml = String::new();
        use std::io::Read;
        doc.read_to_string(&mut xml).unwrap();
        // Vraie numérotation OOXML : référence au numId 1 (ordered)
        assert!(xml.contains("<w:numPr>"), "devrait utiliser w:numPr: {xml}");
        assert!(
            xml.contains("w:val=\"1\""),
            "devrait référencer numId 1 (ordered)"
        );
        assert!(xml.contains("Premier"));
    }

    #[tokio::test]
    async fn write_docx_list_unordered() {
        let tmp = working_dir();
        let path = tmp.path().join("test.docx");
        let content = serde_json::json!([
            { "type": "list", "items": ["Item A", "Item B"], "ordered": false }
        ]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut doc = archive.by_name("word/document.xml").unwrap();
        let mut xml = String::new();
        use std::io::Read;
        doc.read_to_string(&mut xml).unwrap();
        // Vraie numérotation OOXML : référence au numId 2 (bullet)
        assert!(xml.contains("<w:numPr>"), "devrait utiliser w:numPr: {xml}");
        assert!(
            xml.contains("w:val=\"2\""),
            "devrait référencer numId 2 (bullet)"
        );
        assert!(xml.contains("Item A"));
    }

    #[tokio::test]
    async fn reject_non_docx_extension() {
        let tmp = working_dir();
        let path = tmp.path().join("test.txt");
        let content = serde_json::json!([{ "type": "paragraph", "text": "test" }]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn write_docx_multiple_blocks() {
        let tmp = working_dir();
        let path = tmp.path().join("test.docx");
        let content = serde_json::json!([
            { "type": "heading", "text": "Titre principal", "level": 1 },
            { "type": "paragraph", "text": "Introduction", "bold": false, "italic": false },
            { "type": "paragraph", "text": "Texte italique", "italic": true },
            { "type": "table", "headers": ["Col1", "Col2"], "rows": [["v1", "v2"]] },
            { "type": "list", "items": ["a", "b"], "ordered": false }
        ]);
        let result = write_document(path.to_str().unwrap(), &content, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        assert!(result.content.contains("5 blocs"));
    }
}
