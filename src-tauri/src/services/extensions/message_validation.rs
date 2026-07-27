use super::types::MAX_MESSAGE_BYTES;
use serde_json::Value;

const MAX_JSON_DEPTH: usize = 20;
const MAX_JSON_COLLECTION_ITEMS: usize = 4_096;
const MAX_JSON_NODES: usize = 16_384;

pub fn validate(value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| "Message d'extension invalide.")?;
    let mut nodes = 0;
    if bytes.len() > MAX_MESSAGE_BYTES || !shape_is_bounded(value, 0, &mut nodes) {
        return Err("Message d'extension trop volumineux.".to_string());
    }
    Ok(())
}

fn shape_is_bounded(value: &Value, depth: usize, nodes: &mut usize) -> bool {
    *nodes = nodes.saturating_add(1);
    if depth > MAX_JSON_DEPTH || *nodes > MAX_JSON_NODES {
        return false;
    }
    match value {
        Value::Array(items) => {
            items.len() <= MAX_JSON_COLLECTION_ITEMS
                && items
                    .iter()
                    .all(|item| shape_is_bounded(item, depth + 1, nodes))
        }
        Value::Object(items) => {
            items.len() <= MAX_JSON_COLLECTION_ITEMS
                && items
                    .values()
                    .all(|item| shape_is_bounded(item, depth + 1, nodes))
        }
        _ => true,
    }
}
