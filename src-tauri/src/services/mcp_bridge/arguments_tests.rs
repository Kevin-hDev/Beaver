use super::arguments::validate;
use serde_json::{json, Value};

fn object_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "minLength": 2, "maxLength": 8},
            "count": {"type": "integer", "minimum": 1, "maximum": 5},
            "mode": {"enum": ["read", "write"]},
            "tags": {"type": "array", "items": {"type": "string"}, "maxItems": 2}
        },
        "required": ["name"],
        "additionalProperties": false
    })
}

#[test]
fn requires_a_bounded_json_object() {
    assert!(validate(&json!([1, 2]), Some(&object_schema())).is_err());
    assert!(validate(&json!({"blob": "x".repeat(65_537)}), Some(&object_schema())).is_err());

    let mut deep = json!({});
    for _ in 0..17 {
        deep = json!({"next": deep});
    }
    assert!(validate(&deep, Some(&json!({"type": "object"}))).is_err());

    let many = json!({"items": (0..257).collect::<Vec<_>>()});
    assert!(validate(&many, Some(&json!({"type": "object"}))).is_err());
}

#[test]
fn validates_types_required_values_arrays_and_bounds() {
    let schema = object_schema();
    assert!(validate(
        &json!({"name": "alice", "count": 2, "mode": "read"}),
        Some(&schema)
    )
    .is_ok());
    assert!(validate(&json!({"count": 2}), Some(&schema)).is_err());
    assert!(validate(&json!({"name": "a"}), Some(&schema)).is_err());
    assert!(validate(&json!({"name": "alice", "count": 8}), Some(&schema)).is_err());
    assert!(validate(&json!({"name": "alice", "mode": "admin"}), Some(&schema)).is_err());
    assert!(validate(
        &json!({"name": "alice", "tags": ["a", "b", "c"]}),
        Some(&schema)
    )
    .is_err());
    assert!(validate(&json!({"name": "alice", "extra": true}), Some(&schema)).is_err());
}

#[test]
fn validates_common_schema_combinations() {
    let schema = json!({
        "type": "object",
        "properties": {
            "value": {
                "oneOf": [
                    {"type": "string", "pattern": "^[a-z]+$"},
                    {"type": "integer", "minimum": 10}
                ]
            }
        },
        "required": ["value"]
    });
    assert!(validate(&json!({"value": "valid"}), Some(&schema)).is_ok());
    assert!(validate(&json!({"value": 12}), Some(&schema)).is_ok());
    assert!(validate(&json!({"value": "INVALID"}), Some(&schema)).is_err());
    assert!(validate(&json!({"value": 4}), Some(&schema)).is_err());
}

#[test]
fn missing_or_unsupported_schema_blocks_the_call() {
    assert!(validate(&json!({}), None).is_err());
    assert!(validate(
        &json!({}),
        Some(&json!({"type": "object", "$ref": "#/$defs/x"}))
    )
    .is_err());
    assert!(validate(&json!({}), Some(&json!({"type": "array"}))).is_err());

    let oversized = json!({"type": "object", "examples": vec![json!({}); 257]});
    assert!(validate(&json!({}), Some(&oversized)).is_err());
}
