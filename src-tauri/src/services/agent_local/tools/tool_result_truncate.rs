use crate::services::agent_local::types_tools::ToolResult;
use crate::services::paths::data_dir;

const MAX_CHARS_BASH: usize = 30_000;
const MAX_CHARS_GREP: usize = 10_000;
const MAX_CHARS_GLOB: usize = 5_000;
const MAX_CHARS_WEB_FETCH: usize = 50_000;
const MAX_CHARS_WEB_SEARCH: usize = 10_000;
const MAX_CHARS_LIST_DIR: usize = 10_000;
const MAX_CHARS_READ_FILE: usize = 200_000;
const MAX_CHARS_ERROR: usize = 30_000;
const PREVIEW_SIZE: usize = 2_000;
const FULL_RESULT_PREFIX: &str = "[Résultat complet disponible : ";

pub(crate) fn full_result_reference(content: &str) -> Option<&str> {
    content.lines().find(|line| {
        line.starts_with(FULL_RESULT_PREFIX)
            && line.ends_with(']')
            && line.contains("tool-results/")
    })
}

fn max_chars_for_tool(name: &str) -> Option<usize> {
    match name {
        "bash" | "bash_control" => Some(MAX_CHARS_BASH),
        "grep" => Some(MAX_CHARS_GREP),
        "glob" => Some(MAX_CHARS_GLOB),
        "web_fetch" => Some(MAX_CHARS_WEB_FETCH),
        "web_search" => Some(MAX_CHARS_WEB_SEARCH),
        "list_dir" => Some(MAX_CHARS_LIST_DIR),
        "read_file" => Some(MAX_CHARS_READ_FILE),
        _ => None,
    }
}

pub(crate) async fn truncate_result(
    mut result: ToolResult,
    tool_name: &str,
    session_id: &str,
) -> ToolResult {
    let max = if result.is_error {
        MAX_CHARS_ERROR
    } else {
        let Some(max) = max_chars_for_tool(tool_name) else {
            return result;
        };
        max
    };
    let total = result.content.chars().count();
    if total <= max {
        return result;
    }

    let use_read_file_envelope = !result.is_error && tool_name == "read_file";
    let existing_path = retained_source_path(&result, tool_name, session_id);
    let full_content = std::mem::take(&mut result.content);
    let persist_path = match existing_path {
        Some(path) => Some(path),
        None => persist_result(&full_content, session_id).await,
    };
    let preview_size = if use_read_file_envelope {
        read_file_preview_size(max, total, persist_path.as_deref())
    } else {
        PREVIEW_SIZE
    };
    let preview = full_content.chars().take(preview_size).collect();
    apply_truncation(result, preview, persist_path, total)
}

fn read_file_preview_size(max: usize, total: usize, persist_path: Option<&str>) -> usize {
    let total_kb = total / 1024;
    let file_hint = persist_path
        .map(|path| format!("\n{FULL_RESULT_PREFIX}{path}]"))
        .unwrap_or_default();
    let prefix =
        format!("[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]{file_hint}\n");
    const U64_DECIMAL_CHARS: usize = 20;
    let suffix_chars = "\n[ chars omis]".chars().count() + U64_DECIMAL_CHARS;
    max.saturating_sub(prefix.chars().count() + suffix_chars)
}

fn retained_source_path(result: &ToolResult, tool_name: &str, session_id: &str) -> Option<String> {
    if tool_name != "read_file" || super::session_store::validate_session_id(session_id).is_err() {
        return None;
    }
    let path = result.artifacts.source_path.as_ref()?;
    let directory = data_dir().join("tool-results").join(session_id);
    (path.parent() == Some(directory.as_path())).then(|| path.to_string_lossy().into_owned())
}

fn apply_truncation(
    mut result: ToolResult,
    preview: String,
    persist_path: Option<String>,
    total: usize,
) -> ToolResult {
    let omitted = total.saturating_sub(preview.chars().count());
    let total_kb = total / 1024;

    let file_hint = match persist_path.as_deref() {
        Some(path) => format!("\n{FULL_RESULT_PREFIX}{path}]"),
        None => String::new(),
    };

    result.content = format!(
        "[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]{file_hint}\n{preview}\n[{omitted} chars omis]"
    );
    result.mark_truncated(true);
    if persist_path.is_none() {
        result = result.with_warning("Le résultat complet n'a pas pu être enregistré.");
    }
    result
}

async fn persist_result(content: &str, session_id: &str) -> Option<String> {
    super::session_store::validate_session_id(session_id).ok()?;
    let dir = data_dir().join("tool-results").join(session_id);
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());
    let path = dir.join(&file_name);
    crate::services::private_store::atomic_write_async(path.clone(), content.as_bytes().to_vec())
        .await
        .ok()?;
    Some(path.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "tool_result_truncate_tests.rs"]
mod tests;
