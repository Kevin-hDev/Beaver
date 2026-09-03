use serde_json::Value;

pub fn render(contract: &Value) -> Result<String, String> {
    let mut output = String::from("<!-- BEGIN GENERATED EXTENSION UI CONTRACT -->\n");
    output.push_str("### UI contract surface\n\n| Category | Values |\n|---|---|\n");
    for (label, key) in [
        ("Modes", "modes"),
        ("Contribution types", "contributionTypes"),
        ("Primitives", "primitives"),
        ("Theme bases", "themeBases"),
        ("Locales", "locales"),
        ("Loading stages", "loadingStages"),
    ] {
        output.push_str(&format!(
            "| {label} | `{}` |\n",
            strings(contract, key)?.join("`, `")
        ));
    }
    output.push_str(
        "\n### UI placements\n\n| Key | Type | Cardinality | Scope |\n|---|---|---|---|\n",
    );
    for placement in array(contract, "placements")? {
        output.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            text(placement, "key")?,
            text(placement, "contributionType")?,
            text(placement, "cardinality")?,
            text(placement, "scope")?,
        ));
    }
    output.push_str("\n### UI limits\n\n| Name | Value |\n|---|---:|\n");
    for (name, value) in contract["limits"]
        .as_object()
        .ok_or_else(|| "missing UI limits".to_string())?
    {
        output.push_str(&format!(
            "| `{name}` | {} |\n",
            value
                .as_u64()
                .ok_or_else(|| "invalid UI limit".to_string())?
        ));
    }
    output.push_str("\n### Public UI tokens\n\n");
    output.push_str(&format!(
        "`{}`\n\n",
        strings(contract, "themeTokens")?.join("`, `")
    ));
    output.push_str("### UI diagnostics\n\n");
    output.push_str(&format!(
        "`{}`\n",
        strings(contract, "diagnosticCodes")?.join("`, `")
    ));
    output.push_str("<!-- END GENERATED EXTENSION UI CONTRACT -->");
    Ok(output)
}

fn strings<'a>(contract: &'a Value, key: &str) -> Result<Vec<&'a str>, String> {
    array(contract, key)?
        .iter()
        .map(|value| value.as_str().ok_or_else(|| format!("invalid {key}")))
        .collect()
}

fn array<'a>(contract: &'a Value, key: &str) -> Result<&'a [Value], String> {
    contract[key]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing {key}"))
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key].as_str().ok_or_else(|| format!("missing {key}"))
}
