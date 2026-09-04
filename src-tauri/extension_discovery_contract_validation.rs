use serde_json::{Map, Value};
use std::collections::BTreeSet;

const IMPORTED_LIMITS: [&str; 5] = [
    "maxExtensions",
    "maxToolsPerExtension",
    "maxSkillsPerExtension",
    "maxResourcesPerExtension",
    "maxIdentifierChars",
];
const TOOL_NAMES: [&str; 3] = [
    "list_extensions",
    "inspect_extensions",
    "load_extension_resource",
];
const LIMITS: [(&str, u64); 9] = [
    ("contextThresholdPercent", 10),
    ("unknownContextTokens", 20_000),
    ("maxInspectedExtensions", 4),
    ("maxProjectedExtensionNameJsonBytes", 64),
    ("maxProjectedExtensionDescriptionJsonBytes", 240),
    ("maxProjectedContributionNameJsonBytes", 96),
    ("maxProjectedContributionSummaryJsonBytes", 120),
    ("maxCompactCatalogBytes", 32_768),
    ("maxSerializedResultBytes", 393_216),
];

pub fn validate(discovery: &Value, host: &Value) -> Result<(), String> {
    let root = object(discovery, "extension discovery contract")?;
    exact_keys(root, ["toolNames", "imports", "limits"])?;
    strings(root.get("toolNames"), &TOOL_NAMES)?;
    strings(root.get("imports"), &IMPORTED_LIMITS)?;
    let limits = root
        .get("limits")
        .and_then(Value::as_object)
        .ok_or_else(|| "invalid extension discovery contract limits".to_string())?;
    exact_keys(limits, LIMITS.map(|(name, _)| name))?;
    let maximum = host
        .pointer("/validation/maxNumericLimit")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| "invalid host contract numeric maximum".to_string())?;
    let host_limits = object(&host["limits"], "host extension limits")?;
    for (name, expected) in LIMITS {
        let value = limits.get(name).and_then(Value::as_u64);
        if value != Some(expected) || expected == 0 || expected > maximum {
            return Err(format!(
                "invalid extension discovery contract limit: {name}"
            ));
        }
        if host["limits"].get(name).is_some() {
            return Err("extension discovery contract duplicates host limit authority".to_string());
        }
    }
    for name in IMPORTED_LIMITS {
        let valid = host_limits
            .get(name)
            .and_then(Value::as_u64)
            .is_some_and(|value| value > 0 && value <= maximum && usize::try_from(value).is_ok());
        if !valid {
            return Err("unknown extension discovery contract import".to_string());
        }
    }
    if limits["contextThresholdPercent"].as_u64() > Some(100)
        || limits["maxInspectedExtensions"].as_u64() > host_limits["maxExtensions"].as_u64()
        // maxMessageBytes is only a transport relation; it is not a generated
        // discovery constant because no discovery behavior consumes its value.
        || limits["maxSerializedResultBytes"].as_u64() > host_limits["maxMessageBytes"].as_u64()
    {
        return Err("extension discovery contract limit is out of bounds".to_string());
    }
    super::budget::validate(limits, host_limits)?;
    Ok(())
}

fn object<'a>(value: &'a Value, subject: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("invalid {subject}"))
}

fn exact_keys<'a>(
    values: &Map<String, Value>,
    expected: impl IntoIterator<Item = &'a str>,
) -> Result<(), String> {
    let expected = expected.into_iter().collect::<BTreeSet<_>>();
    if values.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return Err("unknown or missing extension discovery contract key".to_string());
    }
    Ok(())
}

fn strings(value: Option<&Value>, expected: &[&str]) -> Result<(), String> {
    let values = value
        .and_then(Value::as_array)
        .ok_or_else(|| "invalid extension discovery contract names".to_string())?;
    let parsed = values
        .iter()
        .map(Value::as_str)
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "invalid extension discovery contract names".to_string())?;
    if parsed.len() != expected.len()
        || parsed.iter().collect::<BTreeSet<_>>().len() != parsed.len()
        || parsed
            .iter()
            .zip(expected)
            .any(|(actual, expected)| actual != expected)
    {
        return Err("invalid extension discovery contract names".to_string());
    }
    Ok(())
}
