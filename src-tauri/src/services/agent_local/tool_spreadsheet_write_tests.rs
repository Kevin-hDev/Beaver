#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_spreadsheet_write::{parse_cell_ref, write_spreadsheet};
    use calamine::{open_workbook_auto, Reader, Sheets};
    use serde_json::json;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    // --- Tests unitaires parse_cell_ref ---

    #[test]
    fn parse_a1() {
        assert_eq!(parse_cell_ref("A1"), Some((0, 0)));
    }

    #[test]
    fn parse_b2() {
        assert_eq!(parse_cell_ref("B2"), Some((1, 1)));
    }

    #[test]
    fn parse_z26() {
        assert_eq!(parse_cell_ref("Z26"), Some((25, 25)));
    }

    #[test]
    fn parse_aa1() {
        assert_eq!(parse_cell_ref("AA1"), Some((0, 26)));
    }

    #[test]
    fn parse_invalid_empty() {
        assert_eq!(parse_cell_ref(""), None);
    }

    #[test]
    fn parse_invalid_no_row() {
        assert_eq!(parse_cell_ref("ABC"), None);
    }

    #[test]
    fn parse_row_zero_invalid() {
        assert_eq!(parse_cell_ref("A0"), None);
    }

    #[test]
    fn parse_column_overflow_invalid() {
        assert_eq!(parse_cell_ref("ZZZZ1"), None);
    }

    // --- Tests écriture xlsx ---

    #[tokio::test]
    async fn create_new_xlsx_set_cell() {
        let dir = tmp();
        let path = dir.path().join("new.xlsx");

        let ops = json!([
            {"type": "set_cell", "cell": "A1", "value": "Bonjour"},
            {"type": "set_cell", "cell": "B1", "value": 42.0}
        ]);

        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        assert!(path.exists());

        // Vérifier avec calamine
        let mut wb: Sheets<_> = open_workbook_auto(&path).unwrap();
        let sheet = wb.worksheet_range_at(0).unwrap().unwrap();
        let a1 = sheet.get((0, 0)).map(|c| c.to_string()).unwrap_or_default();
        assert_eq!(a1, "Bonjour");
    }

    #[tokio::test]
    async fn create_xlsx_set_row() {
        let dir = tmp();
        let path = dir.path().join("row.xlsx");

        let ops = json!([
            {"type": "set_row", "row": 0, "values": ["Col1", "Col2", "Col3"]}
        ]);

        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);

        let mut wb: Sheets<_> = open_workbook_auto(&path).unwrap();
        let sheet = wb.worksheet_range_at(0).unwrap().unwrap();
        let a1 = sheet.get((0, 0)).map(|c| c.to_string()).unwrap_or_default();
        let b1 = sheet.get((0, 1)).map(|c| c.to_string()).unwrap_or_default();
        assert_eq!(a1, "Col1");
        assert_eq!(b1, "Col2");
    }

    #[tokio::test]
    async fn create_xlsx_formula() {
        let dir = tmp();
        let path = dir.path().join("formula.xlsx");

        let ops = json!([
            {"type": "set_cell", "cell": "A1", "value": 10.0},
            {"type": "set_cell", "cell": "A2", "value": 20.0},
            {"type": "set_formula", "cell": "A3", "formula": "=SUM(A1:A2)"}
        ]);

        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        // On vérifie juste que le fichier est créé — calamine ne calcule pas les formules
        assert!(path.exists());
    }

    #[tokio::test]
    async fn set_cell_does_not_create_formula() {
        let dir = tmp();
        let path = dir.path().join("formula-text.xlsx");

        let ops = json!([
            {"type": "set_cell", "cell": "A1", "value": "=HYPERLINK(\"http://example.test\")"}
        ]);

        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);

        let mut wb: Sheets<_> = open_workbook_auto(&path).unwrap();
        let formulas = wb.worksheet_formula("Sheet1").unwrap();
        assert!(formulas
            .get((0, 0))
            .map(String::as_str)
            .unwrap_or("")
            .is_empty());
    }

    #[tokio::test]
    async fn unknown_operation_is_error() {
        let dir = tmp();
        let path = dir.path().join("unknown.xlsx");
        let ops = json!([{ "type": "surprise" }]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn modify_existing_xlsx() {
        let dir = tmp();
        let path = dir.path().join("modify.xlsx");

        // Créer d'abord
        let ops_create = json!([
            {"type": "set_cell", "cell": "A1", "value": "original"}
        ]);
        let r1 = write_spreadsheet(path.to_str().unwrap(), &ops_create, dir.path()).await;
        assert!(!r1.is_error, "Création: {}", r1.content);

        // Modifier
        let ops_modify = json!([
            {"type": "set_cell", "cell": "A1", "value": "modifié"},
            {"type": "set_cell", "cell": "B1", "value": 99.0}
        ]);
        let r2 = write_spreadsheet(path.to_str().unwrap(), &ops_modify, dir.path()).await;
        assert!(!r2.is_error, "Modification: {}", r2.content);

        // Vérifier
        let mut wb: Sheets<_> = open_workbook_auto(&path).unwrap();
        let sheet = wb.worksheet_range_at(0).unwrap().unwrap();
        let a1 = sheet.get((0, 0)).map(|c| c.to_string()).unwrap_or_default();
        assert_eq!(a1, "modifié");
    }

    #[tokio::test]
    async fn invalid_path_outside_zone() {
        let dir = tmp();
        let ops = json!([{"type": "set_cell", "cell": "A1", "value": "test"}]);
        let result = write_spreadsheet("/nonexistent/outside/file.xlsx", &ops, dir.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn empty_operations() {
        let dir = tmp();
        let path = dir.path().join("empty.xlsx");
        let ops = json!([]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        // Doit créer le fichier sans erreur
        assert!(!result.is_error, "Erreur: {}", result.content);
        assert!(path.exists());
    }

    #[tokio::test]
    async fn unsupported_format_csv() {
        let dir = tmp();
        let path = dir.path().join("file.csv");
        let ops = json!([{"type": "set_cell", "cell": "A1", "value": "test"}]);
        let result = write_spreadsheet(path.to_str().unwrap(), &ops, dir.path()).await;
        assert!(result.is_error);
    }
}
