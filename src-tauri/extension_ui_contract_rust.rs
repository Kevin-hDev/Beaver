use serde_json::Value;
use std::collections::BTreeSet;

pub fn render(contract: &Value) -> Result<String, String> {
    let mut output = format!(
        "#[allow(dead_code)]\npub const EXTENSION_UI_API_VERSION: &str = {:?};\n",
        contract["apiVersion"]
            .as_str()
            .ok_or_else(|| "missing UI API version".to_string())?
    );
    for (name, key) in [
        ("UiMode", "modes"),
        ("UiContributionType", "contributionTypes"),
        ("UiPrimitive", "primitives"),
        ("UiThemeBase", "themeBases"),
    ] {
        render_enum(&mut output, name, array(contract, key)?)?;
    }
    for (name, key) in [
        ("UI_LOCALES", "locales"),
        ("UI_PLACEMENT_OPERATIONS", "placementOperations"),
        ("UI_PLACEMENT_KEYS", "placements"),
        ("UI_ICONS", "icons"),
        ("UI_THEME_TOKENS", "themeTokens"),
        ("UI_LOADING_STAGES", "loadingStages"),
        ("UI_DIAGNOSTIC_CODES", "diagnosticCodes"),
    ] {
        let values = if key == "placements" {
            array(contract, key)?
                .iter()
                .map(|value| value["key"].as_str())
                .collect::<Option<Vec<_>>>()
        } else {
            array(contract, key)?
                .iter()
                .map(Value::as_str)
                .collect::<Option<Vec<_>>>()
        }
        .ok_or_else(|| format!("invalid {key}"))?;
        render_slice(&mut output, name, &values);
    }
    super::rust_objects::render(&mut output, contract)?;
    for (name, value) in contract["limits"]
        .as_object()
        .ok_or_else(|| "missing UI limits".to_string())?
    {
        output.push_str(&format!(
            "#[allow(dead_code)]\npub const {}: usize = {};\n",
            constant(name),
            value
                .as_u64()
                .ok_or_else(|| "invalid UI limit".to_string())?
        ));
    }
    Ok(output)
}

fn render_enum(output: &mut String, name: &str, values: &[Value]) -> Result<(), String> {
    output.push_str(
        "#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]\n",
    );
    output.push_str(&format!("pub enum {name} {{\n"));
    let mut variants = BTreeSet::new();
    for value in values {
        let value = value
            .as_str()
            .ok_or_else(|| "invalid UI enum".to_string())?;
        let variant = variant(value);
        if !variants.insert(variant.clone()) {
            return Err("extension UI enum variants collide".to_string());
        }
        output.push_str(&format!(
            "    #[serde(rename = {value:?})]\n    {variant},\n"
        ));
    }
    output.push_str("}\n");
    Ok(())
}

fn render_slice(output: &mut String, name: &str, values: &[&str]) {
    let values = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const {name}: &[&str] = &[{values}];\n"
    ));
}

fn array<'a>(contract: &'a Value, key: &str) -> Result<&'a [Value], String> {
    contract[key]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing {key}"))
}

fn variant(value: &str) -> String {
    value
        .split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn constant(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_uppercase() && !output.is_empty() {
            output.push('_');
        }
        output.push(character.to_ascii_uppercase());
    }
    output
}
