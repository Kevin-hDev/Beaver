use serde_json::Value;

const MAX_DEPTH: usize = 16;
const MAX_ELEMENTS: usize = 256;
const INVALID: &str = "schéma MCP invalide";

pub fn validate(value: &Value) -> Result<(), String> {
    let mut elements = 0;
    visit(value, 0, &mut elements)
}

fn visit(value: &Value, depth: usize, elements: &mut usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(INVALID.to_string());
    }
    match value {
        Value::Object(object) => {
            *elements = elements.saturating_add(object.len());
            if *elements > MAX_ELEMENTS {
                return Err(INVALID.to_string());
            }
            for child in object.values() {
                visit(child, depth + 1, elements)?;
            }
        }
        Value::Array(array) => {
            *elements = elements.saturating_add(array.len());
            if *elements > MAX_ELEMENTS {
                return Err(INVALID.to_string());
            }
            for child in array {
                visit(child, depth + 1, elements)?;
            }
        }
        _ => {}
    }
    Ok(())
}
