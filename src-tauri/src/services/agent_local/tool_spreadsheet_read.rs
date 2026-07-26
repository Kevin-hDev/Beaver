use crate::services::agent_local::security::validate_read_path;
use crate::services::agent_local::tool_office_limits::{ensure_source_size, MAX_CSV_SOURCE_BYTES};
use crate::services::agent_local::types_tools::ToolResult;
use regex::Regex;
use serde_json::Value;
use std::io::BufRead;
use std::path::Path;
use std::sync::LazyLock;

const DEFAULT_MAX_ROWS: usize = 500;
const HARD_MAX_ROWS: usize = 5000;
const HARD_MAX_COLS: usize = 1000;

static RANGE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Z]+)(\d+):([A-Z]+)(\d+)$").unwrap());

pub fn col_letters_to_index(s: &str) -> usize {
    s.chars()
        .fold(0usize, |acc, c| acc * 26 + (c as usize - 'A' as usize + 1))
        - 1
}

pub fn parse_range(range_str: &str) -> Option<(usize, usize, usize, usize)> {
    let caps = RANGE_REGEX.captures(range_str)?;
    let col_start = col_letters_to_index(&caps[1]);
    let row_start: usize = caps[2].parse::<usize>().ok()?.saturating_sub(1);
    let col_end = col_letters_to_index(&caps[3]);
    let row_end: usize = caps[4].parse::<usize>().ok()?.saturating_sub(1);
    Some((row_start, col_start, row_end, col_end))
}

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
    let truncated = total > max_rows;
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
    ensure_source_size(resolved, MAX_CSV_SOURCE_BYTES, "CSV")?;
    let file =
        std::fs::File::open(resolved).map_err(|_| "Impossible de lire le fichier".to_string())?;
    let mut first_line = String::new();
    std::io::BufReader::new(file)
        .read_line(&mut first_line)
        .map_err(|_| "Impossible de lire le CSV".to_string())?;
    let delimiter = detect_csv_delimiter(&first_line);

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_path(resolved)
        .map_err(|_| "Impossible de lire le CSV".to_string())?;

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|_| "Impossible de lire les en-têtes".to_string())?
        .iter()
        .take(HARD_MAX_COLS)
        .map(|s| s.to_string())
        .collect();

    let mut rows: Vec<Vec<Value>> = Vec::new();
    let mut truncated = false;

    for record in rdr.records() {
        if rows.len() >= max_rows {
            truncated = true;
            break;
        }
        let rec = record.map_err(|_| "Erreur de lecture d'une ligne CSV".to_string())?;
        let row: Vec<Value> = rec.iter().map(|s| Value::String(s.to_string())).collect();
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
        return ToolResult::err("Le paramètre 'path' est requis");
    }

    let max = max_rows.unwrap_or(DEFAULT_MAX_ROWS).min(HARD_MAX_ROWS);

    let resolved = super::tool_office_utils::resolve_path(path, working_dir);

    let validated = match validate_read_path(&resolved, working_dir) {
        Ok(p) => p,
        Err(e) => return ToolResult::err(e),
    };

    let ext = validated
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let result = match ext.as_str() {
        "csv" | "tsv" => read_csv(&validated, max),
        "xlsx" | "xls" | "ods" | "xlsm" => {
            super::tool_spreadsheet_calamine::read_excel(&validated, sheet, range_str, max)
        }
        _ => {
            return ToolResult::err(
                "Format non supporté. Formats acceptés : xlsx, xls, ods, xlsm, csv, tsv",
            )
        }
    };

    match result {
        Ok(json) => ToolResult::ok(json.to_string()),
        Err(e) => ToolResult::err(e),
    }
}
