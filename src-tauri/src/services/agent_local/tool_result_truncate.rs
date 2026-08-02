use crate::services::agent_local::types_tools::ToolResult;
use crate::services::paths::data_dir;

const MAX_CHARS_BASH: usize = 30_000;
const MAX_CHARS_GREP: usize = 10_000;
const MAX_CHARS_GLOB: usize = 5_000;
const MAX_CHARS_WEB_FETCH: usize = 50_000;
const MAX_CHARS_WEB_SEARCH: usize = 10_000;
const MAX_CHARS_LIST_DIR: usize = 10_000;
const MAX_CHARS_ERROR: usize = 30_000;
const PREVIEW_SIZE: usize = 2_000;

fn max_chars_for_tool(name: &str) -> Option<usize> {
    match name {
        "bash" | "bash_control" => Some(MAX_CHARS_BASH),
        "grep" => Some(MAX_CHARS_GREP),
        "glob" => Some(MAX_CHARS_GLOB),
        "web_fetch" => Some(MAX_CHARS_WEB_FETCH),
        "web_search" => Some(MAX_CHARS_WEB_SEARCH),
        "list_dir" => Some(MAX_CHARS_LIST_DIR),
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

    let preview = result.content.chars().take(PREVIEW_SIZE).collect();
    let full_content = std::mem::take(&mut result.content);
    let persist_path = persist_result(full_content, session_id).await;
    apply_truncation(result, preview, persist_path, total)
}

fn apply_truncation(
    mut result: ToolResult,
    preview: String,
    persist_path: Option<String>,
    total: usize,
) -> ToolResult {
    let omitted = total - PREVIEW_SIZE;
    let total_kb = total / 1024;

    let file_hint = match persist_path.as_deref() {
        Some(path) => format!("\n[Résultat complet disponible : {path}]"),
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

async fn persist_result(content: String, session_id: &str) -> Option<String> {
    super::session_store::validate_session_id(session_id).ok()?;
    let dir = data_dir().join("tool-results").join(session_id);
    let file_name = format!("{}.txt", uuid::Uuid::new_v4());
    let path = dir.join(&file_name);
    crate::services::private_store::atomic_write_async(path.clone(), content.into_bytes())
        .await
        .ok()?;
    Some(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ordinary_result_under_limit_is_unchanged() {
        let result = truncate_result(ToolResult::ok("small"), "bash", "unused").await;

        assert_eq!(result.content, "small");
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn large_errors_are_bounded_and_the_full_result_is_retained() {
        let session_id = uuid::Uuid::new_v4().to_string();
        let full = "é".repeat(MAX_CHARS_ERROR + 1);
        let result = truncate_result(
            ToolResult::external("test_extension_failure", full.clone(), false),
            "extension",
            &session_id,
        )
        .await;

        assert!(result.is_error);
        assert!(result.truncated);
        assert!(result.content.contains("[Résultat tronqué"));
        let directory = data_dir().join("tool-results").join(&session_id);
        let files = std::fs::read_dir(&directory)
            .expect("persisted result directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("persisted result entries");
        assert_eq!(files.len(), 1);
        assert_eq!(std::fs::read_to_string(files[0].path()).unwrap(), full);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[tokio::test]
    async fn truncation_is_utf8_safe() {
        let session_id = uuid::Uuid::new_v4().to_string();
        let result = truncate_result(
            ToolResult::ok("🎉".repeat(MAX_CHARS_GLOB + 1)),
            "glob",
            &session_id,
        )
        .await;

        assert!(result.truncated);
        assert!(result.content.is_char_boundary(result.content.len()));
        let _ = std::fs::remove_dir_all(data_dir().join("tool-results").join(session_id));
    }

    #[tokio::test]
    async fn tools_without_a_success_limit_remain_unchanged() {
        let content = "r".repeat(100_000);
        let result =
            truncate_result(ToolResult::ok(content.clone()), "read_file", "unused").await;

        assert_eq!(result.content, content);
        assert!(!result.truncated);
    }

    #[test]
    fn persistence_failure_is_explicit_and_does_not_change_an_error_to_success() {
        let result = apply_truncation(
            ToolResult::execution("test_failure", "", false),
            "x".repeat(PREVIEW_SIZE),
            None,
            PREVIEW_SIZE + 1,
        );

        assert!(result.is_error);
        assert!(result.truncated);
        assert!(result.warnings[0].contains("pas pu être enregistré"));
    }

    #[tokio::test]
    async fn result_storage_rejects_an_invalid_session_path() {
        assert!(persist_result("secret".into(), "../outside").await.is_none());
    }

    #[tokio::test]
    async fn persisted_result_path_is_directly_readable_by_the_file_tool() {
        let session_id = uuid::Uuid::new_v4().to_string();
        let path = persist_result("complete result".into(), &session_id)
            .await
            .expect("persisted result path");
        assert!(std::path::Path::new(&path).is_absolute());

        let working_dir = tempfile::tempdir().unwrap();
        let result = super::super::tool_files::read_file(&path, working_dir.path(), 0, 10).await;
        assert!(!result.is_error);
        assert!(result.content.contains("complete result"));

        let _ = std::fs::remove_dir_all(data_dir().join("tool-results").join(session_id));
    }
}
