use serde_json::Value;
use std::collections::BTreeSet;

pub fn render(output: &mut String, name: &str, values: &[Value]) -> Result<(), String> {
    output.push_str("#[allow(dead_code)]\n#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]\n");
    output.push_str(&format!("pub enum {name} {{\n"));
    let strings = strings(values)?;
    let variants = strings
        .iter()
        .map(|value| rust_variant(value))
        .collect::<Vec<_>>();
    if variants.iter().collect::<BTreeSet<_>>().len() != variants.len() {
        return Err("extension contract enum variants collide".to_string());
    }
    for (value, variant) in strings.iter().zip(&variants) {
        output.push_str(&format!(
            "    #[serde(rename = {value:?})]\n    {variant},\n"
        ));
    }
    output.push_str("}\n");
    output.push_str(&format!("#[allow(dead_code)]\nimpl {name} {{\n    pub const fn as_str(&self) -> &'static str {{\n        match self {{\n"));
    for (value, variant) in strings.iter().zip(&variants) {
        output.push_str(&format!("            Self::{variant} => {value:?},\n"));
    }
    output.push_str("        }\n    }\n}\n");
    Ok(())
}

fn strings(values: &[Value]) -> Result<Vec<&str>, String> {
    values
        .iter()
        .map(|value| value.as_str().ok_or("invalid contract string".to_string()))
        .collect()
}

fn rust_variant(value: &str) -> String {
    value
        .split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + characters.as_str())
                .unwrap_or_default()
        })
        .collect()
}
