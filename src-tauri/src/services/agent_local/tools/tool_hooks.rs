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

    for field in super::tool_path_args::fields(tool_name)
        .iter()
        .filter(|field| field.usage != super::tool_path_args::PathUse::WorkingDirectory)
    {
        if args[field.name]
            .as_str()
            .is_some_and(|path| path.contains(".."))
        {
            return PreHookDecision::Deny("Chemin avec '..' interdit".into());
        }
    }

    if super::tool_path_args::first_value(
        tool_name,
        super::tool_path_args::PathUse::Write,
        args,
    )
    .is_some_and(is_protected_app_file)
    {
        return PreHookDecision::Deny(
            "Écriture interdite sur les fichiers de configuration de l'application".into(),
        );
    }

    PreHookDecision::Allow
}

/// Hooks exécutés APRÈS chaque tool call.
/// Peut modifier le résultat (ex: filtrer des données sensibles).
pub fn run_post_hooks(tool_name: &str, _args: &Value, mut result: ToolResult) -> ToolResult {
    if matches!(
        tool_name,
        "bash" | "bash_control" | "read_file" | "grep" | "glob" | "list_dir"
    ) {
        result.content = crate::services::agent_local::sensitive_data::redact_text(&result.content);
    }
    super::tool_dispatcher_error::enrich(result, tool_name)
}
