use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;

pub enum PreHookDecision {
    Allow,
    Deny(String),
}

fn is_protected_app_file(path_str: &str) -> bool {
    let path = std::path::Path::new(path_str);
    let data_dir = crate::services::paths::data_dir();
    let canonical = path.canonicalize().unwrap_or_else(|_| {
        path.parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.join(path.file_name().unwrap_or_default()))
            .unwrap_or_else(|| path.to_path_buf())
    });
    let data_canonical = data_dir.canonicalize().unwrap_or(data_dir);
    crate::services::agent_local::sensitive_data::PROTECTED_APP_FILES
        .iter()
        .any(|f| canonical == data_canonical.join(f))
}

pub fn run_pre_hooks(tool_name: &str, args: &Value) -> PreHookDecision {
    if tool_name == "bash"
        && args["command"]
            .as_str()
            .is_some_and(super::memory_paths::command_mentions_memory)
    {
        return PreHookDecision::Deny(
            "Utilisez les outils de fichiers pour accéder à la mémoire.".into(),
        );
    }

    if matches!(
        tool_name,
        "write_file"
            | "edit_file"
            | "read_file"
            | "write_spreadsheet"
            | "write_document"
            | "list_dir"
            | "glob"
            | "grep"
            | "read_spreadsheet"
            | "read_document"
            | "read_image"
    ) {
        if let Some(path) = args["path"].as_str() {
            if path.contains("..") {
                return PreHookDecision::Deny("Chemin avec '..' interdit".into());
            }
        }
    }

    if tool_name == "process_image" {
        for key in &["input_path", "output_path"] {
            if let Some(path) = args[*key].as_str() {
                if path.contains("..") {
                    return PreHookDecision::Deny("Chemin avec '..' interdit".into());
                }
            }
        }
    }

    if matches!(
        tool_name,
        "write_file" | "edit_file" | "write_spreadsheet" | "write_document"
    ) {
        if let Some(path) = args["path"].as_str() {
            if is_protected_app_file(path) {
                return PreHookDecision::Deny(
                    "Écriture interdite sur les fichiers de configuration de l'application".into(),
                );
            }
        }
    }

    PreHookDecision::Allow
}

/// Hooks exécutés APRÈS chaque tool call.
/// Peut modifier le résultat (ex: filtrer des données sensibles).
pub fn run_post_hooks(tool_name: &str, _args: &Value, mut result: ToolResult) -> ToolResult {
    if matches!(
        tool_name,
        "bash" | "bash_write" | "read_file" | "grep" | "glob" | "list_dir"
    ) {
        result.content =
            crate::services::agent_local::sensitive_data::redact_text(&result.content);
    }
    super::tool_dispatcher_error::enrich(result, tool_name)
}
