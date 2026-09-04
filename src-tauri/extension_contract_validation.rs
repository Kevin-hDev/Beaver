use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub fn validate(contract: &Value, directory: &Path) -> Result<(), String> {
    let maximum = contract["validation"]["maxNumericLimit"]
        .as_u64()
        .filter(|value| *value > 0)
        .ok_or_else(|| "invalid extension contract numeric maximum".to_string())?;
    for section in ["limits", "timeouts"] {
        let values = contract[section]
            .as_object()
            .ok_or_else(|| format!("missing extension contract object: {section}"))?;
        for (name, value) in values {
            let value = value
                .as_u64()
                .filter(|value| *value > 0 && *value <= maximum)
                .ok_or_else(|| format!("extension contract value out of range: {name}"))?;
            usize::try_from(value)
                .map_err(|_| format!("extension contract value exceeds usize: {name}"))?;
        }
    }
    validate_timeout_order(contract)?;
    validate_method_budgets(contract)?;
    validate_strings(contract)?;
    validate_catalog_count(contract, directory)
}

fn validate_timeout_order(contract: &Value) -> Result<(), String> {
    let timeout = |name: &str| {
        contract["timeouts"][name]
            .as_u64()
            .ok_or_else(|| format!("invalid extension contract timeout: {name}"))
    };
    if timeout("toolCallTimeoutMs")? >= timeout("hostRequestTimeoutMs")? {
        return Err("toolCallTimeoutMs must be below hostRequestTimeoutMs".to_string());
    }
    if timeout("mcpToolTimeoutMs")? >= timeout("coreRequestTimeoutMs")? {
        return Err("mcpToolTimeoutMs must be below coreRequestTimeoutMs".to_string());
    }
    Ok(())
}

fn validate_method_budgets(contract: &Value) -> Result<(), String> {
    let maximum_name_chars = contract_code_limit(contract)?;
    let core_timeout = contract["timeouts"]["coreRequestTimeoutMs"]
        .as_u64()
        .ok_or_else(|| "invalid core request timeout".to_string())?;
    let methods = contract["methods"]["hostToCore"]
        .as_array()
        .ok_or_else(|| "invalid host to core methods".to_string())?;
    if methods.is_empty() || methods.len() > 64 {
        return Err("invalid host to core method count".to_string());
    }
    let mut names = BTreeSet::new();
    for method in methods {
        let name = method["name"]
            .as_str()
            .filter(|name| valid_protocol_code(name, maximum_name_chars))
            .ok_or_else(|| "invalid host to core method".to_string())?;
        if !names.insert(name) || !matches!(method["level"].as_str(), Some("stable" | "advanced")) {
            return Err("invalid host to core method".to_string());
        }
        match method["kind"].as_str() {
            Some("request") => {
                let budget = method["rustBudgetMs"]
                    .as_u64()
                    .ok_or_else(|| "invalid host to core budget".to_string())?;
                if budget >= core_timeout && budget != 0 {
                    return Err(
                        "host to core budget must be below coreRequestTimeoutMs".to_string()
                    );
                }
            }
            Some("notification") if method["rustBudgetMs"].is_null() => {}
            _ => return Err("invalid host to core method kind".to_string()),
        }
    }
    Ok(())
}

fn validate_strings(contract: &Value) -> Result<(), String> {
    let maximum_name_chars = contract_code_limit(contract)?;
    for (pointer, expected) in [
        ("/capabilities", &["tools", "events", "ui"][..]),
        (
            "/contributionTypes",
            &["tool", "event", "ui", "skill", "resource"][..],
        ),
        ("/resultBlockTypes", &["text", "file"][..]),
        ("/resultFilePurposes", &["artifact", "preview"][..]),
        ("/resourceTypes", &["text", "image", "file"][..]),
    ] {
        exact_contract_strings(contract, pointer, expected, maximum_name_chars)?;
    }
    exact_optional_capabilities(contract, maximum_name_chars)?;
    for pointer in [
        "/methods/coreToHost",
        "/events",
        "/hostStates",
        "/loadStages",
        "/effectClasses",
        "/diagnostics/hostCodes",
        "/diagnostics/runtimeCodes",
        "/errors/backendCodes",
        "/errors/protocolReasons",
    ] {
        let values = contract
            .pointer(pointer)
            .and_then(Value::as_array)
            .ok_or_else(|| format!("invalid extension contract list: {pointer}"))?;
        if values.is_empty() || values.len() > 128 {
            return Err(format!("invalid extension contract list: {pointer}"));
        }
        let strings = values
            .iter()
            .map(Value::as_str)
            .collect::<Option<BTreeSet<_>>>()
            .ok_or_else(|| format!("invalid extension contract list: {pointer}"))?;
        if strings.len() != values.len()
            || strings
                .iter()
                .any(|value| !valid_protocol_code(value, maximum_name_chars))
        {
            return Err(format!("invalid extension contract list: {pointer}"));
        }
    }
    let capabilities = contract["capabilities"]
        .as_array()
        .ok_or_else(|| "invalid extension contract capabilities".to_string())?;
    let optional = contract["optionalCapabilities"]
        .as_array()
        .ok_or_else(|| "invalid extension contract optional capabilities".to_string())?;
    if optional.iter().any(|value| capabilities.contains(value)) {
        return Err("extension contract capability is declared twice".to_string());
    }
    Ok(())
}

fn exact_contract_strings(
    contract: &Value,
    pointer: &str,
    expected: &[&str],
    maximum_name_chars: usize,
) -> Result<(), String> {
    let values = contract
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("invalid extension contract list: {pointer}"))?;
    let strings = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| format!("invalid extension contract list: {pointer}"))?;
    if strings.as_slice() != expected
        || strings
            .iter()
            .any(|value| !valid_protocol_code(value, maximum_name_chars))
    {
        return Err(format!("invalid extension contract list: {pointer}"));
    }
    Ok(())
}

fn exact_optional_capabilities(contract: &Value, maximum_name_chars: usize) -> Result<(), String> {
    let values = contract["optionalCapabilities"]
        .as_array()
        .ok_or_else(|| "invalid extension contract optional capabilities".to_string())?;
    let strings = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "invalid extension contract optional capabilities".to_string())?;
    let expected = ["skills", "resources", "richToolResults"];
    if strings.as_slice() != expected
        || strings
            .iter()
            .any(|value| !valid_optional_capability(value, maximum_name_chars))
    {
        return Err("invalid extension contract optional capabilities".to_string());
    }
    Ok(())
}

fn validate_catalog_count(contract: &Value, directory: &Path) -> Result<(), String> {
    let catalog = super::io::read_bounded(
        &directory.join("builtin-plugins/catalog.json"),
        1_048_576,
        "builtin plugin catalog exceeds its size limit",
    )?;
    let catalog: Value = serde_json::from_slice(&catalog)
        .map_err(|_| "invalid builtin plugin catalog".to_string())?;
    let builtin_count = catalog["plugins"]
        .as_array()
        .filter(|plugins| plugins.len() <= 128)
        .ok_or_else(|| "invalid builtin plugin catalog".to_string())?
        .len() as u64;
    let maximum = contract["limits"]["maxExtensions"].as_u64().unwrap_or(0);
    let user = contract["limits"]["maxUserExtensions"]
        .as_u64()
        .unwrap_or(0);
    if maximum != user.saturating_add(builtin_count) {
        return Err("maxExtensions must equal maxUserExtensions plus builtin plugins".to_string());
    }
    Ok(())
}

fn contract_code_limit(contract: &Value) -> Result<usize, String> {
    contract["limits"]["maxContractCodeChars"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| "invalid extension contract code limit".to_string())
}

fn valid_protocol_code(value: &str, maximum_chars: usize) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && value.len() <= maximum_chars
        && bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
}

fn valid_optional_capability(value: &str, maximum_chars: usize) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && value.len() <= maximum_chars
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}
