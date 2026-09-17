use serde_json::Value;

pub fn render(methods: &[Value]) -> Result<String, String> {
    let mut output = String::new();
    for method in methods
        .iter()
        .filter(|method| method["kind"] == "notification")
    {
        let name = method["name"]
            .as_str()
            .ok_or_else(|| "invalid host notification contract".to_string())?;
        output.push_str(&format!(
            "#[allow(dead_code)]\npub const {}_METHOD: &str = {name:?};\n",
            constant(name)
        ));
    }
    Ok(output)
}

fn constant(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}
