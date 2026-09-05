use serde_json::Value;

pub fn render(output: &mut String, contract: &Value) -> Result<(), String> {
    for (name, field) in [
        ("OptionalExtensionCapability", "optionalCapabilities"),
        ("ExtensionContributionType", "contributionTypes"),
        ("ExtensionResultBlockType", "resultBlockTypes"),
        ("ExtensionResultFilePurpose", "resultFilePurposes"),
        ("ExtensionResourceType", "resourceTypes"),
    ] {
        let values = contract
            .get(field)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("missing extension contract array: {field}"))?;
        super::enum_renderer::render(output, name, values)?;
    }
    Ok(())
}
