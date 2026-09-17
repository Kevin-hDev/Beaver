use serde_json::{Map, Value};

pub fn redact_json(value: &Value) -> Value {
    redact_json_inner(value, 0)
}

pub fn redact_json_preserving_shape(value: &mut Value) {
    redact_json_in_place(value, 0, true);
}

pub fn redact_json_high_confidence_preserving_shape(value: &mut Value) {
    redact_json_in_place(value, 0, false);
}

fn redact_json_in_place(value: &mut Value, depth: usize, broad: bool) {
    if depth > 32 {
        *value = Value::String(REDACTED.to_string());
        return;
    }
    match value {
        Value::String(content) if broad => redact_string(content),
        Value::String(content) => redact_high_confidence_string(content),
        Value::Array(items) => {
            for item in items {
                redact_json_in_place(item, depth + 1, broad);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                if is_sensitive_key(key) {
                    zeroize_json_value(item, 0);
                    *item = Value::String(REDACTED.to_string());
                } else {
                    redact_json_in_place(item, depth + 1, broad);
                }
            }
        }
        _ => {}
    }
}

fn redact_json_inner(value: &Value, depth: usize) -> Value {
    if depth > 8 {
        return Value::String(REDACTED.to_string());
    }
    match value {
        Value::String(content) => Value::String(redact_text(content)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(64)
                .map(|item| redact_json_inner(item, depth + 1))
                .collect(),
        ),
        Value::Object(map) => Value::Object(
            map.iter()
                .take(64)
                .map(|(key, item)| {
                    let item = if is_sensitive_key(key) {
                        Value::String(REDACTED.to_string())
                    } else {
                        redact_json_inner(item, depth + 1)
                    };
                    (key.clone(), item)
                })
                .collect::<Map<String, Value>>(),
        ),
        _ => value.clone(),
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| !matches!(character, '_' | '-'))
        .flat_map(char::to_lowercase)
        .collect::<String>();
    [
        "apikey",
        "token",
        "secret",
        "password",
        "authorization",
        "clientsecret",
        "accesstoken",
        "refreshtoken",
    ]
    .iter()
    .any(|suffix| normalized.ends_with(suffix))
}

fn zeroize_json_value(value: &mut Value, depth: usize) {
    if depth > 64 {
        return;
    }
    match value {
        Value::String(content) => content.zeroize(),
        Value::Array(items) => {
            for item in items {
                zeroize_json_value(item, depth + 1);
            }
        }
        Value::Object(map) => {
            for item in map.values_mut() {
                zeroize_json_value(item, depth + 1);
            }
        }
        _ => {}
    }
}
