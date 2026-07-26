use super::*;

#[test]
fn fold_keeps_validation_targets_out_of_training_rows() {
    let points: Vec<_> = (1..=9)
        .map(|value| SourcePoint {
            row: serde_json::json!({"date": value.to_string(), "y": value}),
            date: value.to_string(),
            series_id: None,
            value: value as f64,
        })
        .collect();
    let grouped = group_points(points);
    let fold = build_fold(&grouped, 2, 2, 0, "y").unwrap();
    assert_eq!(fold.series[0].training, [1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(fold.series[0].actual, [6.0, 7.0]);
    assert!(fold.rows[5]["y"].is_null());
}
