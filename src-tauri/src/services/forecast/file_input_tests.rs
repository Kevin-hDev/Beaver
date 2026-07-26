use super::file_input::ensure_request_data;
use super::types::ForecastRequest;
use serde_json::Value;

fn make_request(path: &str) -> ForecastRequest {
    ForecastRequest {
        data: None,
        file_path: Some(path.to_string()),
        data_profile_id: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon: 3,
        frequency: "D".into(),
        model: Some("chronos-bolt-small".into()),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

#[tokio::test]
async fn loads_csv_file_into_json_records() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forecast.csv");
    std::fs::write(&path, "date,sales\n2026-05-01,100\n2026-05-02,120\n").unwrap();

    let mut request = make_request(path.to_str().unwrap());
    ensure_request_data(&mut request, None).await.unwrap();

    let rows: Vec<Value> = serde_json::from_str(request.data.as_deref().unwrap()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["date"], "2026-05-01");
    assert_eq!(rows[1]["sales"], 120.0);
}

#[tokio::test]
async fn replaces_blank_data_with_file_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forecast.csv");
    std::fs::write(&path, "date,sales\n2026-05-01,100\n").unwrap();

    let mut request = make_request(path.to_str().unwrap());
    request.data = Some("   ".to_string());
    ensure_request_data(&mut request, None).await.unwrap();

    let rows: Vec<Value> = serde_json::from_str(request.data.as_deref().unwrap()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["sales"], 100.0);
}

#[tokio::test]
async fn replaces_invalid_json_with_file_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("forecast.csv");
    std::fs::write(&path, "date,sales\n2026-05-01,100\n").unwrap();

    let mut request = make_request(path.to_str().unwrap());
    request.data = Some("{invalid".to_string());
    ensure_request_data(&mut request, None).await.unwrap();

    let rows: Vec<Value> = serde_json::from_str(request.data.as_deref().unwrap()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["sales"], 100.0);
}
