use serde_json::{Map, Value};

pub fn render(discovery: &Value, host: &Value) -> Result<String, String> {
    let mut output = String::new();
    render_numbers(&mut output, object(discovery, "limits")?)?;
    render_imports(&mut output, discovery, host)?;
    render_strings(
        &mut output,
        "DISCOVERY_TOOL_NAMES",
        array(discovery, "toolNames")?,
    )?;
    Ok(output)
}

fn render_imports(output: &mut String, discovery: &Value, host: &Value) -> Result<(), String> {
    let host_limits = object(host, "limits")?;
    for name in strings(array(discovery, "imports")?)? {
        let value = host_limits
            .get(name)
            .and_then(Value::as_u64)
            .ok_or_else(|| "invalid imported extension limit".to_string())?;
        output.push_str(&format!(
            "#[allow(dead_code)]\npub const HOST_{}: usize = {value};\n",
            constant(name)
        ));
    }
    Ok(())
}

fn render_numbers(output: &mut String, values: &Map<String, Value>) -> Result<(), String> {
    for (name, value) in values {
        output.push_str(&format!(
            "#[allow(dead_code)]\npub const {}: usize = {};\n",
            constant(name),
            value.as_u64().ok_or("invalid discovery numeric value")?
        ));
    }
    Ok(())
}

fn render_strings(output: &mut String, name: &str, values: &[Value]) -> Result<(), String> {
    let values = strings(values)?
        .into_iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const {name}: &[&str] = &[{values}];\n"
    ));
    Ok(())
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| "invalid discovery contract object".to_string())
}

fn array<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], String> {
    value
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| "invalid discovery contract array".to_string())
}

fn strings(values: &[Value]) -> Result<Vec<&str>, String> {
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or("invalid discovery contract string".to_string())
        })
        .collect()
}

fn constant(name: &str) -> String {
    let mut output = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() && !output.is_empty() {
            output.push('_');
        }
        output.push(if character == '-' || character == '.' {
            '_'
        } else {
            character.to_ascii_uppercase()
        });
    }
    output
}
