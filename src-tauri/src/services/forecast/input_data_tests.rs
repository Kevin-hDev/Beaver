use super::input_data::parse_request_input;
use super::types::ForecastRequest;

fn make_request(data: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(data.to_string()),
        file_path: None,
        data_profile_id: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: vec!["weather".into()],
        horizon: 3,
        frequency: "D".into(),
        model: Some("chronos-bolt-small".into()),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

#[test]
fn parses_history_and_builds_future_dates() {
    let request = make_request(
        r#"
        [
          {"date":"2026-05-01","sales":100,"weather":"sunny"},
          {"date":"2026-05-02","sales":120,"weather":"rain"},
          {"date":"2026-05-03","sales":"135.5","weather":"wind"}
        ]
        "#,
    );

    let parsed = parse_request_input(&request).expect("input should parse");

    assert_eq!(parsed.summary.points, 3);
    assert_eq!(parsed.summary.start, "2026-05-01");
    assert_eq!(parsed.summary.end, "2026-05-03");
    assert_eq!(parsed.values, vec![100.0, 120.0, 135.5]);
    assert_eq!(parsed.snapshot.history.len(), 3);
    assert_eq!(
        parsed.future_dates,
        vec!["2026-05-04", "2026-05-05", "2026-05-06"]
    );
    assert!(parsed
        .snapshot
        .columns
        .iter()
        .any(|column| column == "weather"));
}

#[test]
fn rejects_missing_covariate_column() {
    let mut request = make_request(
        r#"
        [
          {"date":"2026-05-01","sales":100},
          {"date":"2026-05-02","sales":120}
        ]
        "#,
    );
    request.covariate_columns = vec!["holiday".into()];

    let error = parse_request_input(&request).expect_err("missing covariate should fail");

    assert_eq!(error, "Covariable introuvable");
}

#[test]
fn rejects_unknown_date_formats() {
    let mut request = make_request(
        r#"
        [
          {"date":"week-1","sales":100,"weather":"sunny"},
          {"date":"week-2","sales":120,"weather":"rain"}
        ]
        "#,
    );
    request.frequency = "W".into();

    let error = parse_request_input(&request).expect_err("invalid dates must fail");

    assert_eq!(error, "Dates invalides");
}

#[test]
fn parses_multiseries_history_and_future_rows() {
    let mut request = make_request(
        r#"
        [
          {"date":"2026-05-01","asset_id":"A","sales":100,"weather":"sunny"},
          {"date":"2026-05-01","asset_id":"B","sales":200,"weather":"rain"},
          {"date":"2026-05-02","asset_id":"A","sales":110,"weather":"sunny"},
          {"date":"2026-05-02","asset_id":"B","sales":210,"weather":"rain"},
          {"date":"2026-05-03","asset_id":"A","sales":"","weather":"wind"},
          {"date":"2026-05-04","asset_id":"A","sales":null,"weather":"wind"},
          {"date":"2026-05-03","asset_id":"B","sales":"","weather":"storm"},
          {"date":"2026-05-04","asset_id":"B","sales":null,"weather":"storm"}
        ]
        "#,
    );
    request.series_column = Some("asset_id".into());
    request.horizon = 2;

    let parsed = parse_request_input(&request).expect("multi-series input should parse");

    assert_eq!(parsed.summary.points, 4);
    assert_eq!(parsed.snapshot.series_column.as_deref(), Some("asset_id"));
    assert_eq!(parsed.snapshot.series_ids, vec!["A", "B"]);
    assert!(parsed.future_dates.is_empty());
    assert_eq!(parsed.history_rows.len(), 4);
    assert_eq!(parsed.future_rows.len(), 4);
    assert_eq!(parsed.snapshot.history[0].series_id.as_deref(), Some("A"));
    assert_eq!(parsed.snapshot.history[1].series_id.as_deref(), Some("B"));
}
