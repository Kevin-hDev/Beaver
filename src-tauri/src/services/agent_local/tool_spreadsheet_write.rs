use crate::services::agent_local::security::validate_write_path;
use crate::services::agent_local::tool_office_limits::{
    ensure_source_size, validate_zip_archive, MAX_SPREADSHEET_SOURCE_BYTES,
};
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub fn coerce_to_array(value: &Value) -> Option<Vec<Value>> {
    if let Some(arr) = value.as_array() {
        return Some(arr.clone());
    }
    if value.is_object() {
        return Some(vec![value.clone()]);
    }
    if let Some(s) = value.as_str() {
        if let Some(result) = try_parse_json_array(s) {
            return Some(result);
        }
        let unescaped = s.replace("\\\"", "\"").replace("\\\\", "\\");
        if unescaped != s {
            if let Some(result) = try_parse_json_array(&unescaped) {
                return Some(result);
            }
        }
    }
    None
}

fn try_parse_json_array(s: &str) -> Option<Vec<Value>> {
    let trimmed = s.trim();
    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
        return coerce_parsed(parsed);
    }
    None
}

fn coerce_parsed(val: Value) -> Option<Vec<Value>> {
    if let Some(arr) = val.as_array() {
        return Some(arr.clone());
    }
    if val.is_object() {
        return Some(vec![val]);
    }
    None
}

pub fn validate_spreadsheet_input(path: &Path) -> Result<(), String> {
    ensure_source_size(path, MAX_SPREADSHEET_SOURCE_BYTES, "Spreadsheet")?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if matches!(ext.as_str(), "xlsx" | "xlsm" | "ods") {
        validate_zip_archive(path, "Spreadsheet")?;
    }
    Ok(())
}

pub fn describe_value_type(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(_) => "bool".into(),
        Value::Number(_) => "number".into(),
        Value::String(s) => format!("string(len={}): {}...", s.len(), &s[..s.len().min(120)]),
        Value::Array(a) => format!("array(len={})", a.len()),
        Value::Object(o) => {
            let keys: Vec<&str> = o.keys().map(|k| k.as_str()).collect();
            format!("object(keys={})", keys.join(","))
        }
    }
}

pub async fn write_spreadsheet(path: &str, operations: &Value, working_dir: &Path) -> ToolResult {
    if path.is_empty() {
        return ToolResult::err("Le paramètre 'path' est requis");
    }

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_write_path(&resolved) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let ops = match coerce_to_array(operations) {
        Some(arr) => arr,
        None => {
            return ToolResult::err(format!(
                "Le paramètre 'operations' doit être un tableau d'opérations. Reçu: {}",
                describe_value_type(operations)
            ))
        }
    };

    let count = ops.len();

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext != "xlsx" {
        return ToolResult::err("Seul le format .xlsx est supporté pour l'écriture");
    }

    let result = if validated.exists() {
        super::tool_spreadsheet_write_edit::edit_xlsx(&validated, &ops)
    } else {
        super::tool_spreadsheet_write_new::create_xlsx(&validated, &ops)
    };

    match result {
        Ok(_) => ToolResult::ok(format!(
            "Fichier écrit: {} ({} opérations)",
            validated.display(),
            count
        )),
        Err(e) => ToolResult::err(e),
    }
}

/// Convertit une référence de cellule "A1", "B2", "AA100" en (row, col) 0-based.
/// row est u32 (pour rust_xlsxwriter), col est u16.
pub fn parse_cell_ref(cell: &str) -> Option<(u32, u16)> {
    let cell = cell.trim().replace('$', "").to_uppercase();
    let split_pos = cell.find(|c: char| c.is_ascii_digit())?;
    let col_str = &cell[..split_pos];
    let row_str = &cell[split_pos..];

    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }

    let col_idx = super::tool_spreadsheet_read::col_letters_to_index(col_str);
    let row_num: u32 = row_str.parse().ok()?;
    if row_num == 0 {
        return None;
    }

    let col = u16::try_from(col_idx).ok()?;
    Some((row_num - 1, col))
}
