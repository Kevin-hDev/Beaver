use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct ParsedModelfile {
    pub from: Option<String>,
    pub system: Option<String>,
    pub template: Option<String>,
    pub license: Option<String>,
    pub parameters: HashMap<String, Value>,
}

impl ParsedModelfile {
    pub fn to_api_payload(&self, model_name: &str) -> Value {
        let mut payload = json!({ "model": model_name });
        let obj = payload.as_object_mut().unwrap();
        if let Some(v) = &self.from {
            obj.insert("from".into(), json!(v));
        }
        if let Some(v) = &self.system {
            obj.insert("system".into(), json!(v));
        }
        if let Some(v) = &self.template {
            obj.insert("template".into(), json!(v));
        }
        if let Some(v) = &self.license {
            obj.insert("license".into(), json!(v));
        }
        if !self.parameters.is_empty() {
            obj.insert("parameters".into(), json!(self.parameters));
        }
        payload
    }
}

pub fn parse_modelfile(content: &str) -> ParsedModelfile {
    let mut parsed = ParsedModelfile::default();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (keyword, rest) = match trimmed.split_once(char::is_whitespace) {
            Some(p) => (p.0.to_uppercase(), p.1.trim()),
            None => continue,
        };
        match keyword.as_str() {
            "FROM" => parsed.from = Some(rest.to_string()),
            "SYSTEM" => parsed.system = Some(read_value(rest, &mut lines)),
            "TEMPLATE" => parsed.template = Some(read_value(rest, &mut lines)),
            "LICENSE" => parsed.license = Some(read_value(rest, &mut lines)),
            "PARAMETER" => {
                if let Some((key, val)) = rest.split_once(char::is_whitespace) {
                    let value = parse_param_value(val.trim());
                    merge_parameter(&mut parsed.parameters, key.trim(), value);
                }
            }
            _ => {}
        }
    }
    parsed
}

fn read_value<'a, I>(rest: &str, lines: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = &'a str>,
{
    if let Some(stripped) = rest.strip_prefix("\"\"\"") {
        if let Some(end_idx) = stripped.find("\"\"\"") {
            return stripped[..end_idx].to_string();
        }
        let mut buf = String::from(stripped);
        buf.push('\n');
        for next in lines.by_ref() {
            if let Some(end_idx) = next.find("\"\"\"") {
                buf.push_str(&next[..end_idx]);
                return buf;
            }
            buf.push_str(next);
            buf.push('\n');
        }
        return buf.trim_end().to_string();
    }
    if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end_idx) = stripped.rfind('"') {
            return stripped[..end_idx].to_string();
        }
        return stripped.to_string();
    }
    rest.to_string()
}

pub fn parse_param_value(raw: &str) -> Value {
    if let Some(stripped) = raw.strip_prefix('"') {
        if let Some(end) = stripped.rfind('"') {
            return json!(stripped[..end].to_string());
        }
    }
    if let Ok(i) = raw.parse::<i64>() {
        return json!(i);
    }
    if let Ok(f) = raw.parse::<f64>() {
        return json!(f);
    }
    if raw == "true" {
        return json!(true);
    }
    if raw == "false" {
        return json!(false);
    }
    json!(raw)
}

pub fn merge_parameter(map: &mut HashMap<String, Value>, key: &str, value: Value) {
    if let Some(existing) = map.get_mut(key) {
        if let Value::Array(arr) = existing {
            arr.push(value);
            return;
        }
        let prev = std::mem::replace(existing, Value::Null);
        *existing = json!([prev, value]);
        return;
    }
    map.insert(key.to_string(), value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_modelfile() {
        let mf = r#"FROM llama3.2
SYSTEM "You are Mario"
PARAMETER temperature 0.7
PARAMETER num_ctx 32768"#;
        let p = parse_modelfile(mf);
        assert_eq!(p.from.as_deref(), Some("llama3.2"));
        assert_eq!(p.system.as_deref(), Some("You are Mario"));
        assert_eq!(p.parameters["temperature"], json!(0.7));
        assert_eq!(p.parameters["num_ctx"], json!(32768));
    }

    #[test]
    fn parses_triple_quoted_multiline_system() {
        let mf = "FROM llama3\nSYSTEM \"\"\"\nTu es un assistant.\nSois concis.\n\"\"\"\n";
        let p = parse_modelfile(mf);
        assert!(p.system.as_ref().unwrap().contains("Tu es un assistant."));
        assert!(p.system.as_ref().unwrap().contains("Sois concis."));
    }

    #[test]
    fn parses_triple_quoted_single_line() {
        let mf = r#"FROM x
SYSTEM """hello world"""
"#;
        let p = parse_modelfile(mf);
        assert_eq!(p.system.as_deref(), Some("hello world"));
    }

    #[test]
    fn api_payload_only_contains_set_fields() {
        let mf = "FROM base\nSYSTEM \"hi\"";
        let p = parse_modelfile(mf);
        let payload = p.to_api_payload("mymodel");
        let obj = payload.as_object().unwrap();
        assert_eq!(obj["model"], json!("mymodel"));
        assert_eq!(obj["from"], json!("base"));
        assert_eq!(obj["system"], json!("hi"));
        assert!(!obj.contains_key("template"));
        assert!(!obj.contains_key("parameters"));
    }

    #[test]
    fn stop_parameter_multiple_values() {
        let mf = r#"FROM x
PARAMETER stop "<|im_end|>"
PARAMETER stop "<|eot|>"
"#;
        let p = parse_modelfile(mf);
        let stop = &p.parameters["stop"];
        assert!(stop.is_array());
        assert_eq!(stop.as_array().unwrap().len(), 2);
    }

    #[test]
    fn ignores_comments_and_empty_lines() {
        let mf = "# comment\n\nFROM x\n# another\nSYSTEM \"a\"\n";
        let p = parse_modelfile(mf);
        assert_eq!(p.from.as_deref(), Some("x"));
        assert_eq!(p.system.as_deref(), Some("a"));
    }
}
