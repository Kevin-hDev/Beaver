use serde_json::{json, Map, Value};
use std::path::Path;

const MAX_TEXT: usize = 200;
const MAX_JSON: usize = 1000;

pub fn summarize(tool_name: &str, args: &Value, working_dir: &Path) -> Option<Value> {
    let mut out = Map::new();
    match tool_name {
        "read_file" | "write_file" | "edit_file" | "list_dir" | "read_document"
        | "write_document" | "read_spreadsheet" | "write_spreadsheet" | "read_image" => {
            add_path(&mut out, "path", args, working_dir);
            add_path(&mut out, "input_path", args, working_dir);
            add_path(&mut out, "output_path", args, working_dir);
        }
        "bash" => {
            if let Some(command) = args["command"].as_str() {
                out.insert("command".to_string(), json!(safe_command(command)));
            }
        }
        "grep" => {
            add_text(&mut out, "pattern", args);
            add_text(&mut out, "glob", args);
            add_path(&mut out, "path", args, working_dir);
        }
        "glob" => {
            add_text(&mut out, "pattern", args);
            add_path(&mut out, "path", args, working_dir);
        }
        "web_search" | "web_fetch" => {
            add_text(&mut out, "query", args);
            add_text(&mut out, "url", args);
        }
        "todo_write" => {
            let count = args["todos"].as_array().map(|v| v.len()).unwrap_or(0);
            out.insert("todos_count".to_string(), json!(count));
        }
        "todo_resume" => add_text(&mut out, "id", args),
        "todo_delete" => {
            add_text(&mut out, "id", args);
            add_bool(&mut out, "active", args);
        }
        "todo_pause" => add_text(&mut out, "reason", args),
        "todo_history" | "agent_diagnostics" | "ask_user_choice" | "planmode" | "exitplanmode" => {}
        "mcp" => add_text(&mut out, "tool_id", args),
        _ => {}
    }
    bounded_value(Value::Object(out))
}

fn add_text(out: &mut Map<String, Value>, key: &str, args: &Value) {
    if let Some(value) = args[key].as_str() {
        out.insert(key.to_string(), json!(safe_text(value)));
    }
}

fn add_bool(out: &mut Map<String, Value>, key: &str, args: &Value) {
    if let Some(value) = args[key].as_bool() {
        out.insert(key.to_string(), json!(value));
    }
}

fn add_path(out: &mut Map<String, Value>, key: &str, args: &Value, working_dir: &Path) {
    if let Some(value) = args[key].as_str() {
        out.insert(key.to_string(), json!(safe_path(value, working_dir)));
    }
}

fn safe_path(value: &str, working_dir: &Path) -> String {
    if looks_sensitive(value) {
        return "[redacted]".to_string();
    }
    let path = Path::new(value);
    if !path.is_absolute() {
        return safe_relative_path(path);
    }
    if let Ok(rel) = path.strip_prefix(working_dir) {
        return safe_relative_path(rel);
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("[external]/{}", safe_text(name)))
        .unwrap_or_else(|| "[external path]".to_string())
}

fn safe_relative_path(path: &Path) -> String {
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return "[unsafe path]".to_string();
    }
    safe_text(&path.to_string_lossy())
}

fn safe_command(command: &str) -> String {
    if looks_sensitive(command) {
        return "[redacted command]".to_string();
    }
    safe_text(command)
}

fn safe_text(value: &str) -> String {
    let clean = value.replace(['\n', '\r', '\t'], " ");
    clip_chars(clean.trim(), MAX_TEXT)
}

fn bounded_value(value: Value) -> Option<Value> {
    if value.as_object().is_some_and(|obj| obj.is_empty()) {
        return None;
    }
    let Ok(serialized) = serde_json::to_string(&value) else {
        return None;
    };
    if serialized.chars().count() <= MAX_JSON {
        return Some(value);
    }
    Some(json!({ "summary": clip_chars(&serialized, MAX_JSON) }))
}

fn clip_chars(value: &str, max: usize) -> String {
    let mut end = value.len();
    for (count, (idx, _)) in value.char_indices().enumerate() {
        if count == max {
            end = idx;
            break;
        }
    }
    let mut clipped = value[..end].to_string();
    if value[end..].chars().next().is_some() {
        clipped.push_str("...");
    }
    clipped
}

fn looks_sensitive(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "api_key",
        "apikey",
        "authorization",
        "bearer ",
        "password",
        "secret",
        "token",
        "sk-",
        ".env",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}
