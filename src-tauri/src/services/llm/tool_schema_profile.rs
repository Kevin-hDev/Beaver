use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SchemaProfile {
    Generic,
    Google,
    Kimi,
    Xai,
}

pub(super) fn resolve(provider_id: &str, model: &str) -> SchemaProfile {
    match provider_id {
        "google" => SchemaProfile::Google,
        "openrouter" if model.to_ascii_lowercase().starts_with("google/") => SchemaProfile::Google,
        "openrouter"
            if model.to_ascii_lowercase().starts_with("moonshotai/")
                || model.to_ascii_lowercase().starts_with("kimi/") =>
        {
            SchemaProfile::Kimi
        }
        "openrouter" if model.to_ascii_lowercase().starts_with("x-ai/") => SchemaProfile::Xai,
        "moonshot" => SchemaProfile::Kimi,
        "xai" => SchemaProfile::Xai,
        _ => SchemaProfile::Generic,
    }
}

pub(super) fn apply_strict_mode(
    function: &mut serde_json::Map<String, Value>,
    provider_id: &str,
    profile: SchemaProfile,
) {
    if matches!(
        provider_id,
        "openai" | "codex-oauth" | "moonshot" | "deepseek"
    ) || profile == SchemaProfile::Kimi
    {
        function.insert("strict".to_string(), Value::Bool(false));
    }
}

pub(super) fn remove_unsupported_keywords(
    map: &mut serde_json::Map<String, Value>,
    profile: SchemaProfile,
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
    if matches!(profile, SchemaProfile::Google | SchemaProfile::Kimi) {
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
    if profile == SchemaProfile::Kimi {
        for key in ["minimum", "maximum", "minItems", "maxItems"] {
            map.remove(key);
        }
    }
}
