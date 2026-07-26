use calamine::{Data, Reader};
use serde_json::Value;
use std::path::Path;

const EXTRA_ROW_FOR_TRUNCATION: usize = 1;
const HARD_MAX_COLS: usize = 1000;

fn cell_or_formula(
    cell: &Data,
    abs_row: u32,
    abs_col: u32,
    formulas: Option<&calamine::Range<String>>,
) -> Value {
    if let Some(f_range) = formulas {
        if let Some(formula) = f_range.get_value((abs_row, abs_col)) {
            if !formula.is_empty() {
                return Value::String(format!("={formula}"));
            }
        }
    }
    cell_to_json(cell)
}

pub fn cell_to_json(cell: &Data) -> Value {
    match cell {
        Data::Empty => Value::Null,
        Data::String(s) => Value::String(s.clone()),
        Data::Float(f) => serde_json::json!(f),
        Data::Int(i) => serde_json::json!(i),
        Data::Bool(b) => Value::Bool(*b),
        Data::DateTime(dt) => Value::String(format!("{}", dt.as_f64())),
        Data::DateTimeIso(s) => Value::String(s.clone()),
        Data::DurationIso(s) => Value::String(s.clone()),
        Data::Error(e) => Value::String(format!("#ERR:{:?}", e)),
    }
}

pub fn read_excel(
    resolved: &Path,
    sheet: Option<&str>,
    range_str: Option<&str>,
    max_rows: usize,
) -> Result<Value, String> {
    super::tool_spreadsheet_write::validate_spreadsheet_input(resolved)?;
    let mut workbook: calamine::Sheets<_> = calamine::open_workbook_auto(resolved)
        .map_err(|_| "Impossible d'ouvrir le fichier".to_string())?;

    let sheet_names = workbook.sheet_names().to_owned();
    if sheet_names.is_empty() {
        return Err("Le fichier ne contient aucune feuille".into());
    }

    let effective_sheet = match sheet {
        Some(name) if !name.trim().is_empty() => Some(name),
        _ => None,
    };

    let sheet_name = match effective_sheet {
        Some(name) => {
            if sheet_names.contains(&name.to_string()) {
                name.to_string()
            } else {
                return Err(format!("Feuille '{}' introuvable", name));
            }
        }
        None => sheet_names[0].clone(),
    };

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|_| "Impossible de lire la feuille".to_string())?;

    // Calamine matérialise la feuille entière en RAM sous forme de `Range<Data>`
    // dense. Une feuille malveillante peut déclarer une dimension énorme
    // (ex. `A1:XFD1048576`) et allouer plusieurs Go. On borne le nombre de
    // cellules AVANT d'itérer — `max_rows`/`HARD_MAX_COLS` ne limitent que le
    // résultat JSON, pas la structure interne chargée ici.
    let height = range.height();
    let width = range.width();
    super::tool_office_limits::ensure_cell_budget(height as u64, width as u64, "Feuille")?;

    let formulas = workbook.worksheet_formula(&sheet_name).ok();
    let (start_row, start_col) = range.start().unwrap_or((0, 0));

    let bounds = super::tool_spreadsheet_read::parse_range(range_str.unwrap_or(""));
    let all_rows: Vec<Vec<Value>> = range
        .rows()
        .enumerate()
        .filter_map(|(row_idx, row): (usize, &[Data])| {
            if let Some((rs, cs, re, ce)) = bounds {
                if row_idx < rs || row_idx > re {
                    return None;
                }
                let filtered: Vec<Value> = row
                    .iter()
                    .enumerate()
                    .filter(|(col_idx, _)| *col_idx >= cs && *col_idx <= ce)
                    .take(HARD_MAX_COLS)
                    .map(|(col_idx, cell)| {
                        let abs_row = start_row + row_idx as u32;
                        let abs_col = start_col + col_idx as u32;
                        cell_or_formula(cell, abs_row, abs_col, formulas.as_ref())
                    })
                    .collect();
                Some(filtered)
            } else {
                Some(
                    row.iter()
                        .enumerate()
                        .take(HARD_MAX_COLS)
                        .map(|(col_idx, cell)| {
                            let abs_row = start_row + row_idx as u32;
                            let abs_col = start_col + col_idx as u32;
                            cell_or_formula(cell, abs_row, abs_col, formulas.as_ref())
                        })
                        .collect(),
                )
            }
        })
        .take(
            max_rows
                .saturating_add(EXTRA_ROW_FOR_TRUNCATION)
                .saturating_add(1),
        )
        .collect();

    super::tool_spreadsheet_read::build_result(all_rows, max_rows, &sheet_name, &sheet_names)
}
