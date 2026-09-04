use serde_json::{Map, Value};

pub fn render(contract: &Value) -> Result<String, String> {
    let diagnostics = object(contract, "diagnostics")?;
    let errors = object(contract, "errors")?;
    let mut output = format!(
        "#[allow(dead_code)]\npub const BEAVER_API_VERSION: &str = {:?};\n",
        root_string(contract, "apiVersion")?
    );
    render_numbers(&mut output, object(contract, "limits")?)?;
    render_numbers(&mut output, object(contract, "timeouts")?)?;
    for (source, name) in [
        (array(diagnostics, "hostCodes")?, "HOST_DIAGNOSTIC_CODES"),
        (
            array(diagnostics, "runtimeCodes")?,
            "RUNTIME_DIAGNOSTIC_CODES",
        ),
    ] {
        render_named_strings(&mut output, source, "DIAGNOSTIC_", name)?;
    }
    let diagnostics_all = array(diagnostics, "hostCodes")?
        .iter()
        .chain(array(diagnostics, "runtimeCodes")?)
        .cloned()
        .collect::<Vec<_>>();
    output.push_str("#[cfg(test)]\n");
    render_slice(&mut output, "DIAGNOSTIC_CODES", &diagnostics_all)?;
    output.push_str("pub mod backend_error_codes {\n");
    render_named_strings(&mut output, array(errors, "backendCodes")?, "", "ALL")?;
    output.push_str("}\n");
    super::r0_enum_renderer::render(&mut output, contract)?;
    for (name, values) in [
        ("PROTOCOL_ERROR_REASONS", array(errors, "protocolReasons")?),
        (
            "CORE_TO_HOST_METHODS",
            array(object(contract, "methods")?, "coreToHost")?,
        ),
        ("EXTENSION_EVENTS", array_value(contract, "events")?),
    ] {
        render_slice(&mut output, name, values)?;
    }
    super::enum_renderer::render(
        &mut output,
        "HostState",
        array_value(contract, "hostStates")?,
    )?;
    render_named_strings(
        &mut output,
        array_value(contract, "loadStages")?,
        "HOST_LOAD_STAGE_",
        "HOST_LOAD_STAGES",
    )?;
    let effects = array_value(contract, "effectClasses")?;
    render_slice(&mut output, "EXTENSION_EFFECT_CLASSES", effects)?;
    super::effect_renderer::render(&mut output, effects)?;
    render_host_methods(
        &mut output,
        array(object(contract, "methods")?, "hostToCore")?,
    )?;
    Ok(output)
}

fn render_host_methods(output: &mut String, methods: &[Value]) -> Result<(), String> {
    let mut rendered = Vec::with_capacity(methods.len());
    for method in methods {
        let method = method
            .as_object()
            .ok_or_else(|| "invalid host method contract".to_string())?;
        let name = string(method, "name")?;
        let level = string(method, "level")?;
        let kind = string(method, "kind")?;
        let budget = match method.get("rustBudgetMs") {
            Some(Value::Null) => "None".to_string(),
            Some(value) => format!(
                "Some({})",
                value
                    .as_u64()
                    .ok_or_else(|| "invalid host method budget".to_string())?
            ),
            None => return Err("missing host method budget".to_string()),
        };
        rendered.push(format!("({name:?}, {level:?}, {kind:?}, {budget})"));
    }
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const HOST_TO_CORE_METHODS: &[(&str, &str, &str, Option<usize>)] = &[{}];\n",
        rendered.join(", ")
    ));
    let notifications = methods
        .iter()
        .filter(|method| method["kind"] == "notification")
        .collect::<Vec<_>>();
    if notifications.len() != 1 {
        return Err("expected one host load stage notification".to_string());
    }
    let notification = notifications[0]
        .as_object()
        .ok_or_else(|| "invalid host method contract".to_string())?;
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const HOST_LOAD_STAGE_METHOD: &str = {:?};\n",
        string(notification, "name")?
    ));
    Ok(())
}

fn render_numbers(output: &mut String, values: &Map<String, Value>) -> Result<(), String> {
    for (name, value) in values {
        output.push_str(&format!(
            "#[allow(dead_code)]\npub const {}: usize = {};\n",
            constant(name),
            value.as_u64().ok_or("invalid numeric contract value")?
        ));
    }
    Ok(())
}

fn render_named_strings(
    output: &mut String,
    values: &[Value],
    prefix: &str,
    slice: &str,
) -> Result<(), String> {
    for value in strings(values)? {
        output.push_str(&format!(
            "#[allow(dead_code)]\npub const {prefix}{}: &str = {value:?};\n",
            constant(value.trim_start_matches("extensions_"))
        ));
    }
    let names = strings(values)?
        .into_iter()
        .map(|value| {
            format!(
                "{prefix}{}",
                constant(value.trim_start_matches("extensions_"))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const {slice}: &[&str] = &[{names}];\n"
    ));
    Ok(())
}

fn render_slice(output: &mut String, name: &str, values: &[Value]) -> Result<(), String> {
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

fn strings(values: &[Value]) -> Result<Vec<&str>, String> {
    values
        .iter()
        .map(|value| value.as_str().ok_or("invalid contract string".to_string()))
        .collect()
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing extension contract object: {name}"))
}

fn string<'a>(value: &'a Map<String, Value>, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing extension contract string: {name}"))
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

fn root_string<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing extension contract string: {name}"))
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
