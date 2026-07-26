//! Tests de sanitize (PURE) : validation et normalisation des paramètres
//! forecast typés (Integer/Number/Boolean/Select/NumberList).

use super::*;
use crate::services::forecast::model_config::schema::{ParamKind, ParamSpec};
use serde_json::{json, Map};

/// Construit un ParamSpec statique pour les tests.
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

fn spec_with_range(id: &'static str, kind: ParamKind, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        id,
        kind,
        default_value: json!(null),
        min: Some(min),
        max: Some(max),
        options: &[],
    }
}

// --- Integer ---------------------------------------------------------------

#[test]
fn integer_accepts_valid_value() {
    let specs = [spec("horizon", ParamKind::Integer)];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(12));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["horizon"], 12);
}

#[test]
fn integer_accepts_string_numeric() {
    // Tolérance : "12" est parsé en entier.
    let specs = [spec("horizon", ParamKind::Integer)];
    let mut values = Map::new();
    values.insert("horizon".into(), json!("12"));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["horizon"], 12);
}

#[test]
fn integer_rejects_out_of_range() {
    let specs = [spec_with_range("horizon", ParamKind::Integer, 1.0, 100.0)];
    let mut values = Map::new();
    values.insert("horizon".into(), json!(200));

    assert!(sanitize(&specs, values).is_err());
}

// --- Number ----------------------------------------------------------------

#[test]
fn number_accepts_float() {
    let specs = [spec("lr", ParamKind::Number)];
    let mut values = Map::new();
    values.insert("lr".into(), json!(0.001));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["lr"], 0.001);
}

#[test]
fn number_rejects_non_finite() {
    // NaN / Infinity ne sont pas valides.
    let specs = [spec("lr", ParamKind::Number)];
    let mut values = Map::new();
    values.insert("lr".into(), json!(f64::NAN));

    // serde_json ne sérialise pas NaN, on teste via string.
    values.clear();
    values.insert("lr".into(), json!("NaN"));
    assert!(sanitize(&specs, values).is_err());
}

// --- Boolean ---------------------------------------------------------------

#[test]
fn boolean_accepts_true_false() {
    let specs = [spec("flag", ParamKind::Boolean)];

    let mut values_true = Map::new();
    values_true.insert("flag".into(), json!(true));
    let result_true = sanitize(&specs, values_true).unwrap();
    assert_eq!(result_true["flag"], true);

    let mut values_false = Map::new();
    values_false.insert("flag".into(), json!(false));
    let result_false = sanitize(&specs, values_false).unwrap();
    assert_eq!(result_false["flag"], false);
}

#[test]
fn boolean_rejects_non_bool() {
    let specs = [spec("flag", ParamKind::Boolean)];
    let mut values = Map::new();
    values.insert("flag".into(), json!("true")); // string, pas bool

    assert!(sanitize(&specs, values).is_err());
}

// --- Select ----------------------------------------------------------------

#[test]
fn select_accepts_valid_option() {
    let specs = [ParamSpec {
        id: "mode",
        kind: ParamKind::Select,
        default_value: json!("a"),
        min: None,
        max: None,
        options: &["a", "b", "c"],
    }];
    let mut values = Map::new();
    values.insert("mode".into(), json!("b"));

    let result = sanitize(&specs, values).unwrap();
    assert_eq!(result["mode"], "b");
}

#[test]
fn select_rejects_invalid_option() {
    let specs = [ParamSpec {
        id: "mode",
        kind: ParamKind::Select,
        default_value: json!("a"),
        min: None,
        max: None,
        options: &["a", "b"],
    }];
    let mut values = Map::new();
    values.insert("mode".into(), json!("z"));

    assert!(sanitize(&specs, values).is_err());
}
