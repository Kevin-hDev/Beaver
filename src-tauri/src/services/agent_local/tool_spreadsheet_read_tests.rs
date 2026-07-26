#[cfg(test)]
mod tests {
    use crate::services::agent_local::tool_spreadsheet_read::read_spreadsheet;
    use tempfile::TempDir;

    fn working_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn read_csv_basic() {
        let tmp = working_dir();
        let csv_path = tmp.path().join("test.csv");
        std::fs::write(&csv_path, "name,age,city\nAlice,30,Paris\nBob,25,Lyon\n").unwrap();
        let result =
            read_spreadsheet(csv_path.to_str().unwrap(), None, None, None, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["headers"], serde_json::json!(["name", "age", "city"]));
        let rows = json["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn read_csv_semicolon() {
        let tmp = working_dir();
        let csv_path = tmp.path().join("test.csv");
        std::fs::write(&csv_path, "nom;age;ville\nAlice;30;Paris\n").unwrap();
        let result =
            read_spreadsheet(csv_path.to_str().unwrap(), None, None, None, tmp.path()).await;
        assert!(!result.is_error, "Erreur: {}", result.content);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        assert_eq!(json["headers"], serde_json::json!(["nom", "age", "ville"]));
    }

    #[tokio::test]
    async fn read_csv_max_rows() {
        let tmp = working_dir();
        let csv_path = tmp.path().join("big.csv");
        let mut data = String::from("id,value\n");
        for i in 0..100 {
            data.push_str(&format!("{},{}\n", i, i * 10));
        }
        std::fs::write(&csv_path, &data).unwrap();
        let result =
            read_spreadsheet(csv_path.to_str().unwrap(), None, None, Some(10), tmp.path()).await;
        assert!(!result.is_error);
        let json: serde_json::Value = serde_json::from_str(&result.content).unwrap();
        let rows = json["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 10);
        assert_eq!(json["truncated"], true);
    }

    #[tokio::test]
    async fn read_invalid_path() {
        let tmp = working_dir();
        let result = read_spreadsheet("/nonexistent/file.csv", None, None, None, tmp.path()).await;
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn read_csv_sheet_empty_string_falls_back() {
        let tmp = working_dir();
        let csv_path = tmp.path().join("test.csv");
        std::fs::write(&csv_path, "a,b\n1,2\n").unwrap();
        let result =
            read_spreadsheet(csv_path.to_str().unwrap(), Some(""), None, None, tmp.path()).await;
        assert!(
            !result.is_error,
            "sheet='' devrait fallback: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_csv_sheet_whitespace_falls_back() {
        let tmp = working_dir();
        let csv_path = tmp.path().join("test.csv");
        std::fs::write(&csv_path, "x,y\n3,4\n").unwrap();
        let result = read_spreadsheet(
            csv_path.to_str().unwrap(),
            Some("  "),
            None,
            None,
            tmp.path(),
        )
        .await;
        assert!(
            !result.is_error,
            "sheet='  ' devrait fallback: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn read_unsupported_format() {
        let tmp = working_dir();
        let path = tmp.path().join("test.txt");
        std::fs::write(&path, "hello").unwrap();
        let result = read_spreadsheet(path.to_str().unwrap(), None, None, None, tmp.path()).await;
        assert!(result.is_error);
        assert!(result.content.contains("Format non supporté"));
    }
}
