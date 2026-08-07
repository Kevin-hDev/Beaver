const MAX_PARAMETER_ENTRIES: usize = 128;
const MAX_STOP_SEQUENCES: usize = 32;
const MAX_CUSTOM_PARAMETERS: usize = 64;
const MAX_PARAMETER_KEY_BYTES: usize = 64;
const MAX_PARAMETER_VALUE_BYTES: usize = 1024;

const INTEGER_PARAMETERS: &[&str] = &[
    "num_ctx",
    "num_predict",
    "draft_num_predict",
    "seed",
    "repeat_last_n",
    "top_k",
];
const DECIMAL_PARAMETERS: &[&str] = &["temperature", "repeat_penalty", "top_p", "min_p"];

pub fn validate_parameter_entries(entries: &[(String, String)]) -> Result<(), String> {
    if entries.len() > MAX_PARAMETER_ENTRIES {
        return Err(invalid_parameter());
    }
    let mut stop_count = 0;
    let mut custom_count = 0;
    for (key, value) in entries {
        let key = key.trim();
        if !valid_key(key)
            || value.is_empty()
            || value.len() > MAX_PARAMETER_VALUE_BYTES
            || value.contains("\"\"\"")
            || value
                .chars()
                .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
        {
            return Err(invalid_parameter());
        }
        let normalized_key = key.to_ascii_lowercase();
        if INTEGER_PARAMETERS.contains(&normalized_key.as_str()) {
            value
                .trim()
                .parse::<i64>()
                .map_err(|_| invalid_parameter())?;
        } else if DECIMAL_PARAMETERS.contains(&normalized_key.as_str()) {
            let number = value
                .trim()
                .parse::<f64>()
                .map_err(|_| invalid_parameter())?;
            if !number.is_finite() {
                return Err(invalid_parameter());
            }
        } else if normalized_key == "stop" {
            stop_count += 1;
            if stop_count > MAX_STOP_SEQUENCES {
                return Err(invalid_parameter());
            }
        } else {
            custom_count += 1;
            if custom_count > MAX_CUSTOM_PARAMETERS {
                return Err(invalid_parameter());
            }
        }
    }
    Ok(())
}

pub(crate) fn value_for_render<'a>(key: &str, value: &'a str) -> &'a str {
    let normalized_key = key.trim().to_ascii_lowercase();
    if INTEGER_PARAMETERS.contains(&normalized_key.as_str())
        || DECIMAL_PARAMETERS.contains(&normalized_key.as_str())
    {
        value.trim()
    } else {
        value
    }
}

fn valid_key(key: &str) -> bool {
    if key.is_empty() || key.len() > MAX_PARAMETER_KEY_BYTES {
        return false;
    }
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn invalid_parameter() -> String {
    "ollama-parameter-invalid".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_official_numeric_parameter_types() {
        assert!(validate_parameter_entries(&[("num_ctx".into(), "32768".into())]).is_ok());
        assert!(validate_parameter_entries(&[("num_ctx".into(), "1.5".into())]).is_err());
        assert!(validate_parameter_entries(&[("temperature".into(), "0.7".into())]).is_ok());
        assert!(validate_parameter_entries(&[("temperature".into(), "1e309".into())]).is_err());
    }

    #[test]
    fn bounds_and_validates_parameter_entries() {
        let too_many = vec![("stop".to_string(), "x".to_string()); 129];
        assert!(validate_parameter_entries(&too_many).is_err());
        let too_many_stops = vec![("stop".to_string(), "x".to_string()); 33];
        assert!(validate_parameter_entries(&too_many_stops).is_err());
        let too_many_custom = (0..65)
            .map(|index| (format!("custom_{index}"), "x".to_string()))
            .collect::<Vec<_>>();
        assert!(validate_parameter_entries(&too_many_custom).is_err());
        assert!(validate_parameter_entries(&[("invalid-key".into(), "1".into())]).is_err());
    }

    #[test]
    fn accepts_line_feeds_for_text_values_but_rejects_carriage_returns() {
        assert!(
            validate_parameter_entries(&[("stop".into(), "line one\nline two".into(),)]).is_ok()
        );
        assert!(validate_parameter_entries(&[(
            "future_option".into(),
            "safe\rSYSTEM hostile".into(),
        )])
        .is_err());
    }

    #[test]
    fn preserves_whitespace_text_and_rejects_unrepresentable_triple_quotes() {
        assert!(validate_parameter_entries(&[("stop".into(), " ".into())]).is_ok());
        assert!(validate_parameter_entries(&[("stop".into(), "say \"hi\"".into())]).is_ok());
        assert!(validate_parameter_entries(&[("stop".into(), "a\"\"\"b".into())]).is_err());
        assert!(validate_parameter_entries(&[("stop".into(), "a\u{7}b".into())]).is_err());
    }
}
