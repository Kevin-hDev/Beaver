use super::*;
use serde_json::json;

fn request(data: &str) -> ForecastRequest {
    ForecastRequest {
        data: Some(data.into()),
        file_path: None,
        data_profile_id: None,
        target_column: "sales".into(),
        date_column: "date".into(),
        series_column: None,
        covariate_columns: Vec::new(),
        horizon: 2,
        frequency: "D".into(),
        model: Some("timegpt-2-standard".into()),
        confidence_level: 0.9,
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    }
}

fn mono_data() -> &'static str {
    r#"[
      {"date":"2026-05-01","sales":"10","temp":1},
      {"date":"2026-05-02","sales":11,"temp":2},
      {"date":"2026-05-03","sales":null,"temp":3},
      {"date":"2026-05-04","sales":null,"temp":4}
    ]"#
}

fn multi_data() -> &'static str {
    r#"[
      {"date":"2026-05-01","asset":"A","sales":10,"temp":1},
      {"date":"2026-05-01","asset":"B","sales":20,"temp":2},
      {"date":"2026-05-02","asset":"A","sales":11,"temp":3},
      {"date":"2026-05-02","asset":"B","sales":21,"temp":4},
      {"date":"2026-05-03","asset":"A","sales":null,"temp":5},
      {"date":"2026-05-04","asset":"A","sales":null,"temp":6},
      {"date":"2026-05-03","asset":"B","sales":null,"temp":7},
      {"date":"2026-05-04","asset":"B","sales":null,"temp":8}
    ]"#
}

fn with_covariates(data: &str) -> ForecastRequest {
    let mut value = request(data);
    value.covariate_columns = vec!["temp".into()];
    value
}

#[test]
fn plain_single_series_uses_current_v2_contract_and_maps_standard_model() {
    let request = request(r#"[{"date":"2026-05-01","sales":10},{"date":"2026-05-02","sales":11}]"#);
    let input = parse_request_input(&request).unwrap();
    let payload = build_payload(&input, &request).unwrap();

    assert_eq!(endpoint(), "https://api.nixtla.io/v2/forecast");
    assert_eq!(payload["model"], "timegpt-2");
    assert_eq!(payload["series"]["y"], json!([10.0, 11.0]));
    assert_eq!(payload["series"]["sizes"], json!([2]));
}

#[test]
fn mono_covariates_use_v2_feature_major_payload() {
    let request = with_covariates(mono_data());
    let input = parse_request_input(&request).unwrap();
    let payload = build_payload(&input, &request).unwrap();

    assert_eq!(endpoint(), "https://api.nixtla.io/v2/forecast");
    assert_eq!(payload["series"]["sizes"], json!([2]));
    assert_eq!(payload["series"]["X"], json!([[1.0, 2.0]]));
    assert_eq!(payload["series"]["X_future"], json!([[3.0, 4.0]]));
}

#[test]
fn multi_payload_aligns_y_sizes_and_exogenous_rows() {
    let mut request = with_covariates(multi_data());
    request.series_column = Some("asset".into());
    let input = parse_request_input(&request).unwrap();
    let payload = build_payload(&input, &request).unwrap();

    assert_eq!(payload["series"]["sizes"], json!([2, 2]));
    assert_eq!(payload["series"]["y"], json!([10.0, 11.0, 20.0, 21.0]));
    assert_eq!(payload["series"]["X"], json!([[1.0, 3.0, 2.0, 4.0]]));
    assert_eq!(payload["series"]["X_future"], json!([[5.0, 6.0, 7.0, 8.0]]));
    assert!(payload.get("multivariate").is_none());
}

#[test]
fn categorical_covariates_are_declared_and_type_checked() {
    let data = mono_data()
        .replace("\"temp\":1", "\"temp\":\"north\"")
        .replace("\"temp\":2", "\"temp\":\"south\"")
        .replace("\"temp\":3", "\"temp\":\"north\"")
        .replace("\"temp\":4", "\"temp\":\"south\"");
    let request = with_covariates(&data);
    let input = parse_request_input(&request).unwrap();
    let payload = build_payload(&input, &request).unwrap();

    assert_eq!(payload["series"]["categorical_exog"], json!([0]));
    assert_eq!(payload["series"]["X"], json!([["north", "south"]]));

    let mixed = data.replacen("\"south\"", "2", 1);
    let request = with_covariates(&mixed);
    let input = parse_request_input(&request).unwrap();
    assert!(build_payload(&input, &request).is_err());
}

#[test]
fn multivariate_flag_requires_timegpt_21_and_aligned_series() {
    let mut request = with_covariates(multi_data());
    request.series_column = Some("asset".into());
    request.model = Some("timegpt-2.1".into());
    let input = parse_request_input(&request).unwrap();
    assert_eq!(
        build_payload(&input, &request).unwrap()["multivariate"],
        true
    );

    let unaligned = multi_data().replacen(
        "2026-05-01\",\"asset\":\"B",
        "2026-04-30\",\"asset\":\"B",
        1,
    );
    request.data = Some(unaligned);
    let input = parse_request_input(&request).unwrap();
    assert!(build_payload(&input, &request)
        .unwrap()
        .get("multivariate")
        .is_none());
}

#[test]
fn covariates_require_complete_future_rows() {
    let mut request = with_covariates(
        r#"[{"date":"2026-05-01","sales":10,"temp":1},{"date":"2026-05-02","sales":11,"temp":2}]"#,
    );
    let input = parse_request_input(&request).unwrap();
    assert!(build_payload(&input, &request).is_err());

    request.data = Some(mono_data().replace("\"temp\":4", "\"temp\":null"));
    let input = parse_request_input(&request).unwrap();
    assert!(build_payload(&input, &request).is_err());

    request.data = Some(mono_data().replace("\"temp\":2", "\"temp\":null"));
    let input = parse_request_input(&request).unwrap();
    assert!(build_payload(&input, &request).is_err());
}

#[test]
fn builder_rejects_a_truncated_future_group() {
    let mut request = with_covariates(multi_data());
    request.series_column = Some("asset".into());
    let mut input = parse_request_input(&request).unwrap();
    input.future_rows.pop();

    assert!(build_payload(&input, &request).is_err());
}
