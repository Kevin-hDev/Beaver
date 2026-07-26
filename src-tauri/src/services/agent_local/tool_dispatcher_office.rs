use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;

fn log_tool_call(tool_name: &str, args: &Value) {
    let entry = serde_json::json!({
        "ts": chrono::Local::now().to_rfc3339(),
        "tool": tool_name,
        "args": crate::services::agent_local::sensitive_data::redact_json(args),
    });
    eprintln!("[office-tool] {}", entry);
    let dir = crate::services::paths::data_dir().join("logs");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("tool-calls.jsonl");
    if std::fs::metadata(&path)
        .map(|meta| meta.len() > MAX_LOG_BYTES)
        .unwrap_or(false)
    {
        let rotated = dir.join("tool-calls.jsonl.1");
        let _ = std::fs::rename(&path, rotated);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        let _ = writeln!(f, "{}", entry);
    }
}

pub async fn dispatch_office(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    _session_id: &str,
) -> Option<ToolResult> {
    log_tool_call(tool_name, args);
    let result = match tool_name {
        "read_spreadsheet" => {
            let path = args["path"].as_str().unwrap_or("");
            let sheet = args["sheet"].as_str();
            let range = args["range"].as_str();
            let max_rows = args["max_rows"].as_u64().map(|n| n as usize);
            super::tool_spreadsheet_read::read_spreadsheet(
                path,
                sheet,
                range,
                max_rows,
                working_dir,
            )
            .await
        }
        "read_document" => {
            let path = args["path"].as_str().unwrap_or("");
            let pages = args["pages"].as_str();
            super::tool_document_read::read_document(path, pages, working_dir).await
        }
        "read_image" => {
            let path = args["path"].as_str().unwrap_or("");
            super::tool_image_read::read_image(path, working_dir).await
        }
        "write_spreadsheet" => {
            let path = args["path"].as_str().unwrap_or("");
            let operations = &args["operations"];
            super::tool_spreadsheet_write::write_spreadsheet(path, operations, working_dir).await
        }
        "write_document" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = &args["content"];
            super::tool_document_write::write_document(path, content, working_dir).await
        }
        "process_image" => {
            let input_path = args["input_path"].as_str().unwrap_or("");
            let output_path = args["output_path"].as_str().unwrap_or("");
            let operations = &args["operations"];
            super::tool_image_process::process_image(
                input_path,
                output_path,
                operations,
                working_dir,
            )
            .await
        }
        _ => return None,
    };
    Some(result)
}
