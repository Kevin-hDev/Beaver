use serde_json::Value;
use std::collections::BTreeSet;

const TOP_LEVEL_KEYS: &[&str] = &[
    "apiVersion",
    "modes",
    "contributionTypes",
    "primitives",
    "themeBases",
    "locales",
    "placementOperations",
    "placements",
    "protectedOccupants",
    "icons",
    "themeTokens",
    "limits",
    "validation",
    "loadingStages",
    "diagnosticCodes",
];

pub fn validate_keys(contract: &Value) -> Result<(), String> {
    exact_object(contract, TOP_LEVEL_KEYS, &[], "extension UI contract")?;
    object_array(contract, "placements")?
        .iter()
        .try_for_each(|value| {
            exact_object(
                value,
                &["key", "contributionType", "cardinality", "scope"],
                &["thirdPartyChatAllowed"],
                "extension UI placement",
            )
        })?;
    object_array(contract, "protectedOccupants")?
        .iter()
        .try_for_each(|value| {
            exact_object(
                value,
                &["placement", "occupant", "operations"],
                &[],
                "protected UI occupant",
            )
        })?;
    exact_object(
        &contract["validation"],
        &[
            "maxNumericLimit",
            "minOrder",
            "maxOrder",
            "themeValuePattern",
        ],
        &[],
        "extension UI validation",
    )
}

fn exact_object(
    value: &Value,
    required: &[&str],
    optional: &[&str],
    name: &str,
) -> Result<(), String> {
    let object = value.as_object().ok_or_else(|| format!("invalid {name}"))?;
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let required = required.iter().copied().collect::<BTreeSet<_>>();
    let allowed = required
        .iter()
        .copied()
        .chain(optional.iter().copied())
        .collect::<BTreeSet<_>>();
    if !required.is_subset(&actual) || !actual.is_subset(&allowed) {
        return Err(format!("invalid {name} keys"));
    }
    Ok(())
}

fn object_array<'a>(contract: &'a Value, key: &str) -> Result<&'a [Value], String> {
    contract[key]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("invalid extension UI {key}"))
}
