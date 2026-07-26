use super::*;
use serde_json::json;

fn request(data: &str, series_column: Option<&str>) -> ForecastRequest {
    ForecastRequest {
        data: Some(data.into()),
        file_path: None,
        data_profile_id: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        series_column: series_column.map(str::to_string),
        covariate_columns: vec!["temp".into()],
        horizon: 2,
        frequency: "D".into(),
        model: Some("timegpt-2-standard".into()),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

#[test]
fn mono_response_remains_unlabelled() {
    let request = request(
        r#"[{"date":"2026-05-01","sales":10,"temp":1},{"date":"2026-05-02","sales":11,"temp":2},{"date":"2026-05-03","sales":12,"temp":3},{"date":"2026-05-04","sales":null,"temp":4},{"date":"2026-05-05","sales":null,"temp":5}]"#,
        None,
    );
    let input = parse_request_input(&request).unwrap();
    let body = response_body(vec![12.0, 13.0]);
    let result = parse_response(&body, &request, &input, None).unwrap();

    assert!(result
        .predictions
        .iter()
        .all(|point| point.series_id.is_none()));
}

#[test]
fn multiseries_response_restores_each_series_and_date() {
    let request = request(
        r#"[
          {"date":"2026-05-01","asset":"A","sales":10,"temp":1},
          {"date":"2026-05-01","asset":"B","sales":20,"temp":2},
          {"date":"2026-05-02","asset":"A","sales":11,"temp":3},
          {"date":"2026-05-02","asset":"B","sales":21,"temp":4},
          {"date":"2026-05-03","asset":"A","sales":12,"temp":5},
          {"date":"2026-05-03","asset":"B","sales":22,"temp":6},
          {"date":"2026-05-04","asset":"A","sales":null,"temp":7},
          {"date":"2026-05-05","asset":"A","sales":null,"temp":8},
          {"date":"2026-05-04","asset":"B","sales":null,"temp":9},
          {"date":"2026-05-05","asset":"B","sales":null,"temp":10}
        ]"#,
        Some("asset"),
    );
    let input = parse_request_input(&request).unwrap();
    let result = parse_response(
        &response_body(vec![11.0, 12.0, 21.0, 22.0]),
        &request,
        &input,
        None,
    )
    .unwrap();

    let labels: Vec<_> = result
        .predictions
        .iter()
        .map(|point| (point.series_id.as_deref(), point.date.as_str()))
        .collect();
    assert_eq!(
        labels,
        vec![
            (Some("A"), "2026-05-04"),
            (Some("A"), "2026-05-05"),
            (Some("B"), "2026-05-04"),
            (Some("B"), "2026-05-05"),
        ]
    );
}

#[test]
fn response_must_match_the_full_horizon() {
    let request = request(
        r#"[{"date":"2026-05-01","sales":10,"temp":1},{"date":"2026-05-02","sales":11,"temp":2},{"date":"2026-05-03","sales":12,"temp":3},{"date":"2026-05-04","sales":null,"temp":4},{"date":"2026-05-05","sales":null,"temp":5}]"#,
        None,
    );
    let input = parse_request_input(&request).unwrap();

    assert!(parse_response(&response_body(vec![12.0]), &request, &input, None).is_err());
}

fn response_body(mean: Vec<f64>) -> Value {
    let lower: Vec<_> = mean.iter().map(|value| value - 1.0).collect();
    let upper: Vec<_> = mean.iter().map(|value| value + 1.0).collect();
    json!({"mean": mean, "intervals": {"lo-90": lower, "hi-90": upper}})
}
