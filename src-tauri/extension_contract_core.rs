use serde_json::Value;
use std::collections::BTreeSet;

pub fn validate_optional_capabilities(
    contract: &Value,
    maximum_chars: usize,
) -> Result<(), String> {
    let values = contract["optionalCapabilities"]
        .as_array()
        .ok_or_else(|| "invalid extension contract optional capabilities".to_string())?;
    let strings = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "invalid extension contract optional capabilities".to_string())?;
    let expected = [
        "skills",
        "resources",
        "richToolResults",
        "models",
        "memory",
        "automations",
        "subagents",
        "toolInterception",
    ];
    if strings.as_slice() != expected
        || strings.iter().any(|value| {
            let mut bytes = value.bytes();
            !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
                || value.len() > maximum_chars
                || !bytes.all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
                })
        })
    {
        return Err("invalid extension contract optional capabilities".to_string());
    }
    Ok(())
}

pub fn validate(contract: &Value) -> Result<(), String> {
    validate_transport(contract)?;
    let methods = contract["methods"]["hostToCore"]
        .as_array()
        .ok_or_else(|| "invalid host to core methods".to_string())?;
    let capabilities = contract["optionalCapabilities"]
        .as_array()
        .and_then(|values| values.iter().map(Value::as_str).collect::<Option<BTreeSet<_>>>())
        .ok_or_else(|| "invalid extension contract optional capabilities".to_string())?;
    let effects = contract["effectClasses"]
        .as_array()
        .and_then(|values| values.iter().map(Value::as_str).collect::<Option<BTreeSet<_>>>())
        .ok_or_else(|| "invalid extension effect classes".to_string())?;
    let limits = contract["limits"]
        .as_object()
        .ok_or_else(|| "invalid extension limits".to_string())?;

    for method in methods {
        if method["kind"] == "request" && !method["idempotent"].is_boolean() {
            return Err("invalid core API method idempotence".to_string());
        }
        let Some(capability) = method.get("capability") else {
            continue;
        };
        let object = method
            .as_object()
            .ok_or_else(|| "invalid core API method metadata".to_string())?;
        let allowed = [
            "name",
            "level",
            "kind",
            "rustBudgetMs",
            "capability",
            "requiresContext",
            "idempotent",
            "effects",
            "params",
            "result",
        ];
        if object.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err("invalid core API method metadata".to_string());
        }
        let capability = capability
            .as_str()
            .filter(|value| capabilities.contains(value))
            .ok_or_else(|| "invalid core API capability".to_string())?;
        if capability.is_empty()
            || method["kind"] != "request"
            || method["level"] != "stable"
            || !method["requiresContext"].is_boolean()
            || !method["idempotent"].is_boolean()
            || method["result"].as_str().is_none()
        {
            return Err("invalid core API method metadata".to_string());
        }
        validate_effects(method, &effects)?;
        validate_params(method, limits)?;
    }

    let retryable = contract["errors"]["retryableReasons"]
        .as_array()
        .ok_or_else(|| "invalid retryable reasons".to_string())?;
    let reasons = contract["errors"]["protocolReasons"]
        .as_array()
        .ok_or_else(|| "invalid protocol reasons".to_string())?;
    if retryable.is_empty()
        || retryable.len() > reasons.len()
        || retryable.iter().any(|reason| !reasons.contains(reason))
    {
        return Err("invalid retryable reasons".to_string());
    }
    Ok(())
}

fn validate_transport(contract: &Value) -> Result<(), String> {
    let transport = contract["transport"]
        .as_object()
        .ok_or_else(|| "invalid core API transport".to_string())?;
    let expected = ["contextEnvelopeField", "interceptorContributionField"];
    if transport.len() != expected.len()
        || expected.iter().any(|name| {
            transport
                .get(*name)
                .and_then(Value::as_str)
                .is_none_or(|value| value.is_empty() || value.len() > 64)
        })
    {
        return Err("invalid core API transport".to_string());
    }
    Ok(())
}

fn validate_effects(method: &Value, allowed: &BTreeSet<&str>) -> Result<(), String> {
    let values = method["effects"]
        .as_array()
        .filter(|values| !values.is_empty() && values.len() <= allowed.len())
        .ok_or_else(|| "invalid core API effects".to_string())?;
    let unique = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<BTreeSet<_>>>()
        .ok_or_else(|| "invalid core API effects".to_string())?;
    if unique.len() != values.len() || unique.iter().any(|value| !allowed.contains(value)) {
        return Err("invalid core API effects".to_string());
    }
    Ok(())
}

fn validate_params(
    method: &Value,
    limits: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    let params = method["params"]
        .as_array()
        .filter(|values| values.len() <= 16)
        .ok_or_else(|| "invalid core API parameters".to_string())?;
    let mut names = BTreeSet::new();
    for param in params {
        let object = param
            .as_object()
            .ok_or_else(|| "invalid core API parameter".to_string())?;
        if object
            .keys()
            .any(|key| !["name", "type", "required", "limit"].contains(&key.as_str()))
        {
            return Err("invalid core API parameter".to_string());
        }
        let name = param["name"]
            .as_str()
            .filter(|name| !name.is_empty() && name.len() <= 64)
            .ok_or_else(|| "invalid core API parameter".to_string())?;
        let kind = param["type"].as_str();
        if !names.insert(name)
            || !matches!(
                kind,
                Some("string" | "integer" | "boolean" | "object" | "memoryScope" | "subagentType")
            )
            || !param["required"].is_boolean()
        {
            return Err("invalid core API parameter".to_string());
        }
        if let Some(limit) = param.get("limit") {
            let valid = limit
                .as_str()
                .is_some_and(|limit| limits.get(limit).is_some_and(Value::is_u64));
            if !valid {
                return Err("invalid core API parameter limit".to_string());
            }
        }
    }
    Ok(())
}
