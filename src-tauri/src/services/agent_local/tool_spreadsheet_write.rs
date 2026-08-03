use crate::services::agent_local::security::validate_write_path;
use crate::services::agent_local::tool_office_array::{coerce, ArrayInputError};
use crate::services::agent_local::tool_office_limits::{
    ensure_source_size, validate_zip_archive, MAX_SPREADSHEET_OPERATIONS,
    MAX_SPREADSHEET_SOURCE_BYTES,
};
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

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
        Value::String(value) => {
            let preview: String = value.chars().take(120).collect();
            format!("string(len={}): {preview}...", value.chars().count())
        }
        Value::Array(a) => format!("array(len={})", a.len()),
        Value::Object(o) => {
            const MAX_DESCRIBED_KEYS: usize = 12;
            let keys = o
                .keys()
                .take(MAX_DESCRIBED_KEYS)
                .map(String::as_str)
                .collect::<Vec<_>>();
            let suffix = if o.len() > MAX_DESCRIBED_KEYS {
                ",…"
            } else {
                ""
            };
            format!("object(keys={}{suffix})", keys.join(","))
        }
    }
}

pub async fn write_spreadsheet(path: &str, operations: &Value, working_dir: &Path) -> ToolResult {
    if path.is_empty() {
        return ToolResult::validation(
            "spreadsheet_path_required",
            "Le paramètre 'path' est requis",
        );
    }

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_write_path(&resolved, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "spreadsheet_parent_not_found",
                "spreadsheet_write_denied",
                "invalid_spreadsheet_path",
            )
        }
    };

    let ops = match coerce(operations, MAX_SPREADSHEET_OPERATIONS) {
        Ok(operations) => operations,
        Err(ArrayInputError::Invalid) => {
            return ToolResult::validation("spreadsheet_operations_invalid", format!(
                "Le paramètre 'operations' doit être un tableau d'opérations. Reçu: {}",
                describe_value_type(operations)
            ))
        }
        Err(ArrayInputError::TooMany) => return ToolResult::validation(
            "spreadsheet_operation_limit_exceeded",
            format!("Trop d'opérations (maximum {MAX_SPREADSHEET_OPERATIONS})"),
        ),
    };

    let count = ops.len();

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    if ext != "xlsx" {
        return ToolResult::validation(
            "spreadsheet_format_unsupported",
            "Seul le format .xlsx est supporté pour l'écriture",
        );
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
        Err(error) => error.into_tool_result(),
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

    let col_idx = super::tool_spreadsheet_range::column_index(col_str)?;
    let row_idx = super::tool_spreadsheet_range::row_index(row_str)?;
    let col = u16::try_from(col_idx).ok()?;
    let row = u32::try_from(row_idx).ok()?;
    Some((row, col))
}
