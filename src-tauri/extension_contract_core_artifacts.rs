use serde_json::Value;

pub fn render_typescript(contract: &Value, host_methods: &[Value]) -> Result<String, String> {
    let methods = host_methods
        .iter()
        .filter(|method| method.get("capability").is_some())
        .cloned()
        .collect::<Vec<_>>();
    let transport = contract["transport"]
        .as_object()
        .ok_or_else(|| "missing extension contract object: transport".to_string())?;
    Ok(format!(
        "export const CORE_API_METHODS = {} as const;\nexport const CORE_API_TRANSPORT = Object.freeze({} as const);\n",
        serde_json::to_string(&methods).map_err(|_| "cannot render core API methods")?,
        serde_json::to_string(transport).map_err(|_| "cannot render core API transport")?,
    ))
}
