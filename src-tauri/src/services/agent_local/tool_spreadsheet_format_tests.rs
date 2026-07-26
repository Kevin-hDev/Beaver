#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_spreadsheet_write::write_spreadsheet;
    use serde_json::json;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // Helper : ouvre le xlsx généré et retourne le raw bytes du XML de la feuille
    // pour inspecter le formatage (calamine ne relit pas les styles).
    fn read_sheet_xml(path: &std::path::Path) -> String {
        use std::io::Read;
        let file = std::fs::File::open(path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Liste d'abord les noms d'entrées pour éviter le double borrow.
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();

        for (i, name) in names.iter().enumerate() {
            if name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml") {
                let mut buf = String::new();
                archive
                    .by_index(i)
                    .unwrap()
                    .read_to_string(&mut buf)
                    .unwrap();
                return buf;
            }
        }
        String::new()
    }

    fn read_styles_xml(path: &std::path::Path) -> String {
        use std::io::Read;
        let file = std::fs::File::open(path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut buf = String::new();
        archive
            .by_name("xl/styles.xml")
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        buf
    }

    #[tokio::test]
    async fn set_format_bold_italic() {
        let dir = tmp();
        let path = dir.path().join("fmt.xlsx");
        let ops = json!([
            {"type": "set_cell", "cell": "A1", "value": "Hello"},
            {"type": "set_format", "cell": "A1", "bold": true, "italic": true}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        // styles.xml doit contenir la définition bold+italic
        let styles = read_styles_xml(&path);
        assert!(styles.contains("<b/>"), "gras absent de styles.xml");
        assert!(styles.contains("<i/>"), "italique absent de styles.xml");
    }

    #[tokio::test]
    async fn set_format_font_color() {
        let dir = tmp();
        let path = dir.path().join("color.xlsx");
        let ops = json!([
            {"type": "set_cell", "cell": "A1", "value": "Rouge"},
            {"type": "set_format", "cell": "A1", "font_color": "FF0000"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let styles = read_styles_xml(&path);
        // rust_xlsxwriter écrit les couleurs en ARGB "FFFF0000"
        assert!(
            styles.contains("FF0000"),
            "couleur police absente de styles.xml: {styles}"
        );
    }

    #[tokio::test]
    async fn set_format_bg_color() {
        let dir = tmp();
        let path = dir.path().join("bg.xlsx");
        let ops = json!([
            {"type": "set_format", "cell": "A1", "bg_color": "FFFF00"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let styles = read_styles_xml(&path);
        assert!(
            styles.contains("FFFF00"),
            "couleur fond absente de styles.xml: {styles}"
        );
    }

    #[tokio::test]
    async fn set_number_format_euro() {
        let dir = tmp();
        let path = dir.path().join("num.xlsx");
        let ops = json!([
            {"type": "set_number_format", "cell": "A1", "value": 1234.5, "number_format": "#,##0.00 €"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let styles = read_styles_xml(&path);
        assert!(
            styles.contains("#,##0.00"),
            "format nombre absent de styles.xml: {styles}"
        );
    }

    #[tokio::test]
    async fn set_border_thin_all() {
        let dir = tmp();
        let path = dir.path().join("border.xlsx");
        let ops = json!([
            {"type": "set_border", "cell": "A1", "border_style": "thin"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let styles = read_styles_xml(&path);
        assert!(
            styles.contains("thin"),
            "bordure thin absente de styles.xml: {styles}"
        );
    }

    #[tokio::test]
    async fn set_border_specific_sides() {
        let dir = tmp();
        let path = dir.path().join("border_sides.xlsx");
        let ops = json!([
            {"type": "set_border", "cell": "A1", "border_style": "medium", "border_sides": ["top", "left"]}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let styles = read_styles_xml(&path);
        assert!(styles.contains("medium"), "bordure medium absente");
    }

    #[tokio::test]
    async fn merge_cells_a1_c3() {
        let dir = tmp();
        let path = dir.path().join("merge.xlsx");
        let ops = json!([
            {"type": "merge_cells", "start_cell": "A1", "end_cell": "C3"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        // La fusion apparaît dans le XML de la feuille via <mergeCell ref="A1:C3"/>
        let sheet_xml = read_sheet_xml(&path);
        assert!(
            sheet_xml.contains("A1:C3"),
            "fusion absente du XML feuille: {sheet_xml}"
        );
    }

    #[tokio::test]
    async fn set_row_height() {
        let dir = tmp();
        let path = dir.path().join("height.xlsx");
        let ops = json!([
            {"type": "set_row_height", "row": 0, "height": 40.0}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let sheet_xml = read_sheet_xml(&path);
        // <row r="1" ht="40" customHeight="1"/>
        assert!(
            sheet_xml.contains("ht=\"40\"") || sheet_xml.contains("ht=\"40."),
            "hauteur ligne absente du XML feuille: {sheet_xml}"
        );
    }

    #[tokio::test]
    async fn edit_set_format_on_existing() {
        let dir = tmp();
        let path = dir.path().join("edit_fmt.xlsx");

        // 1. Créer
        let ops_create = json!([
            {"type": "set_cell", "cell": "A1", "value": "original"}
        ]);
        let r1 = write_spreadsheet(path.to_str().unwrap(), &ops_create, dir.path()).await;
        assert!(!r1.is_error, "Création: {}", r1.content);

        // 2. Éditer avec set_format (passe par le backend umya)
        let ops_edit = json!([
            {"type": "set_format", "cell": "A1", "bold": true}
        ]);
        let r2 = write_spreadsheet(path.to_str().unwrap(), &ops_edit, dir.path()).await;
        assert!(!r2.is_error, "Édition: {}", r2.content);

        let styles = read_styles_xml(&path);
        assert!(styles.contains("<b/>"), "gras absent après édition umya");
    }

    #[tokio::test]
    async fn set_format_invalid_color_hex() {
        let dir = tmp();
        let path = dir.path().join("bad_color.xlsx");
        let ops = json!([
            {"type": "set_format", "cell": "A1", "font_color": "ZZZZZZ"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(result.is_error, "Devrait échouer sur couleur invalide");
        assert!(
            result.content.contains("couleur invalide"),
            "Message d'erreur inattendu: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn set_format_invalid_border_style() {
        let dir = tmp();
        let path = dir.path().join("bad_border.xlsx");
        let ops = json!([
            {"type": "set_border", "cell": "A1", "border_style": "gigantic"}
        ]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(
            result.is_error,
            "Devrait échouer sur style bordure invalide"
        );
    }
}
