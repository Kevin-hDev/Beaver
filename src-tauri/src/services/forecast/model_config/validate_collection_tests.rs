use super::*;
use crate::services::forecast::model_config::schema::{ParamKind, ParamSpec};
use serde_json::{json, Map, Value};

fn spec(id: &'static str, kind: ParamKind) -> ParamSpec {
    ParamSpec {
        id,
        kind,
        default_value: json!(null),
        min: None,
        max: None,
        options: &[],
    }
}

#[test]
fn number_list_accepts_array() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!([80.0, 90.0, 95.0]));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["levels"].as_array().unwrap().len(), 3);
}

#[test]
fn number_list_sorts_and_deduplicates() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!([95.0, 80.0, 90.0, 80.0]));

    let result = sanitize(&specs, values).unwrap();
    let levels: Vec<f64> = result["levels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_f64().unwrap())
        .collect();
    assert_eq!(levels, vec![80.0, 90.0, 95.0]);
}

#[test]
fn number_list_rejects_too_many_values() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert(
        "levels".into(),
        json!([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]),
    );

    assert!(sanitize(&specs, values).is_err());
}

#[test]
fn number_list_accepts_comma_separated_string() {
    let specs = [spec("levels", ParamKind::NumberList)];
    let mut values = Map::new();
    values.insert("levels".into(), json!("80, 90, 95"));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["levels"].as_array().unwrap().len(), 3);
}

#[test]
fn null_and_empty_values_are_removed() {
    let specs = [spec("a", ParamKind::Integer), spec("b", ParamKind::Select)];
    let mut values = Map::new();
    values.insert("a".into(), Value::Null);
    values.insert("b".into(), json!("   "));

    assert!(sanitize(&specs, values).unwrap().is_empty());
}

#[test]
fn unknown_param_id_is_rejected() {
    let specs = [spec("known", ParamKind::Integer)];
    let mut values = Map::new();
    values.insert("unknown".into(), json!(1));

    assert!(sanitize(&specs, values).is_err());
}

#[test]
fn default_value_is_not_included_in_output() {
    let specs = [ParamSpec {
        id: "horizon",
        kind: ParamKind::Integer,
        default_value: json!(10),
        min: None,
        max: None,
        options: &[],
    }];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(10));

    assert!(sanitize(&specs, values).unwrap().is_empty());
}

#[test]
fn non_default_value_is_included_in_output() {
    let specs = [ParamSpec {
        id: "horizon",
        kind: ParamKind::Integer,
        default_value: json!(10),
        min: None,
        max: None,
        options: &[],
    }];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(20));

    assert_eq!(sanitize(&specs, values).unwrap()["horizon"], 20);
}
