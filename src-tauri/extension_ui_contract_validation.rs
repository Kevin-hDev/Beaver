use serde_json::Value;
use std::collections::BTreeSet;

const STRING_LISTS: &[&str] = &[
    "modes",
    "contributionTypes",
    "primitives",
    "themeBases",
    "locales",
    "placementOperations",
    "icons",
    "themeTokens",
    "loadingStages",
    "diagnosticCodes",
];

pub fn validate(contract: &Value) -> Result<(), String> {
    super::schema::validate_keys(contract)?;
    if contract["apiVersion"].as_str() != Some("1") {
        return Err("invalid extension UI API version".to_string());
    }
    for name in STRING_LISTS {
        unique_strings(contract, name)?;
    }
    validate_limits(contract)?;
    validate_placements(contract)?;
    validate_protections(contract)?;
    validate_theme_tokens(contract)?;
    validate_validation(contract)
}

fn validate_validation(contract: &Value) -> Result<(), String> {
    let validation = &contract["validation"];
    let min_order = validation["minOrder"]
        .as_i64()
        .ok_or_else(|| "invalid extension UI minimum order".to_string())?;
    let max_order = validation["maxOrder"]
        .as_i64()
        .ok_or_else(|| "invalid extension UI maximum order".to_string())?;
    let pattern = validation["themeValuePattern"]
        .as_str()
        .filter(|value| !value.is_empty() && value.len() <= 256)
        .ok_or_else(|| "invalid extension UI theme pattern".to_string())?;
    if min_order >= max_order || pattern.bytes().any(|byte| byte.is_ascii_control()) {
        return Err("invalid extension UI validation bounds".to_string());
    }
    Ok(())
}

fn validate_limits(contract: &Value) -> Result<(), String> {
    let maximum = contract["validation"]["maxNumericLimit"]
        .as_u64()
        .filter(|value| *value > 0)
        .ok_or_else(|| "invalid extension UI numeric maximum".to_string())?;
    let limits = contract["limits"]
        .as_object()
        .ok_or_else(|| "missing extension UI limits".to_string())?;
    for (name, value) in limits {
        value
            .as_u64()
            .filter(|value| *value > 0 && *value <= maximum)
            .ok_or_else(|| format!("extension UI limit out of range: {name}"))?;
    }
    let limit = |name: &str| {
        limits[name]
            .as_u64()
            .ok_or_else(|| format!("missing extension UI limit: {name}"))
    };
    let theoretical_contributions = limit("maxContributionsPerExtension")?
        .checked_mul(128)
        .ok_or_else(|| "extension UI contribution limit overflow".to_string())?;
    if limit("maxGlobalStandardContributions")? > theoretical_contributions {
        return Err("invalid global extension UI contribution limit".to_string());
    }
    let theoretical_bytes = limit("maxUiBytesPerExtension")?
        .checked_mul(128)
        .ok_or_else(|| "extension UI byte limit overflow".to_string())?;
    if limit("maxGlobalUiBytes")? > theoretical_bytes {
        return Err("invalid global extension UI byte limit".to_string());
    }
    Ok(())
}

fn validate_placements(contract: &Value) -> Result<(), String> {
    let placements = contract["placements"]
        .as_array()
        .filter(|values| !values.is_empty() && values.len() <= 32)
        .ok_or_else(|| "invalid extension UI placements".to_string())?;
    let contribution_types = string_set(contract, "contributionTypes")?;
    let mut keys = BTreeSet::new();
    for placement in placements {
        let key = placement["key"]
            .as_str()
            .filter(|value| valid_name(value))
            .ok_or_else(|| "invalid extension UI placement key".to_string())?;
        let third_party_chat = placement.get("thirdPartyChatAllowed");
        if third_party_chat.is_some_and(|value| !value.is_boolean())
            || !keys.insert(key)
            || !placement["contributionType"]
                .as_str()
                .is_some_and(|value| contribution_types.contains(value))
            || placement["cardinality"].as_str() != Some("multiple")
            || !matches!(placement["scope"].as_str(), Some("global" | "session"))
        {
            return Err("invalid extension UI placement".to_string());
        }
    }
    Ok(())
}

fn validate_protections(contract: &Value) -> Result<(), String> {
    let placement_keys = contract["placements"]
        .as_array()
        .ok_or_else(|| "invalid extension UI placements".to_string())?
        .iter()
        .filter_map(|value| value["key"].as_str())
        .collect::<BTreeSet<_>>();
    let operations = string_set(contract, "placementOperations")?;
    let protections = contract["protectedOccupants"]
        .as_array()
        .filter(|values| values.len() <= 32)
        .ok_or_else(|| "invalid protected UI occupants".to_string())?;
    let mut identities = BTreeSet::new();
    for protection in protections {
        let placement = protection["placement"].as_str().unwrap_or_default();
        let occupant = protection["occupant"].as_str().unwrap_or_default();
        let protected = protection["operations"]
            .as_array()
            .ok_or_else(|| "invalid protected UI occupant operations".to_string())?;
        if !placement_keys.contains(placement)
            || !valid_name(occupant)
            || !identities.insert((placement, occupant))
            || protected.is_empty()
            || protected.iter().any(|value| {
                !value
                    .as_str()
                    .is_some_and(|value| operations.contains(value))
            })
        {
            return Err("invalid protected UI occupant".to_string());
        }
    }
    Ok(())
}

fn validate_theme_tokens(contract: &Value) -> Result<(), String> {
    let maximum = contract["limits"]["maxThemeTokens"].as_u64().unwrap_or(0) as usize;
    let tokens = contract["themeTokens"]
        .as_array()
        .ok_or_else(|| "invalid UI tokens".to_string())?;
    if tokens.len() > maximum
        || tokens.iter().any(|value| {
            !value
                .as_str()
                .is_some_and(|token| token.strip_prefix("--").is_some_and(valid_name))
        })
    {
        return Err("invalid extension UI theme token".to_string());
    }
    Ok(())
}

fn unique_strings(contract: &Value, name: &str) -> Result<(), String> {
    let values = contract[name]
        .as_array()
        .filter(|values| !values.is_empty() && values.len() <= 128)
        .ok_or_else(|| format!("invalid extension UI list: {name}"))?;
    let unique = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<BTreeSet<_>>>()
        .ok_or_else(|| format!("invalid extension UI list: {name}"))?;
    if unique.len() != values.len() {
        return Err(format!("duplicate extension UI value: {name}"));
    }
    Ok(())
}

fn string_set<'a>(contract: &'a Value, name: &str) -> Result<BTreeSet<&'a str>, String> {
    contract[name]
        .as_array()
        .ok_or_else(|| format!("invalid extension UI list: {name}"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("invalid extension UI list: {name}"))
        })
        .collect()
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}
