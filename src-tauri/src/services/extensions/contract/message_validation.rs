use super::error_codes;
use super::types::MAX_MESSAGE_BYTES;
use serde_json::Value;

const MAX_JSON_DEPTH: usize = 20;
const MAX_JSON_COLLECTION_ITEMS: usize = 4_096;
const MAX_JSON_NODES: usize = 16_384;

/// Place réservée à l'enveloppe JSON-RPC qui transporte une charge sortante :
/// `jsonrpc`, `id`, `method`, `params.name` et `params.context.workingDirectory`.
/// Sans cette réserve, une charge acceptée ici serait refusée à l'écriture, avec
/// une erreur de transport que l'appelant ne peut pas relier à sa demande.
pub const MAX_REQUEST_ENVELOPE_BYTES: usize = 2_048;

pub fn validate(value: &Value) -> Result<(), String> {
    validate_within(value, 0)
}

pub fn validate_request_payload(value: &Value) -> Result<(), String> {
    validate_within(value, MAX_REQUEST_ENVELOPE_BYTES)
}

fn validate_within(value: &Value, reserved: usize) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|_| error_codes::REQUEST_INVALID.to_string())?;
    let mut nodes = 0;
    if bytes.len().saturating_add(reserved) > MAX_MESSAGE_BYTES
        || !shape_is_bounded(value, 0, &mut nodes)
    {
        return Err(error_codes::REQUEST_TOO_LARGE.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn payload_of(text_bytes: usize) -> Value {
        json!({ "paragraphs": ["a".repeat(text_bytes)] })
    }

    #[test]
    fn a_payload_that_passes_validation_still_fits_its_transport_envelope() {
        let payload = payload_of(MAX_MESSAGE_BYTES - MAX_REQUEST_ENVELOPE_BYTES - 64);
        assert!(validate_request_payload(&payload).is_ok());

        let request = json!({
            "jsonrpc": "2.0",
            "id": "0".repeat(36),
            "method": "tool.call",
            "params": {
                "name": "beaver.office.presentations.create",
                "arguments": payload,
                "context": { "workingDirectory": "x".repeat(1_024) },
            },
        });
        let encoded = serde_json::to_vec(&request).expect("requête sérialisable");
        assert!(
            encoded.len() < MAX_MESSAGE_BYTES,
            "enveloppe de {} octets au-delà de la limite de transport",
            encoded.len(),
        );
    }

    #[test]
    fn a_payload_inside_the_reserved_window_is_refused_with_a_translatable_code() {
        let payload = payload_of(MAX_MESSAGE_BYTES - 64);
        assert!(validate(&payload).is_ok());
        assert_eq!(
            validate_request_payload(&payload),
            Err(error_codes::REQUEST_TOO_LARGE.to_string()),
        );
    }

    #[test]
    fn an_incoming_message_keeps_the_full_transport_budget() {
        let payload = payload_of(MAX_MESSAGE_BYTES - 64);
        assert!(validate(&payload).is_ok());
    }

    #[test]
    fn a_deeply_nested_payload_is_refused() {
        let mut value = json!("leaf");
        for _ in 0..(MAX_JSON_DEPTH + 2) {
            value = json!([value]);
        }
        assert_eq!(
            validate(&value),
            Err(error_codes::REQUEST_TOO_LARGE.to_string()),
        );
    }
}
