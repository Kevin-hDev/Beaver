use super::types::{ExtensionRecord, MAX_EXTENSIONS, MAX_MESSAGE_BYTES};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Envelope<'a> {
    version: u8,
    extensions: &'a [ExtensionRecord],
    recovery_snapshot: &'a Option<Vec<String>>,
}

pub(super) fn serialize(
    version: u8,
    records: &[ExtensionRecord],
    recovery_snapshot: &Option<Vec<String>>,
    existing: Option<Value>,
) -> Result<Vec<u8>, String> {
    if records.len() > MAX_EXTENSIONS {
        return Err("Trop d'extensions enregistrées.".to_string());
    }
    let candidate = serde_json::to_value(Envelope {
        version,
        extensions: records,
        recovery_snapshot,
    })
    .map_err(|_| unavailable())?;
    let value = match existing {
        Some(previous) => preserve_registry_unknown(previous, candidate)?,
        None => candidate,
    };
    serialize_value(&value)
}

pub(super) fn serialize_value(value: &Value) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| unavailable())?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("Registre d'extensions trop volumineux.".to_string());
    }
    Ok(bytes)
}

fn preserve_registry_unknown(previous: Value, candidate: Value) -> Result<Value, String> {
    let previous = previous.as_object().ok_or_else(unavailable)?;
    let candidate = candidate.as_object().ok_or_else(unavailable)?;
    let mut output = previous.clone();
    output.insert("version".to_string(), candidate["version"].clone());
    output.insert(
        "recoverySnapshot".to_string(),
        candidate["recoverySnapshot"].clone(),
    );
    let previous_entries = previous["extensions"].as_array().ok_or_else(unavailable)?;
    let candidate_entries = candidate["extensions"].as_array().ok_or_else(unavailable)?;
    if previous_entries.len() > MAX_EXTENSIONS || candidate_entries.len() > MAX_EXTENSIONS {
        return Err(unavailable());
    }
    let mut merged = Vec::with_capacity(candidate_entries.len());
    for entry in candidate_entries {
        let id = entry.pointer("/manifest/id").and_then(Value::as_str);
        let previous_entry = id.and_then(|id| {
            previous_entries
                .iter()
                .find(|item| item.pointer("/manifest/id").and_then(Value::as_str) == Some(id))
        });
        let mut record = match previous_entry {
            Some(previous_entry) => merge_objects(previous_entry, entry),
            None => entry.clone(),
        };
        // Les contributions viennent uniquement de l'Hôte au démarrage : une
        // ancienne clé inconnue ne doit pas contourner cette reconstruction.
        let object = record.as_object_mut().ok_or_else(unavailable)?;
        object.remove("contributions");
        merged.push(record);
    }
    output.insert("extensions".to_string(), Value::Array(merged));
    Ok(Value::Object(output))
}

fn merge_objects(previous: &Value, candidate: &Value) -> Value {
    let (Some(previous), Some(candidate)) = (previous.as_object(), candidate.as_object()) else {
        return candidate.clone();
    };
    let mut merged: Map<String, Value> = previous.clone();
    for (key, value) in candidate {
        let value = previous
            .get(key)
            .map(|old| merge_objects(old, value))
            .unwrap_or_else(|| value.clone());
        merged.insert(key.clone(), value);
    }
    Value::Object(merged)
}

fn unavailable() -> String {
    super::error_codes::REGISTRY_UNAVAILABLE.to_string()
}
