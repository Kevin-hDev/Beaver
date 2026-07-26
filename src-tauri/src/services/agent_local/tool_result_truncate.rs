use crate::services::agent_local::types_tools::ToolResult;
use crate::services::paths::data_dir;

const MAX_CHARS_BASH: usize = 30_000;
const MAX_CHARS_GREP: usize = 10_000;
const MAX_CHARS_GLOB: usize = 5_000;
const MAX_CHARS_WEB_FETCH: usize = 50_000;
const MAX_CHARS_WEB_SEARCH: usize = 10_000;
const MAX_CHARS_LIST_DIR: usize = 10_000;
const PREVIEW_SIZE: usize = 2_000;

fn max_chars_for_tool(name: &str) -> Option<usize> {
    match name {
        "bash" => Some(MAX_CHARS_BASH),
        "grep" => Some(MAX_CHARS_GREP),
        "glob" => Some(MAX_CHARS_GLOB),
        "web_fetch" => Some(MAX_CHARS_WEB_FETCH),
        "web_search" => Some(MAX_CHARS_WEB_SEARCH),
        "list_dir" => Some(MAX_CHARS_LIST_DIR),
        _ => None,
    }
}

pub(crate) fn truncate_result(
    mut result: ToolResult,
    tool_name: &str,
    session_id: &str,
) -> ToolResult {
    if result.is_error {
        return result;
    }
    let Some(max) = max_chars_for_tool(tool_name) else {
        return result;
    };
    let total = result.content.chars().count();
    if total <= max {
        return result;
    }

    let persist_path = persist_result(&result.content, session_id);
    let preview: String = result.content.chars().take(PREVIEW_SIZE).collect();
    let omitted = total - PREVIEW_SIZE;
    let total_kb = total / 1024;

    let file_hint = match persist_path {
        Some(p) => format!("\n[Résultat complet disponible : {}]", p),
        None => String::new(),
    };

    result.content = format!(
        "[Résultat tronqué — {total_kb} Ko total, preview ci-dessous]{file_hint}\n{preview}\n[{omitted} chars omis]"
    );
    result.truncated = true;
    result
}

fn persist_result(content: &str, session_id: &str) -> Option<String> {
    let dir = data_dir().join("tool-results").join(session_id);
    std::fs::create_dir_all(&dir).ok()?;
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());
    let path = dir.join(&file_name);
    std::fs::write(&path, content).ok()?;
    Some(format!("tool-results/{session_id}/{file_name}"))
}
