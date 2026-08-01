use crate::services::agent_local::security::validate_read_path;
use crate::services::agent_local::tool_office_limits::{ensure_source_size, MAX_CSV_SOURCE_BYTES};
use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::io::BufRead;
use std::path::Path;

use super::tool_spreadsheet_error::SpreadsheetReadError;

const DEFAULT_MAX_ROWS: usize = 500;
const HARD_MAX_ROWS: usize = 5000;
const HARD_MAX_COLS: usize = 1000;

pub fn build_result(
    all_rows: Vec<Vec<Value>>,
    max_rows: usize,
    sheet_name: &str,
    sheet_names: &[String],
) -> Result<Value, String> {
    if all_rows.is_empty() {
        return Ok(serde_json::json!({
            "sheet": sheet_name,
            "headers": [],
            "rows": [],
            "total_rows": 0,
            "sheets": sheet_names,
            "truncated": false
        }));
    }

    let columns_truncated = all_rows.iter().any(|row| row.len() > HARD_MAX_COLS);

    let headers: Vec<String> = all_rows[0]
        .iter()
        .take(HARD_MAX_COLS)
        .map(|v| match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        })
        .collect();

    let data_rows: Vec<Vec<Value>> = all_rows
        .into_iter()
        .skip(1)
        .map(|row| row.into_iter().take(HARD_MAX_COLS).collect())
        .collect();
    let total = data_rows.len();
    let truncated = total > max_rows || columns_truncated;
    let rows: Vec<Vec<Value>> = data_rows.into_iter().take(max_rows).collect();

    Ok(serde_json::json!({
        "sheet": sheet_name,
        "headers": headers,
        "rows": rows,
        "total_rows": total,
        "sheets": sheet_names,
        "truncated": truncated
    }))
}

fn detect_csv_delimiter(first_line: &str) -> u8 {
    let comma_count = first_line.matches(',').count();
    let semicolon_count = first_line.matches(';').count();
    let tab_count = first_line.matches('\t').count();
    if tab_count >= comma_count && tab_count >= semicolon_count {
        b'\t'
    } else if semicolon_count >= comma_count {
        b';'
    } else {
        b','
    }
}

pub fn read_csv(resolved: &Path, max_rows: usize) -> Result<Value, String> {
    read_csv_classified(resolved, max_rows)
        .map_err(|error| error.message().to_string())
}

pub(super) fn read_csv_classified(
    resolved: &Path,
    max_rows: usize,
) -> Result<Value, SpreadsheetReadError> {
    ensure_source_size(resolved, MAX_CSV_SOURCE_BYTES, "CSV")
        .map_err(SpreadsheetReadError::source)?;
    let file = std::fs::File::open(resolved)
        .map_err(|_| SpreadsheetReadError::read("Impossible de lire le fichier"))?;
    let mut first_line = String::new();
    std::io::BufReader::new(file)
        .read_line(&mut first_line)
        .map_err(|_| SpreadsheetReadError::read("Impossible de lire le CSV"))?;
    let delimiter = detect_csv_delimiter(&first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_path(resolved)
        .map_err(|_| SpreadsheetReadError::read("Impossible de lire le CSV"))?;

    let (headers, mut truncated) = {
        let source_headers = rdr
            .headers()
            .map_err(|_| {
                SpreadsheetReadError::invalid(
                    "spreadsheet_csv_invalid",
                    "Impossible de lire les en-têtes",
                )
            })?;
        let headers: Vec<String> = source_headers
            .iter()
            .take(HARD_MAX_COLS)
            .map(str::to_string)
            .collect();
        (headers, source_headers.len() > HARD_MAX_COLS)
    };

    let mut rows: Vec<Vec<Value>> = Vec::new();

    for record in rdr.records() {
        if rows.len() >= max_rows {
            truncated = true;
            break;
        }
        let rec = record.map_err(|_| {
            SpreadsheetReadError::invalid(
                "spreadsheet_csv_invalid",
                "Erreur de lecture d'une ligne CSV",
            )
        })?;
        truncated |= rec.len() > HARD_MAX_COLS;
        let row = rec
            .iter()
            .take(HARD_MAX_COLS)
            .map(|value| Value::String(value.to_string()))
            .collect();
        rows.push(row);
    }

    let total = rows.len();
    Ok(serde_json::json!({
        "sheet": "csv",
        "headers": headers,
        "rows": rows,
        "total_rows": total,
        "sheets": ["csv"],
        "truncated": truncated
    }))
}

pub async fn read_spreadsheet(
    path: &str,
    sheet: Option<&str>,
    range_str: Option<&str>,
    max_rows: Option<usize>,
    working_dir: &Path,
) -> ToolResult {
    if path.is_empty() {
        return ToolResult::validation(
            "spreadsheet_path_required",
            "Le paramètre 'path' est requis",
        );
    }

    let max = max_rows.unwrap_or(DEFAULT_MAX_ROWS).min(HARD_MAX_ROWS);

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_read_path(&resolved, working_dir) {
        Ok(p) => p,
        Err(error) => {
            return super::tool_file_error::path_failure(
                error,
                "spreadsheet_not_found",
                "spreadsheet_read_denied",
                "invalid_spreadsheet_path",
            )
        }
    };

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let result = match ext.as_str() {
        "csv" | "tsv" => read_csv_classified(&validated, max),
        "xlsx" | "xls" | "ods" | "xlsm" => {
            super::tool_spreadsheet_calamine::read_excel_classified(
                &validated,
                sheet,
                range_str,
                max,
            )
        }
        _ => {
            return ToolResult::validation(
                "spreadsheet_format_unsupported",
                "Format non supporté. Formats acceptés : xlsx, xls, ods, xlsm, csv, tsv",
            )
        }
    };

    match result {
        Ok(json) => {
            let truncated = json["truncated"].as_bool().unwrap_or(false);
            let mut result = ToolResult::ok(json.to_string());
            result.mark_truncated(truncated);
            result
        }
        Err(error) => error.into_tool_result(),
    }
}
