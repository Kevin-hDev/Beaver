use super::openrouter_model_metadata::reasoning;
use serde_json::json;

#[test]
fn reasoning_metadata_keeps_only_known_unique_efforts_and_compatible_default() {
    let parsed = reasoning(&json!({
        "mandatory": true,
        "supported_efforts": [
            "max", "low", "low", "minimal", "none", "medium", "high", "xhigh", "ultra"
        ],
        "default_effort": "none"
    }))
    .expect("reasoning object");

    assert_eq!(parsed.0, ["max", "low", "medium", "high", "xhigh"]);
    assert_eq!(parsed.1, None);
}

#[test]
fn reasoning_metadata_distinguishes_absence_from_an_explicit_empty_object() {
    assert_eq!(reasoning(&json!(null)), None);
    assert_eq!(reasoning(&json!({})), None);
    assert_eq!(reasoning(&json!({"supported_efforts": null})), None);
    assert_eq!(
        reasoning(&json!({"supported_efforts": []})),
        Some((Vec::new(), None))
    );
}

#[test]
fn reasoning_metadata_accepts_a_default_only_when_it_is_advertised() {
    assert_eq!(
        reasoning(&json!({
            "supported_efforts": ["low", "high"],
            "default_effort": "high"
        })),
        Some((
            vec!["low".to_string(), "high".to_string()],
            Some("high".to_string())
        ))
    );
    assert_eq!(
        reasoning(&json!({
            "supported_efforts": ["low", "high"],
            "default_effort": "medium"
        })),
        Some((vec!["low".to_string(), "high".to_string()], None))
    );
}

#[test]
fn non_mandatory_reasoning_may_keep_the_known_off_effort() {
    assert_eq!(
        reasoning(&json!({
            "mandatory": false,
            "supported_efforts": ["off", "none", "low"]
        })),
        Some((vec!["off".to_string(), "low".to_string()], None))
    );
    assert_eq!(
        reasoning(
            &json!({"mandatory":false,"supported_efforts":["high","none"],"default_effort":"none"})
        ),
        Some((vec!["high".into(), "off".into()], Some("off".into())))
    );
}

#[test]
fn unsupported_wire_efforts_are_never_offered_as_working_controls() {
    assert_eq!(
        reasoning(&json!({"supported_efforts":["ultra","minimal","low"],"default_effort":"ultra"})),
        Some((vec!["low".to_string()], None))
    );
}
