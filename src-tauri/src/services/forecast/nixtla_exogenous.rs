use serde_json::{Map, Value};

const MAX_CATEGORY_CHARS: usize = 256;

pub(super) fn read_row(row: &Map<String, Value>, columns: &[String]) -> Result<Vec<Value>, String> {
    columns
        .iter()
        .map(|column| match row.get(column) {
            Some(Value::Number(number)) => number
                .as_f64()
                .map(Value::from)
                .ok_or_else(|| "Covariables invalides".to_string()),
            Some(Value::Bool(flag)) => Ok(Value::from(if *flag { 1.0 } else { 0.0 })),
            Some(Value::String(category))
                if !category.trim().is_empty()
                    && category.chars().count() <= MAX_CATEGORY_CHARS =>
            {
                Ok(Value::String(category.clone()))
            }
            Some(Value::Null) | None => Err("Covariables invalides".into()),
            Some(_) => Err("Covariables invalides".into()),
        })
        .collect()
}

pub(super) fn transpose(rows: Vec<Vec<Value>>, feature_count: usize) -> Result<Vec<Value>, String> {
    if feature_count == 0 {
        return Err("Covariables invalides".into());
    }
    let mut columns = vec![Vec::with_capacity(rows.len()); feature_count];
    for row in rows {
        if row.len() != feature_count {
            return Err("Covariables invalides".into());
        }
        for (index, value) in row.into_iter().enumerate() {
            columns[index].push(value);
        }
    }
    Ok(columns.into_iter().map(Value::Array).collect())
}

pub(super) fn categorical_indices(
    history: &[Value],
    future: &[Value],
) -> Result<Vec<usize>, String> {
    if history.len() != future.len() {
        return Err("Covariables invalides".into());
    }
    let mut indices = Vec::new();
    for (index, (past, next)) in history.iter().zip(future).enumerate() {
        let past = past.as_array().ok_or("Covariables invalides")?;
        let next = next.as_array().ok_or("Covariables invalides")?;
        let mut categorical = None;
        for value in past.iter().chain(next) {
            let kind = match value {
                Value::Null => continue,
                Value::String(_) => true,
                Value::Number(_) => false,
                _ => return Err("Covariables invalides".into()),
            };
            if categorical.is_some_and(|current| current != kind) {
                return Err("Covariables invalides".into());
            }
            categorical = Some(kind);
        }
        match categorical {
            Some(true) => indices.push(index),
            Some(false) => {}
            None => return Err("Covariables invalides".into()),
        }
    }
    Ok(indices)
}
