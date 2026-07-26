use super::limits::{MAX_CELL_CHARS, MAX_INPUT_COLUMNS, MAX_INPUT_ROWS};
use super::numeric_parse::parse_finite_number;
use serde_json::Value;
use std::collections::HashSet;

pub fn preview_to_records_json(preview: &Value) -> Result<String, String> {
    if preview["truncated"].as_bool().unwrap_or(false) {
        return Err("Fichier trop volumineux".into());
    }
    let headers = preview["headers"].as_array().ok_or("En-têtes manquants")?;
    let rows = preview["rows"].as_array().ok_or("Lignes manquantes")?;
    if rows.len() > MAX_INPUT_ROWS {
        return Err("Jeu de données trop volumineux".into());
    }
    let mapping = map_headers(headers)?;
    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        let cells = row.as_array().ok_or("Format de ligne invalide")?;
        if cells.len() > MAX_INPUT_COLUMNS {
            return Err("Trop de colonnes".into());
        }
        let mut object = serde_json::Map::with_capacity(mapping.len());
        for (index, name) in &mapping {
            let value = cells.get(*index).cloned().unwrap_or(Value::Null);
            object.insert(name.clone(), normalize_cell(value)?);
        }
        records.push(Value::Object(object));
    }
    serde_json::to_string(&records).map_err(|_| "Conversion fichier impossible".into())
}

fn map_headers(headers: &[Value]) -> Result<Vec<(usize, String)>, String> {
    if headers.is_empty() || headers.len() > MAX_INPUT_COLUMNS {
        return Err("En-têtes invalides".into());
    }
    let mut seen = HashSet::with_capacity(headers.len());
    let mut mapping = Vec::with_capacity(headers.len());
    for (index, header) in headers.iter().enumerate() {
        let raw = header.as_str().ok_or("En-tête invalide")?.trim();
        let name = if raw.is_empty() {
            format!("column_{}", index + 1)
        } else {
            raw.to_string()
        };
        if name.chars().count() > super::limits::MAX_COLUMN_CHARS {
            return Err("En-tête trop long".into());
        }
        if !seen.insert(name.clone()) {
            return Err("En-têtes dupliqués".into());
        }
        mapping.push((index, name));
    }
    Ok(mapping)
}

fn normalize_cell(value: Value) -> Result<Value, String> {
    let Some(raw) = value.as_str() else {
        return Ok(value);
    };
    let trimmed = raw.trim();
    if trimmed.chars().count() > MAX_CELL_CHARS {
        return Err("Valeur de cellule trop longue".into());
    }
    if trimmed.is_empty() {
        return Ok(Value::String(String::new()));
    }
    match parse_finite_number(trimmed) {
        Ok(number) => serde_json::Number::from_f64(number)
            .map(Value::Number)
            .ok_or("Valeur numérique invalide".into()),
        Err(_) => Ok(Value::String(trimmed.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keeps_original_positions_for_blank_headers() {
        let json = preview_to_records_json(&json!({
            "headers": ["date", "", "sales"],
            "rows": [["2026-01-01", "ignored", "12,5"]]
        }))
        .unwrap();
        let rows: Vec<Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(rows[0]["column_2"], "ignored");
        assert_eq!(rows[0]["sales"], 12.5);
    }

    #[test]
    fn rejects_duplicate_headers() {
        let result = preview_to_records_json(&json!({
            "headers": ["date", "date"],
            "rows": []
        }));
        assert_eq!(result, Err("En-têtes dupliqués".into()));
    }
}
