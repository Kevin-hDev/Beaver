#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_document_read::read_document;
    use tempfile::TempDir;

    fn working_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn read_docx_basic() {
        let tmp = working_dir();
        let docx_path = tmp.path().join("test.docx");

        let file = std::fs::File::create(&docx_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("word/document.xml", options).unwrap();
        use std::io::Write;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Hello World</w:t></w:r></w:p>
    <w:p><w:r><w:t>Second paragraph</w:t></w:r></w:p>
  </w:body>
</w:document>"#,
        )
        .unwrap();
        zip.finish().unwrap();

        let result = read_document(docx_path.to_str().unwrap(), None, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        let text = json["text"].as_str().unwrap();
        assert!(text.contains("Hello World"), "text: {text}");
        assert!(text.contains("Second paragraph"), "text: {text}");
    }

    #[tokio::test]
    async fn read_docx_char_count() {
        let tmp = working_dir();
        let docx_path = tmp.path().join("count.docx");

        let file = std::fs::File::create(&docx_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("word/document.xml", options).unwrap();
        use std::io::Write;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>ABC</w:t></w:r></w:p>
  </w:body>
</w:document>"#,
        )
        .unwrap();
        zip.finish().unwrap();

        let result = read_document(docx_path.to_str().unwrap(), None, tmp.path()).await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["format"], "docx");
        assert!(json["char_count"].as_u64().unwrap() >= 3);
    }

    #[tokio::test]
    async fn read_docx_rejects_malformed_xml() {
        let tmp = working_dir();
        let docx_path = tmp.path().join("bad.docx");
        let file = std::fs::File::create(&docx_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        use std::io::Write;
        zip.write_all(br#"<w:document><w:p><w:t>partial"#).unwrap();
        zip.finish().unwrap();

        let result = read_document(docx_path.to_str().unwrap(), None, tmp.path()).await;
        assert!(result.is_error);
        assert!(result.content.contains("malformé"));
    }

    #[tokio::test]
    async fn read_invalid_path() {
        let tmp = working_dir();
        let result = read_document("/nonexistent/file.pdf", None, tmp.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn read_unsupported_format() {
        let tmp = working_dir();
        let path = tmp.path().join("test.odt");
        std::fs::write(&path, "hello").unwrap();
        let result = read_document(path.to_str().unwrap(), None, tmp.path()).await;
        assert!(result.is_error);
        assert!(
            result.content.contains("Format non supporté"),
            "content: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_empty_path() {
        let tmp = working_dir();
        let result = read_document("", None, tmp.path()).await;
        assert!(result.is_error);
    }

    /// Régression : les runs multiples dans un paragraphe ne doivent pas être
    /// collés sans espaces. Avant le fix, "un " + "mot" donnait "unmot".
    #[tokio::test]
    async fn read_docx_preserves_spaces_between_runs() {
        let tmp = working_dir();
        let docx_path = tmp.path().join("runs.docx");

        let file = std::fs::File::create(&docx_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("word/document.xml", options).unwrap();
        use std::io::Write;
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:r><w:t xml:space="preserve">Cette phrase contient un </w:t></w:r>
      <w:r><w:rPr><w:b/></w:rPr><w:t>mot rouge</w:t></w:r>
      <w:r><w:t xml:space="preserve"> et un </w:t></w:r>
      <w:r><w:rPr><w:b/><w:color w:val="2F5496"/></w:rPr><w:t>segment bleu</w:t></w:r>
      <w:r><w:t xml:space="preserve"> soulign&#233;.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#,
        )
        .unwrap();
        zip.finish().unwrap();

        let result = read_document(docx_path.to_str().unwrap(), None, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        let text = json["text"].as_str().unwrap();
        // Le bug collait : "unmot" au lieu de "un mot"
        assert!(
            text.contains("un mot rouge"),
            "espaces entre runs perdus: {text}"
        );
        assert!(
            text.contains("et un segment"),
            "espaces entre runs perdus: {text}"
        );
    }
}
