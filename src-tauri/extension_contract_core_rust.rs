use serde_json::Value;

pub fn render(output: &mut String, methods: &[Value]) -> Result<(), String> {
    render_method_idempotence(output, methods)?;
    output.push_str(
        "#[allow(dead_code)]\npub struct CoreApiParamContract { pub name: &'static str, pub kind: &'static str, pub required: bool, pub limit: Option<&'static str> }\n",
    );
    output.push_str(
        "#[allow(dead_code)]\npub struct CoreApiMethodContract { pub name: &'static str, pub capability: &'static str, pub requires_context: bool, pub idempotent: bool, pub effects: &'static [&'static str], pub params: &'static [CoreApiParamContract], pub result: &'static str }\n",
    );
    let mut rendered = Vec::new();
    for method in methods
        .iter()
        .filter(|method| method.get("capability").is_some())
    {
        let params = method["params"]
            .as_array()
            .ok_or_else(|| "invalid core API parameters".to_string())?
            .iter()
            .map(render_param)
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let effects = method["effects"]
            .as_array()
            .ok_or_else(|| "invalid core API effects".to_string())?
            .iter()
            .map(|effect| {
                effect
                    .as_str()
                    .map(|effect| format!("{effect:?}"))
                    .ok_or_else(|| "invalid core API effect".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        rendered.push(format!(
            "CoreApiMethodContract {{ name: {:?}, capability: {:?}, requires_context: {}, idempotent: {}, effects: &[{}], params: &[{}], result: {:?} }}",
            string(method, "name")?,
            string(method, "capability")?,
            boolean(method, "requiresContext")?,
            boolean(method, "idempotent")?,
            effects,
            params,
            string(method, "result")?,
        ));
    }
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const CORE_API_METHODS: &[CoreApiMethodContract] = &[{}];\n",
        rendered.join(", ")
    ));
    Ok(())
}

fn render_method_idempotence(output: &mut String, methods: &[Value]) -> Result<(), String> {
    let values = methods
        .iter()
        .filter(|method| method["kind"] == "request")
        .map(|method| {
            Ok(format!(
                "({:?}, {})",
                method["name"].as_str().ok_or("invalid host method")?,
                method["idempotent"]
                    .as_bool()
                    .ok_or("invalid host method idempotence")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const HOST_TO_CORE_IDEMPOTENCE: &[(&str, bool)] = &[{}];\n",
        values.join(", ")
    ));
    Ok(())
}

fn render_param(param: &Value) -> Result<String, String> {
    let limit = match param.get("limit") {
        Some(limit) => format!(
            "Some({:?})",
            limit.as_str().ok_or("invalid parameter limit")?
        ),
        None => "None".to_string(),
    };
    Ok(format!(
        "CoreApiParamContract {{ name: {:?}, kind: {:?}, required: {}, limit: {} }}",
        string(param, "name")?,
        string(param, "type")?,
        boolean(param, "required")?,
        limit,
    ))
}

fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, String> {
    value[name]
        .as_str()
        .ok_or_else(|| format!("invalid core API field: {name}"))
}

fn boolean(value: &Value, name: &str) -> Result<bool, String> {
    value[name]
        .as_bool()
        .ok_or_else(|| format!("invalid core API field: {name}"))
}
