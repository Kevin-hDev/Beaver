use serde_json::Value;

pub fn validate(tool: &str, args: &Value, definition: &Value) -> Result<Value, String> {
    let parameters = definition
        .pointer("/function/parameters")
        .ok_or_else(|| format!("schéma d'outil indisponible pour {tool}"))?;
    validate_value("", args, parameters)?;
    Ok(args.clone())
}

fn validate_value(name: &str, value: &Value, schema: &Value) -> Result<(), String> {
    if let Some(variants) = schema.get("oneOf").and_then(Value::as_array) {
        let matches = variants
            .iter()
            .filter(|variant| validate_value(name, value, variant).is_ok())
            .count();
        return (matches == 1)
            .then_some(())
            .ok_or_else(|| format!("'{}' ne correspond pas au schéma attendu", label(name)));
    }
    if let Some(variants) = schema.get("anyOf").and_then(Value::as_array) {
        return variants
            .iter()
            .any(|variant| validate_value(name, value, variant).is_ok())
            .then_some(())
            .ok_or_else(|| format!("'{}' ne correspond pas au schéma attendu", label(name)));
    }
    if let Some(expected) = schema.get("const") {
        return (value == expected)
            .then_some(())
            .ok_or_else(|| format!("'{}' contient une valeur invalide", label(name)));
    }

    let expected = schema["type"]
        .as_str()
        .ok_or_else(|| "schéma d'outil indisponible".to_string())?;
    let valid = match expected {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    };
    if !valid {
        return Err(format!("'{}' doit être de type {expected}", label(name)));
    }

    validate_constraints(name, value, schema)?;
    match expected {
        "array" => validate_array(name, value, schema),
        "object" => validate_object(name, value, schema),
        "string" => validate_string_format(name, value, schema),
        _ => Ok(()),
    }
}

fn validate_array(name: &str, value: &Value, schema: &Value) -> Result<(), String> {
    let Some(item_schema) = schema.get("items") else {
        return Ok(());
    };
    for (index, item) in value.as_array().into_iter().flatten().enumerate() {
        validate_value(&format!("{name}[{index}]"), item, item_schema)?;
    }
    Ok(())
}

fn validate_object(name: &str, value: &Value, schema: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("'{name}' doit être de type object"))?;
    let required = schema["required"]
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    for required_name in &required {
        if !matches!(object.get(*required_name), Some(value) if !value.is_null()) {
            return Err(format!("paramètre '{required_name}' requis"));
        }
    }

    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Ok(());
    };
    for (child_name, child_value) in object {
        let Some(child_schema) = properties.get(child_name) else {
            let accepted = properties.keys().cloned().collect::<Vec<_>>().join(", ");
            return Err(format!(
                "paramètre '{}' inconnu; paramètres acceptés: {accepted}",
                child_path(name, child_name)
            ));
        };
        if child_value.is_null() && !required.contains(&child_name.as_str()) {
            continue;
        }
        let qualified = child_path(name, child_name);
        validate_value(&qualified, child_value, child_schema)?;
    }
    Ok(())
}

fn child_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else {
        format!("{parent}.{child}")
    }
}

fn label(name: &str) -> &str {
    if name.is_empty() {
        "arguments"
    } else {
        name
    }
}

fn validate_string_format(name: &str, value: &Value, schema: &Value) -> Result<(), String> {
    if schema["format"].as_str() == Some("uuid")
        && value
            .as_str()
            .is_none_or(|text| uuid::Uuid::parse_str(text).is_err())
    {
        return Err(format!("'{name}' doit être un UUID"));
    }
    Ok(())
}

fn validate_constraints(name: &str, value: &Value, schema: &Value) -> Result<(), String> {
    if let Some(text) = value.as_str() {
        let length = text.chars().count() as u64;
        check_u64_bound(name, length, schema, "minLength", false)?;
        check_u64_bound(name, length, schema, "maxLength", true)?;
    }
    if let Some(items) = value.as_array() {
        let length = items.len() as u64;
        check_u64_bound(name, length, schema, "minItems", false)?;
        check_u64_bound(name, length, schema, "maxItems", true)?;
    }
    if let Some(number) = value.as_f64() {
        check_number_bound(name, number, schema, "minimum", false)?;
        check_number_bound(name, number, schema, "maximum", true)?;
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
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
