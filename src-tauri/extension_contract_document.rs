use serde_json::{Map, Value};

pub fn generated_document_section(contract: &Value) -> Result<String, String> {
    let mut output = String::from("<!-- BEGIN GENERATED EXTENSION CONTRACT -->\n");
    output.push_str("### Contract surface\n\n| Category | Values |\n|---|---|\n");
    for (label, values) in [
        (
            "Capabilities",
            strings(array_value(contract, "capabilities")?)?,
        ),
        (
            "Core to host",
            strings(array(object(contract, "methods")?, "coreToHost")?)?,
        ),
        ("Events", strings(array_value(contract, "events")?)?),
        ("Effects", strings(array_value(contract, "effectClasses")?)?),
    ] {
        output.push_str(&format!("| {label} | `{}` |\n", values.join("`, `")));
    }

    output.push_str(
        "\n### Host to core\n\n| Method | Level | Kind | Rust budget (ms) |\n|---|---|---|---:|\n",
    );
    for method in array(object(contract, "methods")?, "hostToCore")? {
        let method = method.as_object().ok_or("invalid host method")?;
        let budget = method["rustBudgetMs"]
            .as_u64()
            .map_or_else(|| "n/a".to_string(), |value| value.to_string());
        output.push_str(&format!(
            "| `{}` | `{}` | `{}` | {budget} |\n",
            string(method, "name")?,
            string(method, "level")?,
            string(method, "kind")?,
        ));
    }

    numeric_table(&mut output, "Limits", object(contract, "limits")?)?;
    numeric_table(&mut output, "Timeouts", object(contract, "timeouts")?)?;

    output.push_str("\n### Errors\n\n| Category | Values |\n|---|---|\n");
    let errors = object(contract, "errors")?;
    for (label, values) in [
        (
            "Protocol reasons",
            strings(array(errors, "protocolReasons")?)?,
        ),
        ("Backend codes", strings(array(errors, "backendCodes")?)?),
    ] {
        output.push_str(&format!("| {label} | `{}` |\n", values.join("`, `")));
    }
    output.push_str("<!-- END GENERATED EXTENSION CONTRACT -->");
    Ok(output)
}

fn numeric_table(
    output: &mut String,
    heading: &str,
    values: &Map<String, Value>,
) -> Result<(), String> {
    output.push_str(&format!(
        "\n### {heading}\n\n| Name | Value |\n|---|---:|\n"
    ));
    for (name, value) in values {
        output.push_str(&format!(
            "| `{name}` | {} |\n",
            value.as_u64().ok_or("invalid numeric contract value")?
        ));
    }
    Ok(())
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing extension contract object: {name}"))
}

fn array<'a>(value: &'a Map<String, Value>, name: &str) -> Result<&'a [Value], String> {
    value
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing extension contract array: {name}"))
}

fn array_value<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], String> {
    value
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing extension contract array: {name}"))
}

fn string<'a>(value: &'a Map<String, Value>, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing extension contract string: {name}"))
}

fn strings(values: &[Value]) -> Result<Vec<&str>, String> {
    values
        .iter()
        .map(|value| value.as_str().ok_or("invalid contract string".to_string()))
        .collect()
}
