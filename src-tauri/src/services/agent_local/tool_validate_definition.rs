use serde_json::Value;

pub fn validate(tool: &str, args: &Value, definition: &Value) -> Result<Value, String> {
    let object = args
        .as_object()
        .ok_or_else(|| "les arguments doivent être un objet JSON".to_string())?;
    let parameters = definition
        .pointer("/function/parameters")
        .and_then(Value::as_object)
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let properties = parameters
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let required = parameters
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;

    for name in required.iter().filter_map(Value::as_str) {
        if !matches!(object.get(name), Some(value) if !value.is_null()) {
            return Err(format!("paramètre '{name}' requis"));
        }
    }

    let mut cleaned = serde_json::Map::with_capacity(properties.len());
    for (name, value) in object {
        let Some(property) = properties.get(name) else {
            eprintln!("[tool-validate] argument inconnu ignoré : {tool}.{name}");
            continue;
        };
        if !value.is_null() {
            validate_value(name, value, property)?;
        }
        cleaned.insert(name.clone(), value.clone());
    }
    Ok(Value::Object(cleaned))
}

fn validate_value(name: &str, value: &Value, property: &Value) -> Result<(), String> {
    let expected = property["type"]
        .as_str()
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let valid = match expected {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "boolean" => value.is_boolean(),
        _ => false,
    };
    if !valid {
        return Err(format!("'{name}' doit être de type {expected}"));
    }
    validate_constraints(name, value, property)?;
    validate_children(name, value, property)
}

fn validate_constraints(name: &str, value: &Value, property: &Value) -> Result<(), String> {
    if let Some(text) = value.as_str() {
        let length = text.chars().count() as u64;
        check_u64_bound(name, length, property, "minLength", false)?;
        check_u64_bound(name, length, property, "maxLength", true)?;
    }
    if let Some(items) = value.as_array() {
        let length = items.len() as u64;
        check_u64_bound(name, length, property, "minItems", false)?;
        check_u64_bound(name, length, property, "maxItems", true)?;
    }
    if let Some(number) = value.as_f64() {
        check_number_bound(name, number, property, "minimum", false)?;
        check_number_bound(name, number, property, "maximum", true)?;
    }
    if let Some(allowed) = property.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            return Err(format!("'{name}' contient une valeur invalide"));
        }
    }
    Ok(())
}

fn check_u64_bound(
    name: &str,
    actual: u64,
    schema: &Value,
    key: &str,
    maximum: bool,
) -> Result<(), String> {
    let Some(bound) = schema.get(key).and_then(Value::as_u64) else {
        return Ok(());
    };
    if (maximum && actual > bound) || (!maximum && actual < bound) {
        return Err(format!("'{name}' est hors limites"));
    }
    Ok(())
}

fn check_number_bound(
    name: &str,
    actual: f64,
    schema: &Value,
    key: &str,
    maximum: bool,
) -> Result<(), String> {
    let Some(bound) = schema.get(key).and_then(Value::as_f64) else {
        return Ok(());
    };
    if (maximum && actual > bound) || (!maximum && actual < bound) {
        return Err(format!("'{name}' est hors limites"));
    }
    Ok(())
}

fn validate_children(name: &str, value: &Value, property: &Value) -> Result<(), String> {
    if let (Some(items), Some(item_schema)) = (value.as_array(), property.get("items")) {
        for (index, item) in items.iter().enumerate() {
            validate_value(&format!("{name}[{index}]"), item, item_schema)?;
        }
    }
    let (Some(object), Some(properties)) = (
        value.as_object(),
        property.get("properties").and_then(Value::as_object),
    ) else {
        return Ok(());
    };
    for (child_name, child_value) in object {
        if let Some(child_schema) = properties.get(child_name) {
            validate_value(child_name, child_value, child_schema)?;
        }
    }
    Ok(())
}
