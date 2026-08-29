use super::route_profile::SchemaPolicy;
use serde_json::Value;

pub(super) fn apply_strict_mode(function: &mut serde_json::Map<String, Value>, strict: bool) {
    if strict {
        function.insert("strict".to_string(), Value::Bool(false));
    }
}

pub(super) fn remove_unsupported_keywords(
    map: &mut serde_json::Map<String, Value>,
    profile: SchemaPolicy,
) {
    for key in [
        "$schema",
        "$id",
        "$ref",
        "default",
        "examples",
        "propertyNames",
    ] {
        map.remove(key);
    }
    if matches!(profile, SchemaPolicy::Google | SchemaPolicy::Kimi) {
        for key in [
            "const",
            "exclusiveMinimum",
            "exclusiveMaximum",
            "minLength",
            "maxLength",
            "pattern",
            "minProperties",
            "maxProperties",
        ] {
            map.remove(key);
        }
    }
    if profile == SchemaPolicy::Kimi {
        for key in ["minimum", "maximum", "minItems", "maxItems"] {
            map.remove(key);
        }
    }
}
