//! Tests de effective_level et apply (PURE) : helpers de payload Nixtla.

use super::{apply, effective_level};
use serde_json::{json, Map};

// --- effective_level --------------------------------------------------------

#[test]
fn effective_level_uses_selected_confidence() {
    // 0.9 → 90% ; 0.8 → 80%
    assert_eq!(effective_level(0.9), 90);
    assert_eq!(effective_level(0.8), 80);
    assert_eq!(effective_level(0.95), 95);
}

#[test]
fn effective_level_zero_confidence() {
    assert_eq!(effective_level(0.0), 0);
}

// --- apply (merge des clés de config dans le payload) -----------------------

#[test]
fn apply_merges_known_keys() {
    let mut payload = json!({"model": "timegpt"});
    let mut config = Map::new();
    config.insert("clean_ex_first".into(), json!(true));
    config.insert("finetune_steps".into(), json!(10));

    apply(&mut payload, &config);

    assert_eq!(payload["clean_ex_first"], true);
    assert_eq!(payload["finetune_steps"], 10);
    assert_eq!(payload["model"], "timegpt");
}

#[test]
fn apply_ignores_unknown_keys() {
    let mut payload = json!({"existing": 1});
    let mut config = Map::new();
    config.insert("unknown_key".into(), json!("ignored"));

    apply(&mut payload, &config);

    assert!(payload.get("unknown_key").is_none());
    assert_eq!(payload["existing"], 1);
}

#[test]
fn apply_only_merges_present_keys() {
    // Config partielle : seules les clés présentes sont mergées.
    let mut payload = json!({});
    let mut config = Map::new();
    config.insert("finetune_depth".into(), json!(2));
    // Les autres clés connues absentes ne sont pas ajoutées.

    apply(&mut payload, &config);

    assert_eq!(payload["finetune_depth"], 2);
    assert!(payload.get("clean_ex_first").is_none());
    assert!(payload.get("finetune_steps").is_none());
}

#[test]
fn apply_handles_all_known_keys() {
    let mut payload = json!({});
    let mut config = Map::new();
    config.insert("clean_ex_first".into(), json!(false));
    config.insert("finetune_steps".into(), json!(5));
    config.insert("finetune_loss".into(), json!("mse"));
    config.insert("finetune_depth".into(), json!(3));
    config.insert("feature_contributions".into(), json!(true));

    apply(&mut payload, &config);

    for key in [
        "clean_ex_first",
        "finetune_steps",
        "finetune_loss",
        "finetune_depth",
        "feature_contributions",
    ] {
        assert!(payload.get(key).is_some(), "{key} devrait être présent");
    }
}

#[test]
fn apply_does_nothing_on_non_object_payload() {
    // Sécurité : un payload non-objet (array, string, nombre) ne doit pas panic.
    let mut payload = json!([1, 2, 3]);
    let mut config = Map::new();
    config.insert("clean_ex_first".into(), json!(true));

    apply(&mut payload, &config);

    assert_eq!(payload, json!([1, 2, 3]));
}
