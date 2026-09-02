use serde_json::Value;

pub fn render(output: &mut String, effects: &[Value]) -> Result<(), String> {
    let effects = effects
        .iter()
        .map(|value| value.as_str().ok_or("invalid contract string".to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    output.push_str(
        "#[derive(Debug, Clone, Copy, Default, serde::Serialize, PartialEq, Eq)]\n\
         #[serde(rename_all = \"kebab-case\")]\n\
         pub enum ExtensionEffect {\n",
    );
    for effect in &effects {
        if *effect == "unknown" {
            output.push_str("#[default]\n");
        }
        output.push_str(&format!("{},\n", rust_variant(effect)));
    }
    output.push_str("}\n#[cfg(test)]\nimpl ExtensionEffect {\npub const ALL: [Self; ");
    output.push_str(&effects.len().to_string());
    output.push_str("] = [");
    for effect in &effects {
        output.push_str(&format!("Self::{},", rust_variant(effect)));
    }
    output.push_str("];\n}\n");
    render_deserialize(output, &effects);
    Ok(())
}

fn render_deserialize(output: &mut String, effects: &[&str]) {
    output.push_str(
        "impl<'de> serde::Deserialize<'de> for ExtensionEffect {\n\
         fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {\n\
         let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;\n\
         Ok(match value.as_str() {\n",
    );
    for effect in effects {
        if *effect != "unknown" {
            output.push_str(&format!(
                "Some({effect:?}) => Self::{},\n",
                rust_variant(effect)
            ));
        }
    }
    output.push_str("_ => Self::Unknown,\n})\n}\n}\n");
}

fn rust_variant(value: &str) -> String {
    value
        .split(['-', '.'])
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_ascii_uppercase().to_string() + characters.as_str()
            })
        })
        .collect()
}
